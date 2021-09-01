use chrono::Utc;
use rocket::{form::Form, http::Status, State};
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(FromForm)]
pub struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions", data = "<form>")]
pub async fn subscribe(form: Form<FormData>, db: &State<PgPool>) -> Status {
    let request_id = Uuid::new_v4();
    // With the % symbol we are telling tracing to use their Display implementation for logging purposes
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subcriber_email = %form.email,
        subcriber_name = %form.name
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subcriber details in the database.");

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
    .instrument(query_span)
    .await
    {
        Ok(_) => Status::Ok,
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            Status::InternalServerError
        }
    }
}
