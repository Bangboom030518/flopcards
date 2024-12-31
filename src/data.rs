use futures::{StreamExt, TryStreamExt};
use http::Request;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::IntoFuture, str::FromStr};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl Subject {
    pub fn name(&self) -> String {
        self.id.replace("-", " ")
    }

    pub fn icon_path(&self) -> String {
        format!("assets/{}.svg", self.id)
    }

    pub async fn fetch_from_id(id: &str) -> Self {
        todo!("query subject")
    }

    pub async fn fetch_all(connection: &libsql::Connection) -> libsql::Result<Vec<Self>> {
        let rows = connection
            .query("SELECT id, name, color FROM subject ORDER BY id", ())
            .await
            .unwrap_or_else(|error| todo!("db error: {error}"));

        rows.into_stream()
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

pub async fn generate_set_id(
    connection: &libsql::Connection,
    name: &str,
    quota: u8,
) -> Result<String, ResourceError> {
    if quota <= 0 {
        return Ok(Uuid::new_v4().to_string());
    }
    let name = name.to_lowercase().replace(' ', "-");
    let rows: Vec<_> = connection
        .query("SELECT id FROM cardset WHERE id = ?0", [name])
        .await?
        .into_stream()
        .try_collect()
        .await?;

    if rows.is_empty() {
        Ok(name.to_string())
    } else {
        if let Some(n) = name.strip_suffix(regex::Regex::new(r"\d+$").unwrap()) {
            let Ok(n) = n.parse::<u32>() else {
                return generate_set_id(connection, name, 0).await;
            };
            generate_set_id(connection, &format!("{name}{}", n + 1), quota - 1).await
        } else {
            generate_set_id(connection, &format!("{name}1"), quota - 1).await
        }
    }
}

pub fn parse_query(query_string: &str) -> HashMap<String, String> {
    dbg!(querystring::querify(query_string));
    HashMap::from_iter(
        querystring::querify(query_string)
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string())),
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Set {
    pub id: String,
    pub title: String,
    pub description: String,
    pub parent: Option<String>,
    pub created: i32,
    pub subject: Subject,
}

impl Set {
    pub fn path(&self) -> String {
        let id = if let Some(parent) = &self.parent {
            format!("{}/{}", parent, self.id)
        } else {
            self.id.clone()
        };
        format!("{}/{id}", self.subject.id)
    }

    pub async fn fetch_all(
        connection: &libsql::Connection,
        subject: &str,
    ) -> libsql::Result<Vec<Self>> {
        let query = if subject == "all" {
            connection.query(
                "SELECT cardset.id, cardset.title, cardset.description, cardset.parent, cardset.created, subject.id, subject.name, subject.color \
                FROM cardset INNER JOIN subject ON cardset.subject=subject.id", ()
            ).await?
        } else {
            connection.query(
                "SELECT cardset.id, cardset.title, cardset.description, cardset.parent, cardset.created, subject.id, subject.name, subject.color \
                FROM cardset WHERE subject = ?0 INNER JOIN subject ON cardset.subject=subject.id", [subject]
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
