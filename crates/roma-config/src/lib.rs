use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomaConfig {
    pub agents: AgentsConfig,
    pub runtime: RuntimeConfig,
    pub resilience: ResilienceConfig,
    pub storage: StorageConfig,
    pub observability: ObservabilityConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    pub atomizer: AgentConfig,
    pub planner: AgentConfig,
    pub executor: AgentConfig,
    pub aggregator: AgentConfig,
    pub verifier: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub llm: LlmConfig,
    pub prediction_strategy: PredictionStrategy,
    pub toolkits: Vec<String>,
    #[serde(default)]
    pub agent_config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    #[serde(default)]
    pub additional_params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionStrategy {
    ChainOfThought,
    ReAct,
    CodeAct,
    BestOfN,
    Direct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub max_depth: usize,
    pub timeout: u64,
    pub verbose: bool,
    pub max_concurrent_tasks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    pub retry: RetryConfig,
    pub circuit_breaker: CircuitBreakerConfig,
    pub checkpoint: CheckpointConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub enabled: bool,
    pub strategy: RetryStrategy,
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryStrategy {
    Exponential,
    Fixed,
    Jittered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    pub enabled: bool,
    pub storage_path: PathBuf,
    pub max_checkpoints: usize,
    pub interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub base_path: PathBuf,
    pub postgres: PostgresConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub enabled: bool,
    pub connection_url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub tracing: TracingConfig,
    pub event_traces: EventTracesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub service_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTracesConfig {
    pub enabled: bool,
    pub track_module_events: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub console_format: ConsoleFormat,
    pub rotation_size_mb: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsoleFormat {
    Detailed,
    Compact,
    Json,
}

impl Default for RomaConfig {
    fn default() -> Self {
        Self {
            agents: AgentsConfig {
                atomizer: AgentConfig {
                    llm: LlmConfig {
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        temperature: 0.7,
                        max_tokens: 4096,
                        additional_params: HashMap::new(),
                    },
                    prediction_strategy: PredictionStrategy::ChainOfThought,
                    toolkits: vec![],
                    agent_config: HashMap::new(),
                },
                planner: AgentConfig {
                    llm: LlmConfig {
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        temperature: 0.7,
                        max_tokens: 8192,
                        additional_params: HashMap::new(),
                    },
                    prediction_strategy: PredictionStrategy::ChainOfThought,
                    toolkits: vec![],
                    agent_config: HashMap::new(),
                },
                executor: AgentConfig {
                    llm: LlmConfig {
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        temperature: 0.0,
                        max_tokens: 8192,
                        additional_params: HashMap::new(),
                    },
                    prediction_strategy: PredictionStrategy::ReAct,
                    toolkits: vec!["artifact".to_string(), "file".to_string()],
                    agent_config: HashMap::new(),
                },
                aggregator: AgentConfig {
                    llm: LlmConfig {
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        temperature: 0.7,
                        max_tokens: 8192,
                        additional_params: HashMap::new(),
                    },
                    prediction_strategy: PredictionStrategy::ChainOfThought,
                    toolkits: vec![],
                    agent_config: HashMap::new(),
                },
                verifier: AgentConfig {
                    llm: LlmConfig {
                        provider: "openai".to_string(),
                        model: "gpt-4".to_string(),
                        temperature: 0.3,
                        max_tokens: 4096,
                        additional_params: HashMap::new(),
                    },
                    prediction_strategy: PredictionStrategy::ChainOfThought,
                    toolkits: vec![],
                    agent_config: HashMap::new(),
                },
            },
            runtime: RuntimeConfig {
                max_depth: 6,
                timeout: 120,
                verbose: true,
                max_concurrent_tasks: 4,
            },
            resilience: ResilienceConfig {
                retry: RetryConfig {
                    enabled: true,
                    strategy: RetryStrategy::Exponential,
                    max_attempts: 3,
                    base_delay_ms: 1000,
                    max_delay_ms: 10000,
                },
                circuit_breaker: CircuitBreakerConfig {
                    enabled: true,
                    failure_threshold: 5,
                    timeout_ms: 60000,
                },
                checkpoint: CheckpointConfig {
                    enabled: true,
                    storage_path: PathBuf::from("/tmp/roma/checkpoints"),
                    max_checkpoints: 10,
                    interval_seconds: 300,
                },
            },
            storage: StorageConfig {
                base_path: PathBuf::from("/tmp/roma"),
                postgres: PostgresConfig {
                    enabled: false,
                    connection_url: String::new(),
                    pool_size: 10,
                },
            },
            observability: ObservabilityConfig {
                tracing: TracingConfig {
                    enabled: true,
                    endpoint: None,
                    service_name: "roma".to_string(),
                },
                event_traces: EventTracesConfig {
                    enabled: true,
                    track_module_events: true,
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                console_format: ConsoleFormat::Detailed,
                rotation_size_mb: 500,
            },
        }
    }
}

pub struct ConfigManager {
    config: RomaConfig,
}

impl ConfigManager {
    pub fn new() -> Result<Self, ConfigError> {
        Self::with_profile("general")
    }

    pub fn with_profile(profile: &str) -> Result<Self, ConfigError> {
        let config_path = format!("config/profiles/{}.yaml", profile);

        let builder = Config::builder()
            .add_source(File::with_name(&config_path).required(false))
            .add_source(File::with_name("config/default").required(false))
            .add_source(Environment::with_prefix("ROMA").separator("__"));

        let config = builder.build()?.try_deserialize()?;

        Ok(Self { config })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let builder = Config::builder()
            .add_source(File::from(path.as_ref()))
            .add_source(Environment::with_prefix("ROMA").separator("__"));

        let config = builder.build()?.try_deserialize()?;

        Ok(Self { config })
    }

    pub fn config(&self) -> &RomaConfig {
        &self.config
    }

    pub fn into_config(self) -> RomaConfig {
        self.config
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self {
            config: RomaConfig::default(),
        }
    }
}
