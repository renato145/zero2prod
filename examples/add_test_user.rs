use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use sqlx::PgPool;
use uuid::Uuid;
use zero2prod::{configuration::get_configuration, get_connection_pool};

pub struct TestUser {
    user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn new(username: String, password: String) -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username,
            password,
        }
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();
        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash)
            Values ($1, $2, $3)",
            self.user_id,
            self.username,
            password_hash,
        )
        .execute(pool)
        .await
        .expect("Failed to create test user.");
    }
}

const ERROR_MSG: &str = "Invalid input, try: `add_test_user NEWUSER PASSWORD`";

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next();
    let username = args.next().expect(ERROR_MSG);
    let password = args.next().expect(ERROR_MSG);
    let user = TestUser::new(username, password);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let pool = get_connection_pool(&configuration.database);
    user.store(&pool).await;
}
