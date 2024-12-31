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
use std::sync::Arc;
use tokio::net::TcpListener;
mod components;
mod data;

fn create_set_popup(subjects: &[Subject]) -> Dialog {
    dialog()
        .class("bg-transparent max-w-[60ch] w-full inset-0 m-auto")
        .id("create-set-dialog")
        .child(
            div()
                .class("card inset-0 relative")
                .child(
                    button("close-popup")
                        .class("w-[5ch] h-[5ch] top-2 right-2 absolute sound-stop-baby")
                        .hx_on("click", "this.parentElement.parentElement.close()")
                        .child(
                            img("/assets/close.svg", "close")
                                .size(24, 24)
                                .class("w-full h-full"),
                        ),
                )
                .child(h2("create set"))
                .child(
                    form(FormMethod::Post, "create-set")
                        .class("w-full grid gap-4")
                        .child(components::text_input(
                            "set-title",
                            "title",
                            "set title",
                            InputType::Text,
                            true,
                            None,
                        ))
                        .child(
                            textarea("description-input", "description")
                                .placeholder("description")
                                .class("text-black"),
                        )
                        .child(
                            select("subject-input", "subject")
                                .options(
                                    subjects.iter().map(|subject| (&subject.name, &subject.id)),
                                )
                                .class("text-black"),
                        )
                        .child(
                            components::button_with_icon("create-set", "create", "create set")
                                .class("input-accent"),
                        ),
                ),
        )
}

async fn index(
    connection: &libsql::Connection,
    request: Request<hyper::body::Incoming>,
) -> libsql::Result<Html> {
    let subjects = Subject::fetch_all(connection).await?;
    let query = Query::from_request(&request);
    let sets = if let Ok(subject) = query.get("subject") {
        Set::fetch_all(connection, &subject).await?
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
                .child(components::fab_dropdown(
                    "create",
                    "create",
                    ["set", "folder"].into_iter().map(|item| {
                        components::button_with_icon(format!("create-{item}"), item, item).onclick(
                            format!(
                                "javascript:document.getElementById('create-{item}-dialog').show()"
                            ),
                        )
                    }),
                ))
                .child(create_set_popup(&subjects))
                .child(components::loading_animation())
                .script(include_str!("../script.js")),
        ))
}

async fn sets_view(
    connection: &libsql::Connection,
    request: Request<hyper::body::Incoming>,
) -> Result<Section, ResourceError> {
    let request = &request;
    let query = Query::from_request(request);
    let subject = query.get("subject")?;
    Ok(components::set_list(
        &Set::fetch_all(connection, &subject).await?,
    ))
}

async fn set(connection: &libsql::Connection, path: &str) -> Result<Html, ResourceError> {
    let set = Set::fetch_from_id(connection, path)
        .await?
        .unwrap_or_else(|| todo!("404"));
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
                .child(h1(format!("{} cards", set.title)))
                .script(include_str!("../script.js")),
        ))
}

async fn edit_set(connection: &libsql::Connection, path: &str) -> Result<Html, ResourceError> {
    let set = Set::fetch_from_id(connection, path)
        .await?
        .ok_or_else(|| ResourceError::NotFound(format!("set '{path}'")))?;

    Ok(html("en")
        .child(
            head()
                .template()
                .style(include_str!("./output.css"))
                .title(format!("edit {} - flopcards", set.title))
                .raw_text("<script src='assets/htmx.min.js'></script>"),
        )
        .child(
            body()
                .class("p-8 grid place-items-center items-start gap-8 bg-neutral")
                .child(h1(format!("edit {}", set.title)))
                .child(
                    fieldset()
                        .class("w-full card grid gap-4")
                        .child(h2("edit title and description").class("text-left"))
                        .child(components::text_input(
                            "set-title",
                            "title",
                            "set title",
                            InputType::Text,
                            true,
                            Some(set.title),
                        ))
                        .child(
                            textarea("description-input", "description")
                                .text(&set.description)
                                .placeholder("description")
                                .class("text-black"),
                        )
                        .child(
                            components::button_with_icon("save-set", "publish", "save set")
                                .class("input-accent"),
                        ),
                )
                .script(include_str!("../script.js")),
        ))
}
async fn router(
    database: &libsql::Database,
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, ResourceError> {
    let connection = database
        .connect()
        .unwrap_or_else(|err| todo!("db error: {err}"));
    let path = request.uri().path();
    match *request.method() {
        Method::GET => {
            if path == "/" {
                index(&connection, request)
                    .await
                    .unwrap_or_else(|error| todo!("handle me: {error}"))
                    .response_ok()
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
                    "sets" => sets_view(&connection, request).await?.response_ok(),
                    _ => Err(ResourceError::NotFound(format!("/view/{path}"))),
                }
            } else if let Some(path) = path.strip_prefix("/sets/") {
                set(&connection, path).await?.response_ok()
            } else if let Some(path) = path.strip_prefix("/edit-set/") {
                edit_set(&connection, path).await?.response_ok()
            } else {
                Err(ResourceError::NotFound(path.to_string()))
            }
        }
        Method::POST => {
            if path == "/create-set" {
                create_set(&connection, request).await
            } else {
                Err(ResourceError::NotFound(path.to_string()))
            }
        }
        _ => Err(ResourceError::NotFound(path.to_string())),
    }
}

async fn create_set(
    connection: &libsql::Connection,
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, ResourceError> {
    let body = data::body_to_string(request).await?;
    dbg!(&body);
    let body = Query::from_str(&body);
    let title = body.get("title")?;
    let subject = body.get("subject")?;
    let id = data::generate_set_id(connection, &format!("{subject}/{title}"), 10).await?;
    let description = body.get("description")?;
    // TODO: handle full path
    let mut query = connection
        .query(
            "INSERT INTO cardset (id, title, description, subject) VALUES (?1, ?2, ?3, ?4) RETURNING id;",
            libsql::params![id, title.clone(), description.clone(), subject.clone()],
        )
        .await?;
    let path = query.next().await?.unwrap().get_str(0)?.to_string();
    let redirect_url = format!("/sets/{path}");
    let response = http::Response::builder()
        .header(http::header::CONTENT_TYPE, "text/html")
        .header(http::header::LOCATION, &redirect_url)
        .status(303)
        .body(Full::new(Bytes::from(format!(
            "Redirecting to <a href='{redirect_url}'>{redirect_url}</a>"
        ))))
        .unwrap();

    Ok(response)
}
struct App {
    database: libsql::Database,
}

impl App {
    async fn run(self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let listener = TcpListener::bind(addr).await?;
        let database = Arc::new(self.database);
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let database = Arc::clone(&database);
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(|request| router(&database, request)))
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
    let database = libsql::Builder::new_remote(
        "libsql://flashcards-charliec.turso.io".to_string(),
        secrets.get("DB_AUTH_TOKEN").expect("no auth token present"),
    )
    .build()
    .await
    .unwrap();

    Ok(App { database })
}
