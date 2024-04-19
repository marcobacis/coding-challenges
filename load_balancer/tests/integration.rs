use std::{sync::Arc, time::Duration};

use lb::{policies::RoundRobinPolicy, Backend, LoadBalancer};
use reqwest::{Client, ClientBuilder, StatusCode};
use wiremock::{
    matchers::{body_json_string, method},
    Mock, MockServer, ResponseTemplate,
};

use crate::common::wait_server_up;

mod common;

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
async fn test_round_robin_three_servers() {
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
