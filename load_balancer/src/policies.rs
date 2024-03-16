use std::sync::Mutex;

pub struct Policy {
    servers: Vec<String>,
    idx: Mutex<usize>,
}

impl Policy {
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            servers,
            idx: Mutex::new(0),
        }
    }

    pub fn next(&self) -> &str {
        let idx = {
            let mut current_server_idx = self.idx.lock().unwrap();
            let idx = (*current_server_idx).clone();
            let max_server_idx = self.servers.len() - 1;

            *current_server_idx = match idx {
                x if x == max_server_idx => 0,
                c => c + 1,
            };
            idx
        };
        self.servers.get(idx).unwrap()
    }
}
