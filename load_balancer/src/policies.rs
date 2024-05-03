use std::sync::atomic::{AtomicUsize, Ordering};

use actix_web::HttpRequest;

use tokio::sync::RwLock;

use crate::Backend;

pub trait RoutingPolicy {
    fn next(&self, request: &HttpRequest) -> impl std::future::Future<Output = String>;

    fn health_results(
        &self,
        results: Vec<Backend>,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send;
}

pub struct RoundRobinPolicy {
    idx: AtomicUsize,
    servers: RwLock<Vec<Backend>>,
}

impl Default for RoundRobinPolicy {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl RoundRobinPolicy {
    pub fn new(servers: Vec<Backend>) -> Self {
        Self {
            idx: AtomicUsize::new(0),
            servers: RwLock::new(servers.clone()),
        }
    }
}

impl RoutingPolicy for RoundRobinPolicy {
    async fn next(&self, _request: &HttpRequest) -> String {
        let servers = self.servers.read().await.clone();
        let max_server_idx = servers.len() - 1;

        // Update index
        let idx = self
            .idx
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |idx| match idx {
                x if x >= max_server_idx => Some(0),
                c => Some(c + 1),
            })
            .unwrap_or_default();

        servers.get(idx).unwrap().url.clone()
    }

    async fn health_results(&self, results: Vec<Backend>) {
        let mut servers = self.servers.write().await;
        *servers = results;
    }
}
