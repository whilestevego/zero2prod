use actix_web::web;
use maud::Markup;

use crate::views;

#[derive(serde::Deserialize, Debug)]
pub struct QueryParams {
    error: Option<String>,
}

pub async fn login_form(query: web::Query<QueryParams>) -> actix_web::Result<Markup> {
    Ok(views::login::get(query.0.error))
}
