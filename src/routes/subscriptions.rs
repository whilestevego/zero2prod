use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{types::Uuid, PgPool};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();

    log::info!(
        "request_id {request_id} - Adding '{}' '{}' as a new subscriber.",
        form.email,
        form.name
    );

    match sqlx::query!(
        r#"
            INSERT INTO subscriptions (email, name, subscribed_at)
            VALUES($1, $2, $3)
        "#,
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            log::info!("request_id {request_id} - New subscriber details have been saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!("request_id {request_id} - Failed to execute query: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
