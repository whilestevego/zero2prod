use argon2::{password_hash::SaltString, Argon2, Params, PasswordHasher};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TestUser {
    pub id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            id: Uuid::now_v7(),
            username: Uuid::now_v7().to_string(),
            password: Uuid::now_v7().to_string(),
        }
    }

    pub async fn insert(&self, db_pool: &PgPool) -> String {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

        sqlx::query!(
            r#"
                INSERT INTO users (id, username, password_hash)
                VALUES ($1, $2, $3)
                RETURNING username, password_hash
            "#,
            self.id,
            self.username,
            password_hash,
        )
        .fetch_one(db_pool)
        .await
        .map(|row| row.password_hash)
        .expect("Failed to get or create test user.")
    }
}
