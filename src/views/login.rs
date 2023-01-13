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
        label {
            "Username "
            input type="text" placeholder="Enter Username" name="username";
        }

        br;

        label {
            "Password "
            input type="password" placeholder="Enter Password" name="password";
        }

        button type="submit" { "Login" }
    }
}
