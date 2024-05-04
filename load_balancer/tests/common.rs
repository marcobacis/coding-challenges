use futures::future::join_all;
use lb::{Backend, Config};
use reqwest::Client;
use wiremock::MockServer;

pub async fn wait_server_up(client: &Client, uri: &str, max_retries: usize) {
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

pub async fn create_mocks(n: usize) -> Vec<MockServer> {
    join_all((0..n).map(|_| MockServer::start())).await
}

pub fn build_config(mocks: &[MockServer]) -> Config {
    let backends = mocks
        .iter()
        .map(|mock| Backend {
            url: mock.uri(),
            health_url: format!("{}/health", mock.uri()),
        })
        .collect();

    Config { backends }
}
