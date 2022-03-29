use flate2::Compression;
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::io::Write;
use walkdir::WalkDir;

use flate2::write::GzEncoder;
use hyper::service::{make_service_fn, service_fn};

use hyper::{Body, Request, Response, Server};

mod config;

use crate::config::{read_config, Config};

const COMPRESSABLE_MIME_TYPES: [&str; 15] = [
    "text/css",
    "application/javascript",
    "text/html",
    "image/svg+xml",
    "text/xml",
    "text/plain",
    "application/json",
    "application/yaml",
    "application/yml",
    "application/toml",
    "text/markdown",
    "application/wasm",
    "application/json-p",
    "text/javascript",
    "text/css",
];

const BASE_PATH: &str = "/public";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut fsmap: HashMap<String, Vec<u8>> = HashMap::new();

    let mut file_content_size: u128 = 0;
    let mut file_content_size_compressed: u128 = 0;

    let config = read_config();
    println!("{:?}", config);
    for argument in env::args() {
        println!("{}", argument);
    }

    for entry in WalkDir::new(BASE_PATH) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let path_str = path.to_str().unwrap();
            let file_content = std::fs::read(path_str).unwrap();

            file_content_size += file_content.len() as u128;
            if COMPRESSABLE_MIME_TYPES.contains(
                &mime_guess::from_path(path_str)
                    .first_or_octet_stream()
                    .as_ref(),
            ) {
                println!("{:?}", path_str);

                let mut z = GzEncoder::new(Vec::new(), Compression::best());
                z.write_all(file_content.as_slice()).unwrap();

                let file_content_gz = z.finish().unwrap();
                file_content_size_compressed += file_content_gz.len() as u128;

                fsmap.insert(
                    format!("{}_gz", String::from(path_str).replace(BASE_PATH, "")),
                    file_content_gz,
                );
            }
            fsmap.insert(String::from(path_str).replace(BASE_PATH, ""), file_content);
        }
    }

    println!(
        "In memory size: {} MiB\nIn memory size compressed: {} MiB\nTotal size: {} MiB",
        file_content_size / 1024 / 1024,
        file_content_size_compressed / 1024 / 1024,
        (file_content_size + file_content_size_compressed) / 1024 / 1024
    );

    let make_svc = make_service_fn(|_conn| {
        let fsmap = fsmap.clone();
        let config = config.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, fsmap.clone(), config.clone())
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 80).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.with_graceful_shutdown(shutdown_signal()).await?;

    Ok(())
}

async fn handle_request(
    req: Request<Body>,
    fsmap: HashMap<String, Vec<u8>>,
    config: Config,
) -> Result<Response<Body>, Infallible> {
    let mut path = req.uri().path();
    let request_headers = req.headers();
    let accept_gzip = request_headers
        .get("accept-encoding")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("gzip");

    let mut res = Response::builder().status(200);

    if !fsmap.contains_key(path) {
        path = "/index.html";
    };

    let access_path = if accept_gzip
        && COMPRESSABLE_MIME_TYPES
            .contains(&mime_guess::from_path(path).first_or_octet_stream().as_ref())
    {
        res = res.header("content-encoding", "gzip");
        format!("{}_gz", path)
    } else {
        String::from(path)
    };
    /*
        for header in config.headers.iter() {
            res = res.header(header.0, header.1);
        }
    */
    println!("{}", access_path);

    let res = res
        .body(Body::from(fsmap.get(&access_path).unwrap().clone()))
        .unwrap();

    Ok(res)
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
