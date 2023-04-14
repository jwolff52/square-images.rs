#![windows_subsystem = "windows"]

#[derive(RustEmbeded)]
#[folder = "static"]
struct Asset;

fn request(req: Request<Body>) -> Response<Body> {
    let path = if req.uri().path() == "/" {
        "index.html"
    } else {
        &req.uri().path()[1..]
    };

    match Asset::get(path) {
        Some(content) => {
            let body: Body = match content {
              Cow::Borrowed(b) => b.into(),
              Cow::Owned(o) => o.into(),
            };

            Response::builder()
                .status(StatusCode::OK)
                .body(body)
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap(),
    }
}