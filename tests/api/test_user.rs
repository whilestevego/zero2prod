use sha3::{Digest, Sha3_256};
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
        let password_hash = Sha3_256::digest(self.password.as_bytes());
        let password_hash = format!("{:x}", password_hash);

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
