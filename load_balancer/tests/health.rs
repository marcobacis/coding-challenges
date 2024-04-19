use std::{sync::Arc, time::Duration};

use lb::{policies::RoundRobinPolicy, Backend, LoadBalancer};
use reqwest::ClientBuilder;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

mod common;
use crate::common::wait_server_up;

#[tokio::test]
async fn test_health_check_simple() {
    let mocks = vec![MockServer::start().await, MockServer::start().await];

    // TODO set unresponsive/error on mock[0]

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("1"))
        .mount(&mocks[1])
        .await;

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let policy = Arc::new(RoundRobinPolicy::new());
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

    // TODO Wait some time to let the health check work

    // TODO Expect only mock[0] to respond to 2 consecutive requests
}
