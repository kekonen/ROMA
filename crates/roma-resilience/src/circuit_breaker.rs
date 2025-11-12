use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_threshold: u32,
    timeout: Duration,
}

struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
            })),
            failure_threshold,
            timeout,
        }
    }

    pub async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        {
            let mut state = self.state.write();

            match state.state {
                CircuitState::Open => {
                    if let Some(last_failure) = state.last_failure_time {
                        if last_failure.elapsed() > self.timeout {
                            info!("Circuit breaker transitioning to half-open");
                            state.state = CircuitState::HalfOpen;
                            state.success_count = 0;
                        } else {
                            return Err(CircuitBreakerError::CircuitOpen);
                        }
                    }
                }
                _ => {}
            }
        }

        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(CircuitBreakerError::OperationFailed(e))
            }
        }
    }

    fn on_success(&self) {
        let mut state = self.state.write();

        match state.state {
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= 3 {
                    info!("Circuit breaker closing after successful recovery");
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                }
            }
            CircuitState::Closed => {
                state.failure_count = 0;
            }
            _ => {}
        }
    }

    fn on_failure(&self) {
        let mut state = self.state.write();

        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        if state.failure_count >= self.failure_threshold {
            warn!(
                "Circuit breaker opening after {} failures",
                state.failure_count
            );
            state.state = CircuitState::Open;
        }
    }

    pub fn current_state(&self) -> CircuitState {
        self.state.read().state
    }

    pub fn reset(&self) {
        let mut state = self.state.write();
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.last_failure_time = None;
        info!("Circuit breaker manually reset");
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    CircuitOpen,
    OperationFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::OperationFailed(e) => write!(f, "Operation failed: {}", e),
        }
    }
}

impl<E: std::error::Error> std::error::Error for CircuitBreakerError<E> {}
