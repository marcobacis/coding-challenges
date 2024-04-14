use tokio::sync::RwLock;

pub struct Policy {
    servers: Vec<String>,
    idx: RwLock<usize>,
}

impl Policy {
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            servers,
            idx: RwLock::new(0),
        }
    }

    pub async fn next(&self) -> &str {
        let max_server_idx = self.servers.len() - 1;

        // Update index
        let idx = {
            let mut idx = self.idx.write().await;
            let current_idx = *idx;
            *idx = match *idx {
                x if x >= max_server_idx => 0,
                c => c + 1,
            };
            current_idx
        };

        self.servers.get(idx).unwrap()
    }
}
