use std::sync::Arc;

use lb::{config::Backend, config::Config, policies::RoundRobinPolicy, LoadBalancer};

#[tokio::main]
async fn main() {
    let config = Config {
        healthcheck_interval_secs: 30,
        backends: vec![
            Backend {
                url: String::from("http://127.0.0.1:8081"),
                health_url: String::from("http://127.0.0.1:8081/health"),
            },
            Backend {
                url: String::from("http://127.0.0.1:8082"),
                health_url: String::from("http://127.0.0.1:8082/health"),
            },
            Backend {
                url: String::from("http://127.0.0.1:8083"),
                health_url: String::from("http://127.0.0.1:8083/health"),
            },
        ],
    };

    let policy = Arc::new(RoundRobinPolicy::new(&config));

    LoadBalancer::new(8080, config.clone(), policy).run().await;
}
