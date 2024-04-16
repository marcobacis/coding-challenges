use std::sync::Arc;

use lb::{policies::RoundRobinPolicy, Backend, LoadBalancer};

#[tokio::main]
async fn main() {
    LoadBalancer::new(
        8080,
        vec![Backend {
            url: String::from("http://127.0.0.1:8081"),
            health_url: String::from("http://127.0.0.1:8081/health"),
        }],
        Arc::new(RoundRobinPolicy::new()),
    )
    .run()
    .await;
}
