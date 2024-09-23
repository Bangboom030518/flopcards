use components::InputStyle;
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
use tokio::net::TcpListener;

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
    fn id(self) -> String {
        let id = match self {
            Self::Maths => "maths",
            Self::FurtherMaths => "further-maths",
            Self::Spanish => "spanish",
            Self::Geography => "geography",
            Self::Other => "other",
        };
        format!("subject-{id}")
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Maths => "Maths",
            Self::FurtherMaths => "Further Maths",
            Self::Spanish => "Spanish",
            Self::Geography => "Geography",
            Self::Other => "Other",
        }
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

#[derive(Clone, Debug)]
pub struct Set {
    pub name: String,
    pub id: String,
    pub subject: Subject,
}

impl Set {
    pub fn new(name: &str, id: &str, subject: Subject) -> Self {
        Self {
            name: name.to_string(),
            id: id.to_string(),
            subject,
        }
    }

    #[deprecated = "use `new` instead"]
    pub fn new_gen_id(name: &str, subject: Subject) -> Self {
        Self {
            name: name.to_string(),
            id: name.to_string(),
            subject,
        }
    }
}

fn subject_menu() -> Menu {
    components::btn_group(
        std::iter::once(
            button("btn-subject-all")
                .class("btn input-accent")
                .child(img("assets/star.svg", "All").size(24, 24))
                .child(p("All")),
        )
        .chain(Subject::all().map(|subject| {
            button(subject.id())
                .class("btn input-gray")
                .child(img(subject.icon_path(), subject.name()).size(24, 24))
                .child(p(subject.name()))
        })),
    )
    .class("w-fit")
}

#[deprecated = "data is example"]
fn example_sets() -> Vec<Set> {
    vec![
        Set::new_gen_id("Vocab 1", Subject::Spanish),
        Set::new_gen_id("Vocab 2", Subject::Spanish),
        Set::new_gen_id("Vocab 3", Subject::Spanish),
        Set::new_gen_id("Vocab 4", Subject::Spanish),
        Set::new_gen_id("Vocab 5", Subject::Spanish),
        Set::new_gen_id("Vocab 6", Subject::Spanish),
    ]
}

fn set_list(sets: Vec<Set>) -> Section {
    section()
        .class("grid grid-flow-row w-full gap-4")
        .children(sets.into_iter().map(|set| {
            article()
                .class("card w-full grid-flow-col")
                .child(h3(set.name))
                .child(p(set.subject.name()))
                .child(
                    a(format!("/sets/{}", set.id))
                        .class("btn input-accent")
                        .text("View Set"),
                )
        }))
}

async fn index(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    html("en")
        .child(head().template().style(include_str!("./output.css")).title("Flopcards - Home").raw_text("<script src='assets/htmx.min.js'></script>"))
        .child(
            body()
                .class("p-8 grid place-items-center items-start gap-8 bg-white text-black dark:bg-gray-950 dark:text-white")
                .child(h1("Flopcards"))
                .child(subject_menu())
                .child(set_list(example_sets()))
                .child(components::fab("create", "create")),
        )
        .response_ok()
}

async fn router(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = request.uri().path();
    match *request.method() {
        Method::GET => {
            if path == "/" {
                index(request).await
            } else if let Some(asset) = path.strip_prefix("/assets/") {
                let bytes =
                    fs::read(format!("/{}/assets/{asset}", env!("CARGO_MANIFEST_DIR"))).unwrap();
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
