use futures::{StreamExt, TryStreamExt};
use http::Request;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::IntoFuture};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl Subject {
    pub async fn fetch_all(connection: &libsql::Connection) -> libsql::Result<Vec<Self>> {
        connection
            .query("SELECT id, name, color FROM subject ORDER BY id", ())
            .await?
            .into_stream()
            .map(|row| {
                row.and_then(|row| {
                    Ok(Self {
                        id: row.get_str(0)?.to_string(),
                        name: row.get_str(1)?.to_string(),
                        color: row.get_str(2)?.to_string(),
                    })
                })
            })
            .try_collect()
            .await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("http request (trick that's worth it) failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("database says no: {0}")]
    Database(#[from] libsql::Error),
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
    #[error("request body smells like sardeens (eeew): {0}")]
    Parse(#[from] serde_json::Error),
    #[error("path '{0}' couldn't be found. maybe it fell out of a coconut tree?")]
    NotFound(String),
    #[error("{0}")]
    Custom(String),
}

pub async fn body_to_string(
    request: Request<hyper::body::Incoming>,
) -> Result<String, ResourceError> {
    let body = request
        .into_body()
        .collect()
        .into_future()
        .await?
        .to_bytes()
        .to_vec();
    Ok(String::from_utf8_lossy(&body).to_string())
}

#[async_recursion::async_recursion]
pub async fn generate_set_id(
    connection: &libsql::Connection,
    name: &str,
    quota: u8,
) -> Result<String, ResourceError> {
    if quota == 0 {
        return Ok(Uuid::new_v4().to_string());
    }
    let name = name.to_lowercase().replace(' ', "-");
    let rows: Vec<_> = connection
        .query("SELECT id FROM cardset WHERE id = ?1", [name.clone()])
        .await?
        .into_stream()
        .try_collect()
        .await?;
    if rows.is_empty() {
        Ok(name.to_string())
    } else {
        generate_set_id(connection, &format!("{name}1"), quota - 1).await
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Set {
    pub id: String,
    pub title: String,
    pub description: String,
    pub parent: Option<String>,
    pub created: String,
    pub subject: Subject,
}

impl Set {
    pub async fn fetch_from_id(
        connection: &libsql::Connection,
        id: &str,
    ) -> libsql::Result<Option<Self>> {
        connection.query(
            "SELECT cardset.id, cardset.title, cardset.description, cardset.parent, cardset.created, subject.id, subject.name, subject.color \
            FROM cardset INNER JOIN subject ON cardset.subject=subject.id WHERE cardset.id = ?1;", [id]
        ).await?
                .next().await.and_then(|row| match row {
                    Some(row) => Ok(Some(Self {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: row.get(2)?,
                        parent: row.get(3)?,
                        created: row.get(4)?,
                        subject: Subject {
                            id: row.get(5)?,
                            name: row.get(6)?,
                            color: row.get(7)?,
                        },
                    })),
                    None => Ok(None)
                })
    }

    pub async fn fetch_all(
        connection: &libsql::Connection,
        subject: &str,
    ) -> libsql::Result<Vec<Self>> {
        let query = if subject == "all" {
            connection.query(
                "SELECT cardset.id, cardset.title, cardset.description, cardset.parent, cardset.created, subject.id, subject.name, subject.color \
                FROM cardset INNER JOIN subject ON cardset.subject=subject.id;", ()
            ).await?
        } else {
            connection.query(
                "SELECT cardset.id, cardset.title, cardset.description, cardset.parent, cardset.created, subject.id, subject.name, subject.color \
                FROM cardset INNER JOIN subject ON cardset.subject=subject.id WHERE cardset.subject = ?1;", [subject]
            ).await?
        };
        query
            .into_stream()
            .map(|row| {
                row.and_then(|row| {
                    Ok(Self {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: row.get(2)?,
                        parent: row.get(3)?,
                        created: row.get(4)?,
                        subject: Subject {
                            id: row.get(5)?,
                            name: row.get(6)?,
                            color: row.get(7)?,
                        },
                    })
                })
            })
            .try_collect()
            .await
    }
}
/*
SELECT cardset.id, cardset.title, cardset.description, cardset.parent, cardset.created, subject.id, subject.name, subject.color FROM cardset WHERE subject = 'geography' INNER JOIN subject ON cardset.subject=subject.id;
 */
#[derive(Clone, Debug, Default)]
pub struct Query(pub HashMap<String, String>);

impl Query {
    pub fn from_request<T>(request: &Request<T>) -> Self {
        request
            .uri()
            .query()
            .map(Self::from_str)
            .unwrap_or_default()
    }

    pub fn from_str(query: &str) -> Self {
        Self(
            url::form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect(),
        )
    }

    pub fn get(&self, key: &str) -> Result<String, ResourceError> {
        self.0
            .get(key)
            .ok_or_else(|| ResourceError::Custom(format!("Expected query param '{key}'")))
            .map(ToString::to_string)
    }
}
