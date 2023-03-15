use actix_files::Files;
use actix_web::{middleware::Logger, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("starting HTTP server at http://localhost:1370");

    HttpServer::new(|| {
        App::new()
            .service(Files::new("/sendfile", "./assets/"))
            // Enable the logger.
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 1370))?
    .run()
    .await
}
