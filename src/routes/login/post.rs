use actix_web::{http::header::LOCATION, web, HttpResponse};
use secrecy::Secret;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    paswword: Secret<String>,
}

pub async fn login(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish()
}
