use html_builder::prelude::*;
use http::Method;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use itertools::Itertools;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::TcpListener;
use url::Url;

mod components;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subject {
    Maths,
    FurtherMaths,
    Spanish,
    Geography,
    Other,
}

impl Subject {
    const fn id(self) -> &'static str {
        match self {
            Self::Maths => "maths",
            Self::FurtherMaths => "further-maths",
            Self::Spanish => "spanish",
            Self::Geography => "geography",
            Self::Other => "other",
        }
    }

    fn name(self) -> String {
        self.id().replace("-", " ")
    }

    const fn icon_name(self) -> &'static str {
        match self {
            Self::Maths => "calculate",
            Self::FurtherMaths => "sigma",
            Self::Spanish => "translate",
            Self::Geography => "globe",
            Self::Other => "more",
        }
    }

    const fn color(self) -> &'static str {
        // bg-orange-800 bg-red-800 bg-yellow-800 bg-emerald-800 bg-purple-800
        // bg-orange-950 bg-red-950 bg-yellow-950 bg-emerald-950 bg-purple-950
        // input-orange input-red input-yellow input-emerald input-purple
        match self {
            Self::Maths => "orange",
            Self::FurtherMaths => "red",
            Self::Spanish => "yellow",
            Self::Geography => "emerald",
            Self::Other => "purple",
        }
    }

    fn icon_path(self) -> String {
        format!("assets/{}.svg", self.icon_name())
    }

    fn all() -> [Self; 5] {
        [
            Self::Maths,
            Self::FurtherMaths,
            Self::Spanish,
            Self::Geography,
            Self::Other,
        ]
    }
}

impl FromStr for Subject {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "maths" => Ok(Self::Maths),
            "further-maths" => Ok(Self::FurtherMaths),
            "spanish" => Ok(Self::Spanish),
            "geography" => Ok(Self::Geography),
            "other" => Ok(Self::Other),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Set {
    pub name: String,
    pub id: String,
    pub subject: Subject,
    pub size: usize,
}

impl Set {
    pub fn new(name: &str, id: &str, subject: Subject, size: usize) -> Self {
        Self {
            name: name.to_string(),
            id: id.to_string(),
            subject,
            size,
        }
    }

    #[deprecated = "use `new` instead"]
    pub fn new_gen_id(name: &str, subject: Subject) -> Self {
        Self {
            name: name.to_string(),
            id: name.to_string(),
            subject,
            size: 69,
        }
    }
}

fn subject_menu() -> Menu {
    components::horizontal_btn_group(Subject::all().map(|subject| {
        button(format!("subject-{}", subject.id()))
            .class(format!("btn input-{}", subject.color()))
            .hx_get(format!("/view/sets?subject={}", subject.id()))
            .hx_target("#setlist")
            .hx_swap("outerHTML swap:200ms")
            .child(img(subject.icon_path(), subject.name()).size(24, 24))
            .child(p(subject.name()))
    }))
    .class("w-fit")
}

#[deprecated = "data is example"]
fn example_sets(subject: Subject) -> Vec<Set> {
    vec![
        Set::new_gen_id("Set 1", subject),
        Set::new_gen_id("Set 2", subject),
        Set::new_gen_id("Set 3", subject),
        Set::new_gen_id("Set 4", subject),
        Set::new_gen_id("Set 5", subject),
        Set::new_gen_id("Set 6", subject),
    ]
}

fn set_list(sets: Vec<Set>) -> Section {
    section()
        .id("setlist")
        .class("grid grid-cols-3 w-full gap-4 fade-out")
        .children(
            sets.into_iter()
                .map(|set| {
                    article()
                        .class(format!("card w-full bg-{}-950", set.subject.color()))
                        .child(h3(set.name))
                        .child(
                            div()
                                .class("w-full flex justify-between")
                                .child(p(format!("{} cards", set.size)))
                                .child(p(set.subject.name()).class(format!(
                                "rounded-full border border-black dark:border-white px-2 bg-{}-800",
                                set.subject.color()
                            ))),
                        )
                        .child(
                            a(format!("/sets/{}", set.id))
                                .class("btn input-accent w-full")
                                .child(img("/assets/study.svg", "study").size(24, 24))
                                .child(p("study")),
                        )
                })
                .collect_vec(),
        )
}

fn index(request: Request<hyper::body::Incoming>) -> Html {
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
                .child(subject_menu())
                .child(set_list(Vec::new()))
                .child(components::fab_dropdown(
                    "create",
                    "create",
                    ["set", "folder"].into_iter().map(|item| {
                        components::button_with_icon(format!("create-{item}"), item, item)
                    }),
                )),
        )
}

fn sets_view(request: Request<hyper::body::Incoming>) -> Section {
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
        .unwrap_or_else(|| todo!("400: malformed request 3"))
        .parse::<Subject>()
        .unwrap_or_else(|_| todo!("400: malformed request 4"));
    set_list(example_sets(subject))
}

async fn router(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = request.uri().path();
    match *request.method() {
        Method::GET => {
            if path == "/" {
                index(request).response_ok()
            } else if let Some(asset) = path.strip_prefix("/assets/") {
                let bytes = fs::read(format!("/{}/assets/{asset}", env!("CARGO_MANIFEST_DIR")))
                    .unwrap_or_else(|_| todo!("404"));
                let content_type = match asset.split_once(".").expect("no mime type").1 {
                    "svg" => "image/svg+xml; charset=utf-8",
                    "jpg" | "jpeg" => "image/jpeg",
                    "min.js" | "js" => "text/javascript",
                    file_extension => todo!("handle '.{file_extension}' files"),
                };
                let response = http::Response::builder()
                    .header(http::header::CONTENT_TYPE, content_type)
                    .body(Full::new(Bytes::from(bytes)))
                    .unwrap();
                return Ok(response);
            } else if let Some(path) = path.strip_prefix("/view/") {
                match path {
                    "sets" => sets_view(request).response_ok(),
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

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(router))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
