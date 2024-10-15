use html_builder::prelude::*;
use http::Method;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use itertools::Itertools;
use sqlx::{Executor, MySqlPool};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

mod components;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subject {
    id: String,
    color: String,
}

impl Subject {
    fn name(&self) -> String {
        self.id.replace("-", " ")
    }

    fn icon_path(&self) -> String {
        format!("assets/{}.svg", self.id)
    }

    async fn fetch_from_id(id: &str) -> Self {
        todo!("query subject")
    }

    async fn fetch_all(pool: &MySqlPool) -> Vec<Self> {
        let rows = sqlx::query!("SELECT id, color FROM subject ORDER BY id")
            .fetch_all(pool)
            .await
            .unwrap_or_else(|error| todo!("db error: {error}"));
        // TODO: extract strings not Vec<u8>s
        rows.into_iter()
            .map(|record| Self {
                id: String::from_utf8(record.id).unwrap(),
                color: String::from_utf8(record.color).unwrap(),
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Set {
    pub name: String,
    pub id: u32,
    pub subject: Subject,
    pub size: usize,
}

impl Set {
    pub fn new(name: &str, id: u32, subject: Subject, size: usize) -> Self {
        Self {
            name: name.to_string(),
            id,
            subject,
            size,
        }
    }

    async fn fetch_all(pool: &MySqlPool, subject_id: &str) -> Vec<Self> {
        todo!("query all sets")
    }
}

fn subject_menu(subjects: Vec<Subject>) -> Menu {
    // input-red input-orange input-yellow input-emerald input-purple
    components::horizontal_btn_group(subjects.into_iter().map(|subject| {
        button(format!("subject-{}", subject.id))
            .class(format!("btn input-{} sound-mmm", subject.color))
            .hx_get(format!("/view/sets?subject={}", subject.id))
            .hx_target("#setlist")
            .hx_swap("outerHTML swap:200ms")
            .child(img(subject.icon_path(), subject.name()).size(24, 24))
            .child(p(subject.name()))
    }))
    .class("w-fit")
}

fn create_set_popup() -> Dialog {
    dialog()
        .id("create-set-dialog")
        .class("card w-3/4 inset-0 m-auto")
        .child(h2("Create Set"))
        .child(
            form(FormMethod::Post, "create_set")
                .class("w-full grid gap-4")
                .child(components::text_input(
                    "set-title",
                    "Set Title",
                    InputType::Text,
                ))
                .child(
                    components::button_with_icon("create-set", "create", "create set")
                        .class("input-accent"),
                ),
        )
}

fn set_list(sets: Vec<Set>) -> Section {
    section()
        .id("setlist")
        .class("grid grid-cols-3 w-full gap-4 fade-out")
        .children(
            sets.into_iter()
                .map(|set| {
                    article()
                        .class(format!("card w-full bg-{}-950", set.subject.color))
                        .child(h3(set.name))
                        .child(
                            div()
                                .class("w-full flex justify-between")
                                .child(p(format!("{} cards", set.size)))
                                .child(p(set.subject.name()).class(format!(
                                "rounded-full border border-black dark:border-white px-2 bg-{}-800",
                                set.subject.color
                            ))),
                        )
                        .child(
                            a(format!("/sets/{}", set.id))
                                .class(format!("btn input-{} w-full sound-yes", set.subject.color))
                                .child(img("/assets/study.svg", "study").size(24, 24))
                                .child(p("study")),
                        )
                })
                .collect_vec(),
        )
}

async fn index(request: Request<hyper::body::Incoming>, pool: &MySqlPool) -> Html {
    html("en")
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
                .child(subject_menu(Subject::fetch_all(pool).await))
                .child(set_list(Vec::new()).child(
                    p("click on a subject to view sets...").class("col-span-full text-center"),
                ))
                .child(components::fab_dropdown(
                    "create",
                    "create",
                    ["set", "folder"].into_iter().map(|item| {
                        components::button_with_icon(format!("create-{item}"), item, item).onclick(
                            format!(
                                "javascript:document.getElementById('create-{item}-dialog').open()"
                            ),
                        )
                    }),
                ))
                .child(create_set_popup())
                .script(include_str!("../script.js")),
        )
}

async fn sets_view(request: Request<hyper::body::Incoming>, pool: &MySqlPool) -> Section {
    let query: HashMap<String, String> = request
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_default();
    let subject = query
        .get("subject")
        .unwrap_or_else(|| todo!("400: malformed request"));
    set_list(Set::fetch_all(pool, subject).await)
}

async fn router(
    request: Request<hyper::body::Incoming>,
    pool: &MySqlPool,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = request.uri().path();
    match *request.method() {
        Method::GET => {
            if path == "/" {
                index(request, pool).await.response_ok()
            } else if let Some(asset) = path.strip_prefix("/assets/") {
                let bytes = fs::read(format!("/{}/assets/{asset}", env!("CARGO_MANIFEST_DIR")))
                    .unwrap_or_else(|_| todo!("404"));
                let content_type = match asset.split_once(".").expect("no mime type").1 {
                    "svg" => "image/svg+xml; charset=utf-8",
                    "jpg" | "jpeg" => "image/jpeg",
                    "min.js" | "js" => "text/javascript",
                    "mp3" => "audio/mpeg",
                    file_extension => todo!("handle '.{file_extension}' files"),
                };
                let response = http::Response::builder()
                    .header(http::header::CONTENT_TYPE, content_type)
                    .body(Full::new(Bytes::from(bytes)))
                    .unwrap();
                return Ok(response);
            } else if let Some(path) = path.strip_prefix("/view/") {
                match path {
                    "sets" => sets_view(request, pool).await.response_ok(),
                    _ => todo!("404"),
                }
            } else {
                todo!("404")
            }
        }
        _ => todo!("404"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = Arc::new(MySqlPool::connect(include_str!("../DATABASE_URL")).await?);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;
    println!("serving on http://localhost:3000");
    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);
        let pool = Arc::clone(&pool);
        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|request| router(request, &pool)))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
