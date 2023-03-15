use axum::{
    body::StreamBody,
    extract::Path,
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use mimalloc::MiMalloc;
use std::net::SocketAddr;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/sendfile/:req_filename", get(sendfile_api))
        .into_make_service();

    let addr = SocketAddr::from(([0, 0, 0, 0], 1370));
    println!("SSL disabled. Listening on {}", addr);
    Server::bind(&addr)
        .serve(router)
        .await
        .expect("Server startup failed.");
}

async fn sendfile_api(Path(req_filename): Path<String>) -> impl IntoResponse {
    let file = File::open(format!("./assets/{}", req_filename)).await.expect("file not found");
    let mut headers = HeaderMap::new();
    headers.append(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.append(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", req_filename)
            .parse()
            .unwrap(),
    );
    let reader_stream = ReaderStream::new(file);
    let body = StreamBody::new(reader_stream);
    (headers, body)
}
