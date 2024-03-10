use std::fmt::Display;

use actix_web::{
    http::{header::ContentType, Error, StatusCode},
    web, App, HttpRequest, HttpResponse, HttpServer, ResponseError,
};

#[derive(Debug)]
enum LBError {
    RedirectError(reqwest::Error),
}

impl Display for LBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RedirectError(source) => f.write_fmt(format_args!("{:?}", source)),
        }
    }
}

impl ResponseError for LBError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
}

async fn handler(req: HttpRequest, payload: web::Bytes) -> Result<HttpResponse, LBError> {
    println!("{}", req.uri());
    // TODO Get server
    let server = "http://127.0.0.1:8081";
    forward(req, payload, server).await
}

async fn forward(
    req: HttpRequest,
    payload: web::Bytes,
    server: &str,
) -> Result<HttpResponse, LBError> {
    let uri = format!("{}{}", server, req.uri());

    let client = reqwest::Client::new();
    let request_builder = client
        .request(req.method().clone(), uri)
        .headers(req.headers().clone().into())
        .body(payload.clone());

    let response = request_builder
        .send()
        .await
        .map_err(|err| LBError::RedirectError(err))?;

    let mut response_builder = HttpResponse::build(response.status());
    for h in response.headers().iter() {
        response_builder.append_header(h);
    }
    let body = response
        .bytes()
        .await
        .map_err(|err| LBError::RedirectError(err))?;

    Ok(response_builder.body(body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/{requested:.*}", web::get().to(handler)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
