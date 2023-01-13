use maud::Markup;

use crate::views;

pub async fn login_form() -> actix_web::Result<Markup> {
    Ok(views::login::get())
}
