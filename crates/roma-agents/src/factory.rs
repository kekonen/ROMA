use roma_config::AgentsConfig;
use std::sync::Arc;

use crate::{Aggregator, Atomizer, Executor, Planner, Verifier};

pub struct AgentFactory {
    config: AgentsConfig,
}

impl AgentFactory {
    pub fn new(config: AgentsConfig) -> Self {
        Self { config }
    }

    pub fn create_atomizer(&self) -> Arc<Atomizer> {
        Arc::new(Atomizer::new(self.config.atomizer.clone()))
    }

    pub fn create_planner(&self) -> Arc<Planner> {
        Arc::new(Planner::new(self.config.planner.clone()))
    }

    pub fn create_executor(&self) -> Arc<Executor> {
        Arc::new(Executor::new(self.config.executor.clone()))
    }

    pub fn create_aggregator(&self) -> Arc<Aggregator> {
        Arc::new(Aggregator::new(self.config.aggregator.clone()))
    }

    pub fn create_verifier(&self) -> Arc<Verifier> {
        Arc::new(Verifier::new(self.config.verifier.clone()))
    }

    pub fn create_all(&self) -> Agents {
        Agents {
            atomizer: self.create_atomizer(),
            planner: self.create_planner(),
            executor: self.create_executor(),
            aggregator: self.create_aggregator(),
            verifier: self.create_verifier(),
        }
    }
}

#[derive(Clone)]
pub struct Agents {
    pub atomizer: Arc<Atomizer>,
    pub planner: Arc<Planner>,
    pub executor: Arc<Executor>,
    pub aggregator: Arc<Aggregator>,
    pub verifier: Arc<Verifier>,
}
