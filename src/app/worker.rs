// ─── < Imports > ────────────────────────────────────────────────────

use anyhow::{Context, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

// ─── < Constants > ────────────────────────────────────────────────────

const SHUTDOWN_POLL_INTERVAL: Duration = Duration::from_millis(250);

// ─── < Structs > ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ShutdownToken {
    should_stop: Arc<AtomicBool>,
}

pub struct WorkerHandle {
    name: &'static str,
    shutdown: ShutdownToken,
    handle: Option<JoinHandle<()>>,
}

// ─── < Implementations > ────────────────────────────────────────────────────

impl ShutdownToken {
    fn new() -> Self {
        Self {
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn request_shutdown(&self) {
        self.should_stop.store(true, Ordering::Release);
    }

    pub fn should_stop(&self) -> bool {
        self.should_stop.load(Ordering::Acquire)
    }

    /// Duerme como mucho `duration`, despertándose seguido para mirar el
    /// flag. Devuelve `true` si el sueño se cortó por un apagado pedido.
    pub fn sleep(&self, duration: Duration) -> bool {
        let started_at = Instant::now();

        while !self.should_stop() {
            let elapsed = started_at.elapsed();

            if elapsed >= duration {
                return false;
            }

            let remaining = duration.saturating_sub(elapsed);
            thread::sleep(remaining.min(SHUTDOWN_POLL_INTERVAL));
        }

        true
    }
}

impl WorkerHandle {
    pub fn spawn<F>(name: &'static str, run: F) -> Result<Self>
    where
        F: FnOnce(ShutdownToken) + Send + 'static,
    {
        let shutdown = ShutdownToken::new();
        let worker_shutdown = shutdown.clone();

        let handle = thread::Builder::new()
            .name(name.to_string())
            .spawn(move || run(worker_shutdown))
            .with_context(|| format!("no se pudo iniciar el worker {name}"))?;

        Ok(Self {
            name,
            shutdown,
            handle: Some(handle),
        })
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.shutdown.request_shutdown();

        let Some(handle) = self.handle.take() else {
            return;
        };

        if handle.join().is_err() {
            log::warn!("worker {} terminó con panic", self.name);
        }
    }
}
