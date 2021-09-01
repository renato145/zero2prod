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
    let request_id = Uuid::new_v4();
    log::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber.",
        request_id,
        form.email,
        form.name
    );
    log::info!(
        "request_id {} - Saving new subcriber details in the database.",
        request_id
    );
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
        Ok(_) => {
            log::info!(
                "request_id {} - New subscriber details have been saved.",
                request_id
            );
            Status::Ok
        }
        Err(e) => {
            log::error!(
                "request_id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            Status::InternalServerError
        }
    }
}
