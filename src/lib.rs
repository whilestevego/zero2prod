use std::{io, net::TcpListener};

use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse, dev::Server};

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> Result<Server, io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

// Page 28