use flate2::Compression;
use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap,
};
use std::convert::Infallible;
use std::io::Write;
use std::{collections::HashMap, str::FromStr};
use walkdir::WalkDir;

use flate2::write::GzEncoder;
use hyper::service::{make_service_fn, service_fn};
use lazy_static::lazy_static;

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

lazy_static! {
    static ref CONFIG: Config = read_config();
}
lazy_static! {
    static ref PAGES: HashMap<String, Vec<u8>> = read_to_memory();
}

lazy_static! {
    static ref DOCUMENT_MAP: HeaderMap = create_header_map(HeaderMapType::Document);
}

lazy_static! {
    static ref ALL_MAP: HeaderMap = create_header_map(HeaderMapType::All);
}

pub enum HeaderMapType {
    Document,
    All,
}

pub fn create_header_map(map_type: HeaderMapType) -> HeaderMap<HeaderValue> {
    let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
    let config = &CONFIG;
    if matches!(map_type, HeaderMapType::Document) {
        for header in &config.headers.document {
            headers.insert(
                HeaderName::from_str(header.0).unwrap(),
                HeaderValue::from_str(header.1).unwrap(),
            );
        }
    }
    for header in &config.headers.all {
        headers.insert(
            HeaderName::from_str(header.0).unwrap(),
            HeaderValue::from_str(header.1).unwrap(),
        );
    }

    headers
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = &PAGES.contains_key("/");
    let _ = &CONFIG.clone();
    let _ = &DOCUMENT_MAP.clone();
    let _ = &ALL_MAP.clone();

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let addr = ([0, 0, 0, 0], 80).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);
    //.with_graceful_shutdown(shutdown_signal())
    server.await?;

    Ok(())
}

fn read_to_memory() -> HashMap<String, Vec<u8>> {
    let mut fsmap: HashMap<String, Vec<u8>> = HashMap::new();

    let mut file_content_size: u128 = 0;
    let mut file_content_size_compressed: u128 = 0;

    for entry in WalkDir::new(BASE_PATH) {
        let entry = entry.unwrap();
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
        "In memory size: {} MiB\nIn memory size compressed: {} MiB\nTotal memory size: {} MiB",
        file_content_size / 1024 / 1024,
        file_content_size_compressed / 1024 / 1024,
        (file_content_size + file_content_size_compressed) / 1024 / 1024
    );

    fsmap
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let fsmap = &PAGES;

    let mut path = req.uri().path();

    let request_headers = req.headers();
    let accept_gzip = request_headers
        .get("accept-encoding")
        .unwrap_or(&HeaderValue::from_static(""))
        .to_str()
        .unwrap()
        .contains("gzip");

    let mut res = Response::builder().status(200);

    if !fsmap.contains_key(path) {
        path = "/index.html";
    };

    let content_type = mime_guess::from_path(path).first_or_octet_stream();

    if content_type == "text/html" {
        res.headers_mut().unwrap().extend(DOCUMENT_MAP.clone());
    } else {
        res.headers_mut().unwrap().extend(ALL_MAP.clone());
    }

    res = res.header("content-type", content_type.as_ref());
    let access_path = if accept_gzip && COMPRESSABLE_MIME_TYPES.contains(&content_type.as_ref()) {
        res = res.header("content-encoding", "gzip");
        format!("{}_gz", path)
    } else {
        String::from(path)
    };

    let res = res
        .body(Body::from(fsmap.get(&access_path).unwrap().clone()))
        .unwrap();

    Ok(res)
}
/*
async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
*/
