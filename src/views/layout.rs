use maud::{html, Markup};

pub fn layout(title: &str, content: Markup) -> Markup {
    html! {
        html lang="en" {
            head {
                meta http-equiv="content-type" content="text/html; charset=utf-8";
                title { (title) }
            }

            body {
                (content)
            }
        }
    }
}
