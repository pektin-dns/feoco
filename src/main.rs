use hashbrown::HashMap;
use hyper::{
    header::{HeaderName, HeaderValue},
    HeaderMap,
};
use rust_web_server::read_to_memory;
use std::convert::Infallible;
use std::str::FromStr;

use hyper::service::{make_service_fn, service_fn};
use lazy_static::lazy_static;

use hyper::{Body, Request, Response, Server};

mod config;
use crate::config::{read_config, Config};
mod lib;
use crate::lib::COMPRESSABLE_MIME_TYPES;

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
