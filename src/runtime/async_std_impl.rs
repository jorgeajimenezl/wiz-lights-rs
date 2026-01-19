//! async-std runtime implementation.

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use async_std::net::UdpSocket as AsyncStdUdpSocket;

use super::{AsyncUdpSocket, Spawner, TimedOut};

/// async-std-based UDP socket.
pub struct UdpSocket(AsyncStdUdpSocket);

impl AsyncUdpSocket for UdpSocket {
    async fn bind(addr: &str) -> io::Result<Self> {
        AsyncStdUdpSocket::bind(addr).await.map(UdpSocket)
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

/// async-std task spawner.
pub struct AsyncStdSpawner;

impl Spawner for AsyncStdSpawner {
    type JoinHandle<T: Send + 'static> = AsyncStdJoinHandle<T>;

    fn spawn<F, T>(future: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        AsyncStdJoinHandle(async_std::task::spawn(future))
    }
}

/// Wrapper around async-std's JoinHandle.
pub struct AsyncStdJoinHandle<T>(async_std::task::JoinHandle<T>);

impl<T> Future for AsyncStdJoinHandle<T> {
    type Output = T;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::pin::Pin::new(&mut self.0).poll(cx)
    }
}

impl<T: Send + 'static> AsyncStdJoinHandle<T> {
    /// Cancel the task.
    ///
    /// Note: async-std JoinHandle doesn't have abort, so this is a no-op.
    /// The task will continue running until it completes.
    pub fn abort(&self) {
        // async-std doesn't support task abortion directly
        // The task will be dropped when this handle is dropped
    }
}

/// Internal instant type for async-std.
#[derive(Debug, Clone, Copy)]
pub struct InstantInner(std::time::Instant);

impl InstantInner {
    pub fn now() -> Self {
        InstantInner(std::time::Instant::now())
    }

    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}

/// Sleep for the specified duration using async-std.
pub async fn sleep_impl(duration: Duration) {
    async_std::task::sleep(duration).await
}

/// Run a future with a timeout using async-std.
pub async fn timeout_impl<F, T>(duration: Duration, future: F) -> Result<T, TimedOut>
where
    F: Future<Output = T>,
{
    async_std::future::timeout(duration, future)
        .await
        .map_err(|_| TimedOut)
}

/// Spawn a task using async-std.
pub fn spawn<F, T>(future: F) -> AsyncStdJoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    AsyncStdSpawner::spawn(future)
}
