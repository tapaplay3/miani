pub struct Config {
    pub endpoint: String,
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:1234/v1".to_string(),
            model: "local-model".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
