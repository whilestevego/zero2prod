use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

use self::send_grid::{Content, MIMEType, Personalization, Recipient, SendEmailRequestBody};

#[derive(Clone, Debug)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
    ) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token,
        }
    }

    // curl --request POST \
    // --url https://api.sendgrid.com/v3/mail/send \
    // --header "Authorization: Bearer $SENDGRID_API_KEY" \
    // --header 'Content-Type: application/json' \
    // --data '{"personalizations": [{"to": [{"email": "test@example.com"}]}],"from": {"email": "test@example.com"},"subject": "Sending with SendGrid is Fun","content": [{"type": "text/plain", "value": "and easy to do anywhere, even with cURL"}]}'
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/mail/send", &self.base_url);

        let from_recipient = Recipient {
            name: "",
            email: self.sender.as_ref(),
        };

        let reply_to_recipient = Recipient {
            name: "",
            email: self.sender.as_ref(),
        };

        let to_recipient = Recipient {
            name: "",
            email: recipient.as_ref(),
        };

        let request_body = SendEmailRequestBody {
            from: from_recipient,
            reply_to: reply_to_recipient,
            subject,
            content: &vec![
                Content {
                    type_: MIMEType::TextHTML,
                    value: html_content,
                },
                Content {
                    type_: MIMEType::TextPlain,
                    value: text_content,
                },
            ],
            personalizations: &vec![Personalization {
                to: vec![to_recipient],
            }],
        };

        let builder = self
            .http_client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.authorization_token.expose_secret()),
            )
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }
}

mod send_grid {
    #[derive(serde::Serialize, Debug)]
    pub struct SendEmailRequestBody<'a> {
        pub from: Recipient<'a>,
        pub reply_to: Recipient<'a>,
        pub subject: &'a str,
        pub content: &'a Vec<Content<'a>>,
        pub personalizations: &'a Vec<Personalization<'a>>,
    }

    #[derive(serde::Serialize, Debug)]
    pub struct Recipient<'a> {
        pub email: &'a str,
        pub name: &'a str,
    }

    #[derive(serde::Serialize, Debug)]
    pub enum MIMEType {
        #[serde(rename = "text/plain")]
        TextPlain,
        #[serde(rename = "text/html")]
        TextHTML,
    }

    #[derive(serde::Serialize, Debug)]
    pub struct Content<'a> {
        #[serde(rename = "type")]
        pub type_: MIMEType,
        pub value: &'a str,
    }

    #[derive(serde::Serialize, Debug)]
    pub struct Personalization<'a> {
        pub to: Vec<Recipient<'a>>,
    }
}

#[cfg(test)]
mod tests {
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::{en::Paragraph, en::Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::domain::SubscriberEmail;

    use super::EmailClient;

    struct SendEmailRequestBodyMatcher;

    impl wiremock::Match for SendEmailRequestBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                vec!["from", "personalizations", "subject", "content"]
                    .iter()
                    .all(|x| body.get(x).is_some())
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/mail/send"))
            .and(method("POST"))
            .and(SendEmailRequestBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();

        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
