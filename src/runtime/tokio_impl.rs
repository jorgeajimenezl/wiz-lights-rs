//! Tokio runtime implementation.

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::UdpSocket as TokioUdpSocket;

use super::{AsyncUdpSocket, Spawner, TimedOut};

/// Tokio-based UDP socket.
pub struct UdpSocket(TokioUdpSocket);

impl AsyncUdpSocket for UdpSocket {
    async fn bind(addr: &str) -> io::Result<Self> {
        TokioUdpSocket::bind(addr).await.map(UdpSocket)
    }

    async fn connect(&self, addr: &str) -> io::Result<()> {
        self.0.connect(addr).await
    }

    async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf).await
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf).await
    }

    async fn send_to(&self, buf: &[u8], addr: &str) -> io::Result<usize> {
        self.0.send_to(buf, addr).await
    }

    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf).await
    }

    fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
        self.0.set_broadcast(broadcast)
    }
}

/// Tokio task spawner.
pub struct TokioSpawner;

impl Spawner for TokioSpawner {
    type JoinHandle<T: Send + 'static> = TokioJoinHandle<T>;

    fn spawn<F, T>(future: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        TokioJoinHandle(tokio::spawn(future))
    }
}

/// Wrapper around tokio's JoinHandle that extracts the value on await.
pub struct TokioJoinHandle<T>(tokio::task::JoinHandle<T>);

impl<T> Future for TokioJoinHandle<T> {
    type Output = T;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        use std::task::Poll;
        match std::pin::Pin::new(&mut self.0).poll(cx) {
            Poll::Ready(Ok(v)) => Poll::Ready(v),
            Poll::Ready(Err(e)) => {
                // Task was cancelled or panicked - propagate panic
                if e.is_panic() {
                    std::panic::resume_unwind(e.into_panic());
                }
                // Task was cancelled - this shouldn't happen in normal usage
                panic!("Task was cancelled unexpectedly");
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: Send + 'static> TokioJoinHandle<T> {
    /// Abort the task.
    pub fn abort(&self) {
        self.0.abort();
    }
}

/// Internal instant type for tokio.
#[derive(Debug, Clone, Copy)]
pub struct InstantInner(tokio::time::Instant);

impl InstantInner {
    pub fn now() -> Self {
        InstantInner(tokio::time::Instant::now())
    }

    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}

/// Sleep for the specified duration using tokio.
pub async fn sleep_impl(duration: Duration) {
    tokio::time::sleep(duration).await
}

/// Run a future with a timeout using tokio.
pub async fn timeout_impl<F, T>(duration: Duration, future: F) -> Result<T, TimedOut>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| TimedOut)
}

/// Spawn a task using tokio.
pub fn spawn<F, T>(future: F) -> TokioJoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    TokioSpawner::spawn(future)
}
