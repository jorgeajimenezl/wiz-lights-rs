//! Runtime-agnostic async abstractions.
//!
//! This module provides traits and implementations that allow the library to work
//! with any async runtime (tokio, async-std, smol).
//!
//! # Feature Flags
//!
//! Enable one of the following features to select your runtime:
//!
//! - `runtime-tokio` (default) - Use the tokio runtime
//! - `runtime-async-std` - Use the async-std runtime
//! - `runtime-smol` - Use the smol runtime
//!
//! # Example
//!
//! ```toml
//! [dependencies]
//! # Using async-std
//! wiz-lights-rs = { version = "0.1", default-features = false, features = ["runtime-async-std"] }
//!
//! # Using smol
//! wiz-lights-rs = { version = "0.1", default-features = false, features = ["runtime-smol"] }
//! ```

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::time::Duration;

#[cfg(feature = "runtime-tokio")]
mod tokio_impl;

#[cfg(feature = "runtime-async-std")]
mod async_std_impl;

#[cfg(feature = "runtime-smol")]
mod smol_impl;

// Re-export the active runtime's types
#[cfg(feature = "runtime-tokio")]
pub use tokio_impl::*;

#[cfg(feature = "runtime-async-std")]
pub use async_std_impl::*;

#[cfg(feature = "runtime-smol")]
pub use smol_impl::*;

/// A boxed future type for runtime abstraction.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait for async UDP socket operations.
///
/// This trait abstracts over different async runtime's UDP socket implementations,
/// allowing the library to be runtime-agnostic.
pub trait AsyncUdpSocket: Send + Sync + Sized {
    /// Bind to the specified address.
    fn bind(addr: &str) -> impl Future<Output = io::Result<Self>> + Send;

    /// Connect to the specified address.
    fn connect(&self, addr: &str) -> impl Future<Output = io::Result<()>> + Send;

    /// Send data to the connected address.
    fn send(&self, buf: &[u8]) -> impl Future<Output = io::Result<usize>> + Send;

    /// Receive data from the connected address.
    fn recv(&self, buf: &mut [u8]) -> impl Future<Output = io::Result<usize>> + Send;

    /// Send data to a specific address.
    fn send_to(&self, buf: &[u8], addr: &str) -> impl Future<Output = io::Result<usize>> + Send;

    /// Receive data and the source address.
    fn recv_from(
        &self,
        buf: &mut [u8],
    ) -> impl Future<Output = io::Result<(usize, SocketAddr)>> + Send;

    /// Enable or disable broadcast mode.
    fn set_broadcast(&self, broadcast: bool) -> io::Result<()>;
}

/// Trait for async task spawning.
///
/// This trait abstracts over different async runtime's task spawning mechanisms.
pub trait Spawner {
    /// A handle to a spawned task.
    type JoinHandle<T: Send + 'static>: Future<Output = T> + Send;

    /// Spawn a future as a background task.
    fn spawn<F, T>(future: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;
}

/// Sleep for the specified duration.
pub async fn sleep(duration: Duration) {
    sleep_impl(duration).await
}

/// Run a future with a timeout.
///
/// Returns `Err(TimedOut)` if the timeout expires before the future completes.
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, TimedOut>
where
    F: Future<Output = T>,
{
    timeout_impl(duration, future).await
}

/// Error returned when a timeout expires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedOut;

impl std::fmt::Display for TimedOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation timed out")
    }
}

impl std::error::Error for TimedOut {}

/// A measurement of monotonically increasing time.
#[derive(Debug, Clone, Copy)]
pub struct Instant(InstantInner);

impl Instant {
    /// Returns the current instant.
    pub fn now() -> Self {
        Instant(InstantInner::now())
    }

    /// Returns the duration elapsed since this instant was created.
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}

// Async mutex re-export
#[cfg(feature = "runtime-tokio")]
pub use tokio::sync::Mutex;

#[cfg(feature = "runtime-async-std")]
pub use async_std::sync::Mutex;

#[cfg(feature = "runtime-smol")]
pub use async_lock::Mutex;

// JoinHandle type alias for task spawning
#[cfg(feature = "runtime-tokio")]
pub type JoinHandle<T> = tokio_impl::TokioJoinHandle<T>;

#[cfg(feature = "runtime-async-std")]
pub type JoinHandle<T> = async_std_impl::AsyncStdJoinHandle<T>;

#[cfg(feature = "runtime-smol")]
pub type JoinHandle<T> = smol_impl::SmolJoinHandle<T>;

// Compile-time check to ensure exactly one runtime is selected
#[cfg(not(any(
    feature = "runtime-tokio",
    feature = "runtime-async-std",
    feature = "runtime-smol"
)))]
compile_error!(
    "One of \"runtime-tokio\", \"runtime-async-std\", or \"runtime-smol\" features must be enabled"
);

#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("Features \"runtime-tokio\" and \"runtime-async-std\" are mutually exclusive");

#[cfg(all(feature = "runtime-tokio", feature = "runtime-smol"))]
compile_error!("Features \"runtime-tokio\" and \"runtime-smol\" are mutually exclusive");

#[cfg(all(feature = "runtime-async-std", feature = "runtime-smol"))]
compile_error!("Features \"runtime-async-std\" and \"runtime-smol\" are mutually exclusive");
