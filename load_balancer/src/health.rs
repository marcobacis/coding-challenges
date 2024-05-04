use std::{iter::zip, time::Duration};

use futures::future::join_all;
use reqwest::{Client, Error, Response};
use tokio::{sync::mpsc::Sender, time};

use crate::{Backend, Config};

pub async fn health_thread(config: Config, sender: &Sender<Vec<Backend>>) {
    let mut interval = time::interval(Duration::from_secs(30));

    loop {
        let healthy_backends = get_healthy_backends(&config.backends).await;

        sender.send(healthy_backends).await;

        interval.tick().await;
    }
}

async fn get_healthy_backends(backends: &Vec<Backend>) -> Vec<Backend> {
    let client = Client::new();

    let results = join_all(backends.iter().map(|b| client.get(&b.health_url).send())).await;

    zip(backends, results.iter().map(is_healthy))
        .filter_map(|(b, res)| if res { Some(b.clone()) } else { None })
        .collect()
}

fn is_healthy(res: &Result<Response, Error>) -> bool {
    match res {
        Ok(response) => response.status().is_success(),
        _ => false,
    }
}
