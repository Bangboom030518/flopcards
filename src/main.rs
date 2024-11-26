// htmx-swapping
use data::{Query, ResourceError, Set};
use html_builder::prelude::*;
use http::Method;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::fmt::Display;
use std::fs;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod components;
mod data;

fn create_set_popup(subject: impl Display) -> Dialog {
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
                            "set title",
                            InputType::Text,
                        ))
                        .child(
                            input()
                                .r#type("hidden")
                                .name("subject")
                                .id("subject-input")
                                .value(subject),
                        )
                        .child(
                            components::button_with_icon("create-set", "create", "create set")
                                .class("input-accent"),
                        ),
                ),
        )
}

async fn index(request: Request<hyper::body::Incoming>) -> Html {
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
                .child(components::subject_menu())
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
                .child(create_set_popup(""))
                .child(components::loading_animation())
                .script(include_str!("../script.js")),
        )
}

async fn sets_view(request: Request<hyper::body::Incoming>) -> Result<Section, ResourceError> {
    let request = &request;
    let query = Query::from_request(request);
    let subject = query.get("subject")?;
    Set::fetch_all(&subject).await.map(components::set_list)
}

async fn router(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = request.uri().path();
    match *request.method() {
        Method::GET => {
            if path == "/" {
                index(request).await.response_ok()
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
                    "sets" => sets_view(request)
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
                Ok(create_set(request)
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
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, ResourceError> {
    let body = data::body_to_string(request).await?;
    let client = reqwest::Client::new();
    let path = client
        .post(format!(
            "{}?kind=set&{body}",
            include_str!("../DATABASE_URL")
        ))
        .header(http::header::CONTENT_LENGTH, 0)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
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
