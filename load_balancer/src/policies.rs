use std::sync::atomic::{AtomicUsize, Ordering};

use actix_web::HttpRequest;

use crate::Backend;

pub trait RoutingPolicy {
    fn next(&self, request: &HttpRequest, servers: &Vec<Backend>) -> String;
}

pub struct RoundRobinPolicy {
    idx: AtomicUsize,
}

impl Default for RoundRobinPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl RoundRobinPolicy {
    pub fn new() -> Self {
        Self {
            idx: AtomicUsize::new(0),
        }
    }
}

impl RoutingPolicy for RoundRobinPolicy {
    fn next(&self, _request: &HttpRequest, servers: &Vec<Backend>) -> String {
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
}
