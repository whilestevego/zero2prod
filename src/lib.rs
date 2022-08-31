use std::io;

use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse, dev::Server};

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

pub fn run() -> Result<Server, io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
    })
    .bind("127.0.0.1:8000")?
    .run();

    Ok(server)
}

// Page 28