use zero2prod::get_rocket;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    get_rocket(None)
}
