use axum::{
    extract::Path,
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use mimalloc::MiMalloc;
use std::net::SocketAddr;
use tokio_uring::fs::File;

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
    tokio::task::block_in_place(move || {
        tokio_uring::start(async {
            let file = File::open(format!("./assets/{}", req_filename))
                .await
                .expect("file not found");
            let buffer = vec![0; 104857600];
            let (_, buf) = file.read_at(buffer, 0).await;

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
            (headers, buf)
        });
    });
}
