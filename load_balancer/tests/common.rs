use reqwest::Client;

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
