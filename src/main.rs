#![windows_subsystem = "windows"]

use std::{borrow::Cow, convert::Infallible, thread};

use hyper::{Request, Body, Response, StatusCode, Server, service::{service_fn, make_service_fn}};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static"]
struct Asset;

async fn request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = if req.uri().path() == "/" {
        "index.html"
    } else {
        &req.uri().path()[1..]
    };

    match Asset::get(path) {
        Some(content) => {
            let body: Body = match content.data {
              Cow::Borrowed(b) => b.into(),
              Cow::Owned(o) => o.into(),
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(body)
                .unwrap())
        }
        None => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 0).into();

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    let port = server.local_addr().port();

    thread::spawn(move || { async {
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }});

    web_view::builder()
        .title("Square Images")
        .content(web_view::Content::Url(format!("http://localhost:{}", port)))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();
}