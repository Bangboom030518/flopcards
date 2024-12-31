// htmx-swapping
use data::{Query, ResourceError, Set, Subject};
use html_builder::prelude::*;
use http::Method;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
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
                .child(components::set_list(Vec::new()).child(
                    p("click on a subject to view sets...").class("col-span-full text-center"),
                ))
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
        Set::fetch_all(connection, &subject).await?,
    ))
}

async fn router(
    connection: &libsql::Connection,
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = request.uri().path();
    match *request.method() {
        Method::GET => {
            if path == "/" {
                index(connection, request)
                    .await
                    .unwrap_or_else(|error| todo!("handle me: {error}"))
                    .response_ok()
            } else if path == "/favicon.ico" {
                let path = format!("/{}/assets/favicon.ico", env!("CARGO_MANIFEST_DIR"));
                let bytes = fs::read(path).unwrap_or_else(|_| todo!("404"));
                let response = http::Response::builder()
                    .header(http::header::CONTENT_TYPE, "image/x-icon")
                    .body(Full::new(Bytes::from(bytes)))
                    .unwrap();
                return Ok(response);
            } else if let Some(asset) = path.strip_prefix("/assets/") {
                let bytes = fs::read(format!("/{}/assets/{asset}", env!("CARGO_MANIFEST_DIR")))
                    .unwrap_or_else(|_| todo!("404"));
                let content_type = match asset.split_once(".").expect("no mime type").1 {
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
                    "sets" => sets_view(connection, request)
                        .await
                        .unwrap_or_else(|err| todo!("handle me: {err:?}"))
                        .response_ok(),
                    _ => todo!("404"),
                }
            } else {
                todo!("404")
            }
        }
        Method::POST => {
            if path == "/create-set" {
                Ok(create_set(connection, request)
                    .await
                    .unwrap_or_else(|err| todo!("handle me: {err}")))
            } else {
                todo!("404")
            }
        }
        _ => todo!("404"),
    }
}

async fn create_set(
    connection: &libsql::Connection,
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, ResourceError> {
    let body = data::body_to_string(request).await?;
    dbg!(&body);
    let body = data::parse_query(&body);
    let title = body.get("title").unwrap_or_else(|| todo!("missing param"));
    let description = body
        .get("description")
        .unwrap_or_else(|| todo!("missing param"));
    let subject = body
        .get("subject")
        .unwrap_or_else(|| todo!("missing param"));
    // TODO: handle full path
    let mut query = connection
        .query(
            "INSERT INTO cardset (title, description, subject) VALUES (?1, ?2, ?3) RETURNING id;",
            libsql::params![title.clone(), description.clone(), subject.clone()],
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
    connection: libsql::Connection,
}

impl App {
    async fn run(self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let listener = TcpListener::bind(addr).await?;
        let connection = Arc::new(self.connection);
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let connection = Arc::clone(&connection);
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(|request| router(&connection, request)))
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
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
    let connection = database.connect().expect("failed to connect to database");
    Ok(App { connection })
}
