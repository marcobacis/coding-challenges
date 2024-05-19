use std::sync::Arc;

use actix_web::{
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer,
};
use health::HealthResult;
use tokio::sync::mpsc::channel;

use policies::{RoutingPolicy, SafeRoutingPolicy};
use reqwest::Client;
use tokio::sync::mpsc::{Receiver, Sender};

pub mod config;

pub use config::Backend;
pub use config::Config;
pub use error::LBError;

pub mod error;
mod health;
pub mod policies;

pub struct LoadBalancer {
    port: u16,
    data: Arc<AppState>,
    config: Config,
}

impl LoadBalancer {
    pub fn new(port: u16, config: Config, policy: Arc<SafeRoutingPolicy>) -> Self {
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

        let (tx, mut rx): (Sender<Vec<HealthResult>>, Receiver<Vec<HealthResult>>) = channel(32);

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
                .route("/health", web::get().to(HttpResponse::Ok))
                .default_service(web::to(Self::handler))
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

    async fn handler(
        req: HttpRequest,
        data: web::Data<AppState>,
        bytes: web::Bytes,
    ) -> Result<HttpResponse, LBError> {
        let server = data.policy.next(&req).await;
        let uri = format!("{}{}", server, req.uri());

        let request_builder = data
            .client
            .request(req.method().clone(), uri)
            .headers(req.headers().into())
            .body(bytes);

        let response = request_builder.send().await?;

        let mut response_builder = HttpResponse::build(response.status());
        for h in response.headers().iter() {
            response_builder.append_header(h);
        }
        let body = response.bytes().await?;

        Ok(response_builder.body(body))
    }
}

struct AppState {
    policy: Arc<SafeRoutingPolicy>,
    client: Client,
}
