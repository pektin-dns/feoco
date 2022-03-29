use flate2::Compression;
use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Write;
use walkdir::WalkDir;

use flate2::write::GzEncoder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut fsmap: HashMap<String, Vec<u8>> = HashMap::new();

    for entry in WalkDir::new("/public/") {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            println!("{:?}", path);
            let path_str = path.to_str().unwrap();
            let file_content = std::fs::read(path_str).unwrap();

            let mut z = GzEncoder::new(Vec::new(), Compression::best());
            z.write_all(file_content.as_slice()).unwrap();

            let file_content_gz = z.finish().unwrap();

            println!("{:?},{:?}", file_content.len(), file_content_gz.len());

            fsmap.insert(
                String::from(path_str).replace("/public", ""),
                file_content_gz,
            );
        }
    }

    let fsmap = fsmap;
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        let fsmap = fsmap.clone();
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(move |req| hello(req, fsmap.to_owned()))) }
    });

    let addr = ([0, 0, 0, 0], 80).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.with_graceful_shutdown(shutdown_signal()).await?;

    Ok(())
}

async fn hello(
    req: Request<Body>,
    fsmap: HashMap<String, Vec<u8>>,
) -> Result<Response<Body>, Infallible> {
    let p = req.uri().path();
    let path = if p == "/" { "/index.html" } else { p };

    let res = Response::builder()
        .header("content-encoding", "gzip")
        .status(200)
        .body(if fsmap.contains_key(path) {
            Body::from(fsmap.get(path).unwrap().clone())
        } else {
            Body::from(fsmap.get("/index.html").unwrap().clone())
        })
        .unwrap();

    Ok(res)
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
