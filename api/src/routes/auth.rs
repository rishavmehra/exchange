use actix_web::{get, HttpResponse, Responder};

#[get("/sign_in")]
pub async fn sign_in() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}

#[get("/sign_up")]
pub async fn sign_up() -> impl Responder {
    HttpResponse::Ok().body("Hello, world!")
}