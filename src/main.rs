use std::net::SocketAddr;

use axum::Router;
use axum_extra::routing::SpaRouter;

#[tokio::main]
async fn main() {
    let spa = SpaRouter::new("/assets", "assets");
    let router = Router::new()
        .merge(spa)
        .into_make_service();

    let addr = SocketAddr::from(([0, 0, 0, 0], 1370));
    println!("SSL disabled. Listening on {}", addr);
    axum_server::bind(addr)
        .serve(router)
        .await
        .expect("Server startup failed.");
}
