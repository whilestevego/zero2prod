use actix_web::{http::header::LOCATION, web, HttpResponse};
use secrecy::Secret;
use sqlx::PgPool;

use crate::authentication::{validate_credentials, Credentials};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(skip(form, db_pool), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
pub async fn login(form: web::Form<FormData>, db_pool: web::Data<&PgPool>) -> HttpResponse {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(&db_pool, credentials).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

            HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish()
        }
        Err(_) => {
            todo!()
        }
    }
}
