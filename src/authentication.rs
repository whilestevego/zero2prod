use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;

use crate::telemetry::spawn_blocking_with_tracing;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

#[tracing::instrument(name = "Validate credentials", skip(db_pool, credentials))]
pub async fn validate_credentials(
    db_pool: &PgPool,
    credentials: Credentials,
) -> Result<Uuid, AuthError> {
    let mut id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .into(),
    );

    if let Some((stored_id, store_password_hash)) =
        get_stored_credentials(db_pool, &credentials.username).await?
    {
        id = Some(stored_id);
        expected_password_hash = store_password_hash
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task.")??;

    id.ok_or_else(|| AuthError::InvalidCredentials(anyhow::anyhow!("Unknown username.")))
}

#[tracing::instrument(name = "Get stored credentials", skip(db_pool, username))]
async fn get_stored_credentials(
    db_pool: &PgPool,
    username: &str,
) -> Result<Option<(Uuid, Secret<String>)>, anyhow::Error> {
    let row: Option<_> = sqlx::query!(
        r#"
            SELECT id, password_hash
            FROM users
            WHERE username = $1
        "#,
        username,
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to perform a query to validate auth credentials.")?
    .map(|row| (row.id, Secret::new(row.password_hash)));

    Ok(row)
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
pub fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), AuthError> {
    let expected_password_hash = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")?;

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .context("Invalid password.")
        .map_err(AuthError::InvalidCredentials)
}
