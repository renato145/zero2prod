use actix_web::HttpResponse;

pub async fn public_newsletter() -> HttpResponse {
    HttpResponse::Ok().finish()
}
