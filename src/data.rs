use futures::{StreamExt, TryStreamExt};
use http::Request;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, fs, future::IntoFuture};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rating {
    Terrible,
    Bad,
    Ok,
    Good,
    Perfect,
}

impl Rating {
    pub const fn all() -> [Self; 5] {
        [
            Self::Terrible,
            Self::Bad,
            Self::Ok,
            Self::Good,
            Self::Perfect,
        ]
    }
}

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Terrible => "terrible",
            Self::Bad => "bad",
            Self::Ok => "ok",
            Self::Good => "good",
            Self::Perfect => "perfect",
        };
        write!(f, "{string}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub term: String,
    pub definition: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    pub color: String,
}

impl Subject {
    pub fn fetch_all() -> Vec<Self> {
        macros::subjects!()
            .into_iter()
            .map(|(name, color)| Self {
                name: name.to_string(),
                color: color.to_string(),
            })
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("http request (trick that's worth it) failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
    #[error("request body smells like sardeens (eeew): {0}")]
    ParseJson(#[from] serde_json::Error),
    #[error("database didn't want to see a trick that's worth it: {0}")]
    ParseToml(#[from] toml::de::Error),
    #[error("path '{0}' couldn't be found. maybe it fell out of a coconut tree?")]
    NotFound(String),
    #[error("{0}")]
    Custom(String),
    #[error("file system couldn't find the file (where it's at?)")]
    Io(#[from] std::io::Error),
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

#[derive(Clone, Debug, Deserialize)]
pub struct Set {
    pub path: String,
    pub title: String,
    pub description: String,
    pub subject: Subject,
    pub cards: Vec<Card>,
}

impl Set {
    pub fn get(path: &str) -> Result<Self, ResourceError> {
        let config = fs::read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }

    pub fn fetch_all(subject: &str) -> Result<Vec<Self>, ResourceError> {
        let mut sets = Vec::new();
        for entry in fs::read_dir(format!("./flashcards/{subject}"))? {
            let entry = entry.unwrap();
            let name = entry.file_name().into_string().unwrap();
            if !(entry.file_type()?.is_file() && name.ends_with(".toml")) {
                continue;
            }
            sets.push(Self::get(&format!("./flashcards/{subject}/{name}"))?);
        }
        Ok(sets)
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
