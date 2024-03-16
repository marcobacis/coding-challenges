use lb::LoadBalancer;

#[tokio::main]
async fn main() {
    LoadBalancer::new(
        8080,
        vec![
            String::from("http://127.0.0.1:8081"),
            String::from("http://127.0.0.1:8082"),
            String::from("http://127.0.0.1:8083"),
        ],
    )
    .run()
    .await;
}
