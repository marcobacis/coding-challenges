use std::sync::Arc;

use lb::{policies::RoundRobinPolicy, Backend, LoadBalancer};

#[tokio::main]
async fn main() {
    let config = vec![Backend {
        url: String::from("http://127.0.0.1:8081"),
        health_url: String::from("http://127.0.0.1:8081/health"),
    }];

    let policy = Arc::new(RoundRobinPolicy::new(config.clone()));

    LoadBalancer::new(8080, config.clone(), policy)
        .run()
        .await;
}
