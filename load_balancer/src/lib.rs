use std::{fmt::Display, sync::Arc};

use actix_web::{
    http::{header::ContentType, StatusCode},
    web, App, HttpRequest, HttpResponse, HttpServer, ResponseError,
};
use policies::RoutingPolicy;
use reqwest::Client;

pub mod policies;

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

#[derive(Clone)]
pub struct Backend {
    pub url: String,
    pub health_url: String,
}

pub struct LoadBalancer<P>
where
    P: RoutingPolicy + 'static,
{
    port: u16,
    servers: Vec<Backend>,
    policy: Arc<P>,
}

#[actix_web::get("/health")]
async fn healthcheck() -> &'static str {
    "Ok"
}

impl<P: RoutingPolicy + Send + Sync> LoadBalancer<P> {
    pub fn new(port: u16, servers: Vec<Backend>, policy: Arc<P>) -> Self {
        Self {
            port,
            servers,
            policy,
        }
    }

    pub async fn run(&self) {
        let app_data = web::Data::new(AppState {
            servers: self.servers.clone(),
            policy: self.policy.clone(),
            client: Client::new(),
        });

        HttpServer::new(move || {
            App::new()
                .app_data(app_data.clone())
                .service(healthcheck)
                .default_service(web::to(handler::<P>))
        })
        .bind(("127.0.0.1", self.port))
        .unwrap()
        .run()
        .await
        .unwrap();
    }

    fn uri(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

struct AppState<P: RoutingPolicy> {
    servers: Vec<Backend>,
    policy: Arc<P>,
    client: Client,
}

async fn handler<P>(
    req: HttpRequest,
    data: web::Data<AppState<P>>,
    bytes: web::Bytes,
) -> Result<HttpResponse, LBError>
where
    P: RoutingPolicy,
{
    let server = data.policy.next(&req, &data.servers);
    let uri = format!("{}{}", server, req.uri());

    let request_builder = data
        .client
        .request(req.method().clone(), uri)
        .headers(req.headers().into())
        .body(bytes);

    let response = request_builder
        .send()
        .await
        .map_err(LBError::BackendError)?;

    let mut response_builder = HttpResponse::build(response.status());
    for h in response.headers().iter() {
        response_builder.append_header(h);
    }
    let body = response.bytes().await.map_err(LBError::BackendError)?;

    Ok(response_builder.body(body))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::policies::RoundRobinPolicy;

    use super::*;
    use reqwest::{ClientBuilder, StatusCode};
    use wiremock::{
        matchers::{body_json_string, method},
        Mock, MockServer, ResponseTemplate,
    };

    async fn wait_server_up(client: &Client, uri: &str, max_retries: usize) {
        let health_uri = format!("{}/health", uri);
        for _ in 0..max_retries {
            let response = client.get(&health_uri).send().await;
            if response.is_ok() {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        panic!("Server didn't start...");
    }

    #[tokio::test]
    async fn test_get_root() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let policy = Arc::new(RoundRobinPolicy::new());
        let server = LoadBalancer::new(
            8080,
            vec![Backend {
                url: mock_server.uri(),
                health_url: format!("{}/health", mock_server.uri()),
            }],
            policy,
        );
        let server_uri = server.uri();
        tokio::spawn(async move { server.run().await });

        wait_server_up(&client, &server_uri, 3).await;

        let response = client.get(server_uri).send().await.unwrap();
        assert_eq!(StatusCode::OK, response.status());
    }

    #[tokio::test]
    async fn test_post_root() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(body_json_string("{}"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();

        let policy = Arc::new(RoundRobinPolicy::new());

        let server = LoadBalancer::new(
            8081,
            vec![Backend {
                url: mock_server.uri(),
                health_url: format!("{}/health", mock_server.uri()),
            }],
            policy,
        );
        let server_uri = server.uri();
        tokio::spawn(async move { server.run().await });

        wait_server_up(&client, &server_uri, 3).await;

        let response = client.post(server_uri).body("{}").send().await.unwrap();
        assert_eq!(StatusCode::OK, response.status());
    }

    #[tokio::test]
    async fn test_round_robin_two_servers() {
        let mocks = vec![
            MockServer::start().await,
            MockServer::start().await,
            MockServer::start().await,
        ];

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("1"))
            .expect(2)
            .mount(&mocks[0])
            .await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("2"))
            .expect(1)
            .mount(&mocks[1])
            .await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("3"))
            .expect(1)
            .mount(&mocks[2])
            .await;

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();

        let policy = Arc::new(RoundRobinPolicy::new());

        // Spawn server
        let server = LoadBalancer::new(
            8082,
            mocks
                .iter()
                .map(|mock| Backend {
                    url: mock.uri(),
                    health_url: format!("{}/health", mock.uri()),
                })
                .collect(),
            policy,
        );
        let server_uri = server.uri();
        tokio::spawn(async move { server.run().await });
        wait_server_up(&client, &server_uri, 3).await;

        // Send requests, expect to respond in round robin
        let response = client.get(&server_uri).send().await.unwrap();
        assert_eq!(StatusCode::OK, response.status());
        assert_eq!("1", response.text().await.unwrap());

        let response = client.get(&server_uri).send().await.unwrap();
        assert_eq!(StatusCode::OK, response.status());
        assert_eq!("2", response.text().await.unwrap());

        let response = client.get(&server_uri).send().await.unwrap();
        assert_eq!(StatusCode::OK, response.status());
        assert_eq!("3", response.text().await.unwrap());

        let response = client.get(&server_uri).send().await.unwrap();
        assert_eq!(StatusCode::OK, response.status());
        assert_eq!("1", response.text().await.unwrap());
    }
}
