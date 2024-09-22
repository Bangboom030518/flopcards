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

async fn index(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    html("en")
        .child(head().template().style(include_str!("./output.css")))
        .child(
            body()
                .child(h1("Welcome to Flopcards"))
                .child(section().child(h2("Money")).class("card w-full"))
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
