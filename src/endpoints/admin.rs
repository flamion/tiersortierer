use actix_web::{Responder, web, get};

#[get("/test/1/{name}")]
pub async fn admin_endpoint_test(path: web::Path<String>) -> Result<impl Responder, Box<dyn std::error::Error>> {
	let name = path.into_inner();
	Ok(format!("Hi {}", name))
}