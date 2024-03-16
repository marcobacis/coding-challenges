use std::fmt::Display;

use actix_web::{
    http::{header::ContentType, Error, StatusCode},
    web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, ResponseError,
};

#[derive(Debug)]
enum LBError {
    BackendError(reqwest::Error),
}

impl Display for LBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackendError(source) => f.write_fmt(format_args!("{:?}", source)),
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

struct LoadBalancer {
    port: u16,
    servers: Vec<String>,
}

impl LoadBalancer {
    pub fn new(port: u16, servers: Vec<String>) -> Self {
        Self { port, servers }
    }

    pub async fn run(&self) {
        HttpServer::new(|| App::new())
            .bind(("127.0.0.1", self.port))
            .unwrap()
            .run()
            .await;
    }

    pub fn uri(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    async fn handler(&self, req: HttpRequest) -> Result<HttpResponse, LBError> {
        println!("{}", req.uri());
        // TODO Get server from policy
        self.forward(req, self.servers.first().unwrap()).await
    }

    async fn forward(&self, req: HttpRequest, server: &str) -> Result<HttpResponse, LBError> {
        let uri = format!("{}{}", server, req.uri());

        let client = reqwest::Client::new();
        let request_builder = client
            .request(req.method().clone(), uri)
            .headers(req.headers().clone().into());
        //.body(payload.clone());

        let response = request_builder
            .send()
            .await
            .map_err(|err| LBError::BackendError(err))?;

        let mut response_builder = HttpResponse::build(response.status());
        for h in response.headers().iter() {
            response_builder.append_header(h);
        }
        let body = response
            .bytes()
            .await
            .map_err(|err| LBError::BackendError(err))?;

        Ok(response_builder.body(body))
    }
}

#[tokio::main]
async fn main() {
    LoadBalancer::new(8080, vec![String::from("http://127.0.0.1:8081")])
        .run()
        .await;
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

    use crate::LoadBalancer;

    #[tokio::test]
    async fn cose() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let server = LoadBalancer::new(8080, vec![mock_server.uri()]);
        let server_uri = server.uri();

        tokio::spawn(async move { server.run().await });

        let client = reqwest::Client::new();
        let response = client.get(server_uri).send().await.unwrap();

        assert_eq!(StatusCode::OK, response.status());
    }
}
