use std::{fmt::Display, sync::Arc};

use actix_web::{
    http::{header::ContentType, StatusCode},
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, ResponseError,
};
use tokio::sync::mpsc::channel;

use policies::RoutingPolicy;
use reqwest::Client;
use tokio::sync::mpsc::{Receiver, Sender};

mod health;
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

#[derive(Clone, Debug)]
pub struct Backend {
    pub url: String,
    pub health_url: String,
}

pub type Config = Vec<Backend>;

pub struct LoadBalancer<P>
where
    P: RoutingPolicy + 'static,
{
    port: u16,
    data: Arc<AppState<P>>,
    config: Config,
}

#[actix_web::get("/health")]
async fn healthcheck() -> &'static str {
    "Ok"
}

impl<P: RoutingPolicy + Send + Sync> LoadBalancer<P> {
    pub fn new(port: u16, config: Config, policy: Arc<P>) -> Self {
        Self {
            port,
            data: Arc::new(AppState {
                policy: policy.clone(),
                client: Client::new(),
            }),
            config,
        }
    }

    pub async fn run(&self) {
        let app_data = Data::from(self.data.clone());

        let (tx, mut rx): (Sender<Vec<Backend>>, Receiver<Vec<Backend>>) = channel(32);

        // Start health check task
        let config_clone = self.config.clone();
        tokio::spawn(async move {
            health::health_thread(config_clone, &tx).await;
        });

        // Halth check receiver
        let data = self.data.clone();
        tokio::spawn(async move {
            while let Some(healthy_backends) = rx.recv().await {
                data.policy.health_results(healthy_backends).await;
            }
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

    pub fn uri(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

struct AppState<P: RoutingPolicy> {
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
    let server = data.policy.next(&req).await;
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
