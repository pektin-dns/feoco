use std::collections::HashMap;
use std::convert::Infallible;
use walkdir::WalkDir;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut fsmap: HashMap<String, Response<Body>> = HashMap::new();

    for entry in WalkDir::new("/public/") {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let path_str = path.to_str().unwrap();
            let file_content = std::fs::read(path_str).unwrap();
            fsmap.insert(
                String::from(path_str),
                Response::new(Body::from(file_content)),
            );
        }
    }

    let fsmap = fsmap;

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(|req| hello(req, fsmap))) }
    });

    let addr = ([0, 0, 0, 0], 80).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.with_graceful_shutdown(shutdown_signal()).await?;

    Ok(())
}

async fn hello(
    req: Request<Body>,
    fsmap: HashMap<String, Response<Body>>,
) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    let res;
    if fsmap.contains_key(path) {
        res = fsmap.get(path).unwrap().clone();
    } else {
        res = fsmap.get("index.html").unwrap().clone();
    }
    Ok(res.clone())
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
