use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Provider {
    LMStudio,
    Ollama,
}

impl Provider {
    pub fn default_endpoint(&self) -> &str {
        match self {
            Provider::LMStudio => "http://localhost:1234/v1",
            Provider::Ollama => "http://localhost:11434/v1",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Provider::LMStudio => "LM Studio",
            Provider::Ollama => "Ollama",
        }
    }

    pub fn port_hint(&self) -> &str {
        match self {
            Provider::LMStudio => "1234",
            Provider::Ollama => "11434",
        }
    }

    pub fn setup_hint(&self) -> &str {
        match self {
            Provider::LMStudio => "Make sure LM Studio is running on port 1234",
            Provider::Ollama => "Make sure Ollama is running on port 11434 (run 'ollama serve')",
        }
    }
}

impl FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lmstudio" | "lm-studio" | "lms" => Ok(Provider::LMStudio),
            "ollama" => Ok(Provider::Ollama),
            _ => Err(format!("Unknown provider: {}. Available: lmstudio, ollama", s)),
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

pub struct Config {
    pub provider: Provider,
    pub endpoint: String,
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        let provider = Provider::LMStudio;
        Self {
            provider,
            endpoint: provider.default_endpoint().to_string(),
            model: "local-model".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_provider(provider: Provider) -> Self {
        Self {
            provider,
            endpoint: provider.default_endpoint().to_string(),
            model: "local-model".to_string(),
        }
    }
}
