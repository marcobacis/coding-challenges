use std::{sync::Arc, time::Duration};

use lb::{policies::RoundRobinPolicy, LoadBalancer};
use reqwest::{ClientBuilder, StatusCode};
use wiremock::{matchers::method, Mock, ResponseTemplate};

mod common;
use crate::common::{build_config, create_mocks, wait_server_up};

#[tokio::test]
async fn test_health_check_simple() {
    let mocks = create_mocks(2).await;
    let config = build_config(&mocks);

    // Only mock 1 responds
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("1"))
        .mount(&mocks[1])
        .await;

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    let policy = Arc::new(RoundRobinPolicy::new(config.clone()));
    let server = LoadBalancer::new(8082, config, policy);
    let server_uri = server.uri();

    tokio::spawn(async move { server.run().await });
    wait_server_up(&client, &server_uri, 3).await;

    // Wait some time to let the health check work
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Expect only mock[1] to respond to 2 consecutive requests
    let response = client.get(&server_uri).send().await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
    assert_eq!("1", response.text().await.unwrap());

    let response = client.get(&server_uri).send().await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
    assert_eq!("1", response.text().await.unwrap());
}
