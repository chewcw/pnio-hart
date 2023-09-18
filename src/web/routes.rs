use actix_web::{get, post, web, HttpResponse};

#[get("/lookup/{device_name}")]
pub async fn lookup(device_name: web::Path<String>) -> HttpResponse {
    todo!()
}

#[post("/command")]
pub async fn send(command: web::Path<u16>) -> HttpResponse {
    todo!()
}
