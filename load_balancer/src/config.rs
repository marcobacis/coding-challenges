#[derive(Clone, Debug)]
pub struct Backend {
    pub url: String,
    pub health_url: String,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub healthcheck_interval_secs: usize,
    pub backends: Vec<Backend>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            healthcheck_interval_secs: 30,
            backends: Default::default(),
        }
    }
}
