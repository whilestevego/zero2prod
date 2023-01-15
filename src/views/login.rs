use maud::{html, Markup};

use super::layout;

pub fn get() -> Markup {
    layout("Login", form())
}

pub fn post() -> Markup {
    layout("Login", form())
}

pub fn form() -> Markup {
    html! {
        form action="/login" method="post" {
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
