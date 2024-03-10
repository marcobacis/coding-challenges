use std::io::Error;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use clap::{Arg, Command};

async fn hello(req: HttpRequest, payload: web::Bytes) -> Result<HttpResponse, Error> {
    let uri = req.uri();
    println!("Received request on \"{}\"", uri);
    Ok(HttpResponse::Ok().body("Hello world!"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = Command::new("backend")
        .arg(
            Arg::new("PORT")
                .long("port")
                .short('p')
                .help("Port on which to listen")
                .value_parser(clap::value_parser!(u16)),
        )
        .get_matches();

    let port = matches.get_one::<u16>("PORT").unwrap_or(&8080);

    println!("Now listening on port {}", port);

    HttpServer::new(|| App::new().default_service(web::to(hello)))
        .bind(("127.0.0.1", *port))?
        .run()
        .await
}
