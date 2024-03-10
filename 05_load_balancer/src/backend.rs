use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    println!("Received request on \"/\"");
    HttpResponse::Ok().body("Hello world!")
}

#[get("/health")]
async fn health() -> impl Responder {
    println!("Received healthcheck request");
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(health))
        .bind(("127.0.0.1", 8081))?
        .run()
        .await
}
