use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

pub trait RetryPolicy: Send + Sync {
    fn next_delay(&self, attempt: u32) -> Option<Duration>;
}

pub struct ExponentialBackoff {
    base_delay_ms: u64,
    max_delay_ms: u64,
    max_attempts: u32,
}

impl ExponentialBackoff {
    pub fn new(base_delay_ms: u64, max_delay_ms: u64, max_attempts: u32) -> Self {
        Self {
            base_delay_ms,
            max_delay_ms,
            max_attempts,
        }
    }
}

impl RetryPolicy for ExponentialBackoff {
    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }

        let delay = self.base_delay_ms * 2u64.pow(attempt);
        let delay = delay.min(self.max_delay_ms);

        Some(Duration::from_millis(delay))
    }
}

pub struct FixedDelay {
    delay_ms: u64,
    max_attempts: u32,
}

impl FixedDelay {
    pub fn new(delay_ms: u64, max_attempts: u32) -> Self {
        Self {
            delay_ms,
            max_attempts,
        }
    }
}

impl RetryPolicy for FixedDelay {
    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }

        Some(Duration::from_millis(self.delay_ms))
    }
}

pub struct JitteredBackoff {
    base_delay_ms: u64,
    max_delay_ms: u64,
    max_attempts: u32,
}

impl JitteredBackoff {
    pub fn new(base_delay_ms: u64, max_delay_ms: u64, max_attempts: u32) -> Self {
        Self {
            base_delay_ms,
            max_delay_ms,
            max_attempts,
        }
    }
}

impl RetryPolicy for JitteredBackoff {
    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt >= self.max_attempts {
            return None;
        }

        let base = self.base_delay_ms * 2u64.pow(attempt);
        let jitter = (base as f64 * 0.1) as u64;
        let delay = (base + jitter).min(self.max_delay_ms);

        Some(Duration::from_millis(delay))
    }
}

pub async fn retry_with_policy<F, Fut, T, E>(
    policy: &dyn RetryPolicy,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if let Some(delay) = policy.next_delay(attempt) {
                    warn!(
                        "Operation failed (attempt {}): {}. Retrying in {:?}",
                        attempt + 1,
                        e,
                        delay
                    );
                    sleep(delay).await;
                    attempt += 1;
                } else {
                    return Err(e);
                }
            }
        }
    }
}
