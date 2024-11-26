use http::Request;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, future::IntoFuture, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Subject {
    Maths,
    Geography,
    Spanish,
    Other,
}

impl Subject {
    pub const fn color(self) -> &'static str {
        match self {
            Self::Maths => "red",
            Self::Geography => "emerald",
            Self::Spanish => "yellow",
            Self::Other => "purple",
        }
    }

    pub const fn all() -> [Self; 4] {
        [Self::Maths, Self::Geography, Self::Spanish, Self::Other]
    }
}

impl Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Maths => write!(f, "maths"),
            Self::Geography => write!(f, "geography"),
            Self::Spanish => write!(f, "spanish"),
            Self::Other => write!(f, "other"),
        }
    }
}

impl FromStr for Subject {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "maths" => Ok(Self::Maths),
            "geography" => Ok(Self::Geography),
            "spanish" => Ok(Self::Spanish),
            "other" => Ok(Self::Other),
            _ => Err(()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("http request (trick that's worth it) failed: {0}")]
    Http(#[from] reqwest::Error),
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Set {
    pub id: String,
    pub title: String,
    pub description: String,
    pub parent: String,
    pub hash: String,
    pub subject: Subject,
}

impl Set {
    pub fn path(&self) -> String {
        let id = if self.parent.is_empty() {
            self.id.clone()
        } else {
            format!("{}/{}", self.parent, self.id)
        };
        format!("{}/{id}", self.subject)
    }

    pub async fn fetch_all(subject: &str) -> Result<Vec<Self>, ResourceError> {
        let response = reqwest::get(format!(
            "{}?subject={subject}&kind=allSets",
            include_str!("../DATABASE_URL")
        ))
        .await?
        .error_for_status()?;
        let body = response.text().await?;
        Ok(serde_json::from_str(&body)?)
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
