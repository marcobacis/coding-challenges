use std::{sync::Arc, time::Duration};

use lb::{policies::RoundRobinPolicy, LoadBalancer};
use reqwest::{Client, ClientBuilder, StatusCode};
use wiremock::{
    matchers::{body_json_string, method},
    Mock, ResponseTemplate,
};

use crate::common::{build_config, create_mocks, wait_server_up};

mod common;

#[tokio::test]
async fn test_get_root() {
    let mocks = create_mocks(2).await;
    let config = build_config(&mocks);

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mocks[0])
        .await;

    let client = Client::new();

    let policy = Arc::new(RoundRobinPolicy::new(config.clone()));

    let server = LoadBalancer::new(8080, config, policy);
    let server_uri = server.uri();
    tokio::spawn(async move { server.run().await });

    wait_server_up(&client, &server_uri, 3).await;

    let response = client.get(server_uri).send().await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn test_post_root() {
    let mocks = create_mocks(2).await;
    let config = build_config(&mocks);

    Mock::given(method("POST"))
        .and(body_json_string("{}"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mocks[0])
        .await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mocks[0])
        .await;

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    let policy = Arc::new(RoundRobinPolicy::new(config.clone()));
    let server = LoadBalancer::new(8081, config, policy);
    let server_uri = server.uri();
    tokio::spawn(async move { server.run().await });

    wait_server_up(&client, &server_uri, 3).await;

    let response = client.post(server_uri).body("{}").send().await.unwrap();
    assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn test_round_robin_three_servers() {
    let mocks = create_mocks(3).await;
    let config = build_config(&mocks);

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("1"))
        .mount(&mocks[0])
        .await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("2"))
        .mount(&mocks[1])
        .await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("3"))
        .mount(&mocks[2])
        .await;

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    let policy = Arc::new(RoundRobinPolicy::new(config.clone()));

    // Spawn server
    let server = LoadBalancer::new(8082, config, policy);
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
