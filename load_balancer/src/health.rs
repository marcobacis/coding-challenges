use std::{iter::zip, sync::mpsc::Sender, time::Duration};

use futures::future::join_all;
use reqwest::Client;
use tokio::time;

use crate::Backend;

pub async fn health_thread(config: Vec<Backend>, sender: &mut Sender<Vec<Backend>>) {
    let mut interval = time::interval(Duration::from_secs(30));

    loop {
        let healthy_backends = get_healthy_backends(&config).await;

        sender.send(healthy_backends);

        interval.tick().await;
    }
}

async fn get_healthy_backends(backends: &Vec<Backend>) -> Vec<Backend> {
    let client = Client::new();

    let results = join_all(backends.iter().map(|b| client.get(&b.health_url).send())).await;

    zip(backends, results.iter().map(|r| r.is_ok()))
        .filter_map(|(b, res)| if res { Some(b.clone()) } else { None })
        .collect()
}
