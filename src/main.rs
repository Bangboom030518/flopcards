// htmx-swapping
#![warn(clippy::pedantic, clippy::nursery, clippy::todo)]
use data::{Query, ResourceError, Set, Subject};
use html_builder::prelude::*;
use http::Method;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::fs;
use std::net::SocketAddr;
use tokio::net::TcpListener;
mod components;
mod data;

fn index(request: Request<hyper::body::Incoming>) -> Result<Html, ResourceError> {
    let subjects = Subject::fetch_all();
    let query = Query::from_request(&request);
    let sets = if let Ok(subject) = query.get("subject") {
        Set::fetch_all(&subject)?
    } else {
        Vec::new()
    };
    Ok(html("en")
        .child(
            head()
                .template()
                .style(include_str!("./output.css"))
                .title("flopcards - home")
                .raw_text("<script src='assets/htmx.min.js'></script>"),
        )
        .child(
            body()
                .class("p-8 grid place-items-center items-start gap-8 bg-neutral")
                .child(h1("flopcards"))
                .child(components::subject_menu(&subjects))
                .child(components::set_list(&sets))
                .child(components::loading_animation())
                .script(include_str!("../script.js")),
        ))
}

fn sets_view(request: Request<hyper::body::Incoming>) -> Result<Section, ResourceError> {
    let request = &request;
    let query = Query::from_request(request);
    let subject = query.get("subject")?;
    Ok(components::set_list(&Set::fetch_all(&subject)?))
}

fn set(path: &str) -> Result<Html, ResourceError> {
    let set = Set::get(path).unwrap_or_else(|_| todo!("404"));
    Ok(html("en")
        .child(
            head()
                .template()
                .style(include_str!("./output.css"))
                .title(format!("{} - flopcards", set.title))
                .raw_text("<script src='assets/htmx.min.js'></script>"),
        )
        .child(
            body()
                .class("p-8 grid place-items-center items-start gap-8 bg-neutral")
                .child(h1("Study"))
                .child(components::flashcard_stack(set.cards))
                .script(include_str!("../script.js")),
        ))
}

async fn router(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, ResourceError> {
    let path = request.uri().path();
    match *request.method() {
        Method::GET => {
            if path == "/" {
                index(request)?.response_ok()
            } else if path == "/favicon.ico" {
                let path = format!("/{}/assets/favicon.ico", env!("CARGO_MANIFEST_DIR"));
                let bytes = fs::read(path)
                    .map_err(|_| ResourceError::NotFound("/favicon.ico".to_string()))?;
                let response = http::Response::builder()
                    .header(http::header::CONTENT_TYPE, "image/x-icon")
                    .body(Full::new(Bytes::from(bytes)))
                    .unwrap();
                return Ok(response);
            } else if let Some(asset) = path.strip_prefix("/assets/") {
                let path = format!("/assets/{asset}");
                let bytes = fs::read(format!("/{}{path}", env!("CARGO_MANIFEST_DIR")))
                    .map_err(|_| ResourceError::NotFound(path))?;
                let content_type = match asset.split_once('.').expect("no mime type").1 {
                    "svg" => "image/svg+xml; charset=utf-8",
                    "jpg" | "jpeg" => "image/jpeg",
                    "min.js" | "js" => "text/javascript",
                    "mp3" => "audio/mpeg",
                    "webp" => "image/webp",
                    file_extension => todo!("handle '.{file_extension}' files"),
                };
                let response = http::Response::builder()
                    .header(http::header::CONTENT_TYPE, content_type)
                    .body(Full::new(Bytes::from(bytes)))
                    .unwrap();
                return Ok(response);
            } else if let Some(path) = path.strip_prefix("/view/") {
                match path {
                    "sets" => sets_view(request)?.response_ok(),
                    _ => Err(ResourceError::NotFound(format!("/view/{path}"))),
                }
            } else if let Some(path) = path.strip_prefix("/sets/") {
                set(path)?.response_ok()
            } else {
                Err(ResourceError::NotFound(path.to_string()))
            }
        }
        _ => Err(ResourceError::NotFound(path.to_string())),
    }
}

struct App;

impl App {
    async fn run(self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let listener = TcpListener::bind(addr).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(|request| router(request)))
                    .await
                {
                    eprintln!("Error serving connection: {err:?}");
                }
            });
        }
    }
}

impl shuttle_runtime::Service for App {
    fn bind<'async_trait>(
        self,
        addr: SocketAddr,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<(), shuttle_runtime::Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        Self: 'async_trait,
    {
        Box::pin(self.run(addr))
    }
}
#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> Result<App, shuttle_runtime::Error> {
    Ok(App)
}
