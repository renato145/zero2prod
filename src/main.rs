use rocket::http::Status;

#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[get("/health_check")]
async fn health_check() -> Status {
    Status::Ok
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, health_check])
}
