use maud::{html, Markup};

use super::layout;

pub fn get(errors: Option<String>) -> Markup {
    layout("Login", form(errors))
}

pub fn form(errors: Option<String>) -> Markup {
    html! {
        form action="/login" method="post" {
            @if let Some(errors) = errors {
               p { em style="color: red;" { (errors) } }
            }

            div style="display: flex; flex-direction: column; align-items: flex-start; gap: 1rem;" {
                label {
                    "Username "
                    input type="text" placeholder="Enter Username" name="username";
                }

                label {
                    "Password "
                    input type="password" placeholder="Enter Password" name="password";
                }

                button type="submit" { "Login" }
            }
        }
    }
}
