use chrono::Utc;
use rocket::{form::Form, http::Status, State};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(FromForm)]
pub struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions", data = "<form>")]
pub async fn subscribe(form: Form<FormData>, db: &State<PgPool>) -> Status {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&**db)
    .await
    {
        Ok(_) => Status::Ok,
        Err(e) => {
            println!("Failed to execute query: {}", e);
            Status::InternalServerError
        }
    }
}
