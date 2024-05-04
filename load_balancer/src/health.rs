use std::{iter::zip, time::Duration};

use futures::future::join_all;
use reqwest::{Client, Error, Response};
use tokio::{sync::mpsc::Sender, time};

use crate::{Backend, Config};

pub struct HealthResult {
    pub backend: Backend,
    healthy: bool,
}

impl HealthResult {
    pub fn is_healthy(&self) -> bool {
        self.healthy
    }
}

pub async fn health_thread(config: Config, sender: &Sender<Vec<HealthResult>>) {
    let mut interval = time::interval(Duration::from_secs(config.healthcheck_interval_secs as u64));

    loop {
        let healthy_backends = get_healthy_backends(&config.backends).await;

        let _ = sender.send(healthy_backends).await;

        interval.tick().await;
    }
}

async fn get_healthy_backends(backends: &Vec<Backend>) -> Vec<HealthResult> {
    let client = Client::new();

    let results = join_all(backends.iter().map(|b| {
        client
            .get(format!("{}{}", &b.url, &b.healthcheck_path))
            .send()
    }))
    .await;

    zip(backends, results.iter().map(is_healthy))
        .map(|(backend, healthy)| HealthResult {
            backend: backend.clone(),
            healthy,
        })
        .collect()
}

fn is_healthy(res: &Result<Response, Error>) -> bool {
    match res {
        Ok(response) => response.status().is_success(),
        _ => false,
    }
}
