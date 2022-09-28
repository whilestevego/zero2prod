use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{types::Uuid, PgPool};
use tracing::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );

    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");

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
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!("request_id {request_id} - New subscriber details have been saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("request_id {request_id} - Failed to execute query: {e}");
            HttpResponse::InternalServerError().finish()
        }
    }
}
