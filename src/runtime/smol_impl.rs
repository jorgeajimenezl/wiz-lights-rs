//! smol runtime implementation.

use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use async_io::Async;

use super::{AsyncUdpSocket, Spawner, TimedOut};

/// smol-based UDP socket using async-io.
pub struct UdpSocket(Async<std::net::UdpSocket>);

impl AsyncUdpSocket for UdpSocket {
    async fn bind(addr: &str) -> io::Result<Self> {
        let socket = std::net::UdpSocket::bind(addr)?;
        Async::new(socket).map(UdpSocket)
    }

    async fn connect(&self, addr: &str) -> io::Result<()> {
        self.0.get_ref().connect(addr)
    }

    async fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.send(buf).await
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.recv(buf).await
    }

    async fn send_to(&self, buf: &[u8], addr: &str) -> io::Result<usize> {
        let addr: SocketAddr = addr
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        self.0.send_to(buf, addr).await
    }

    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf).await
    }

    fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
        self.0.get_ref().set_broadcast(broadcast)
    }
}

/// smol task spawner.
pub struct SmolSpawner;

impl Spawner for SmolSpawner {
    type JoinHandle<T: Send + 'static> = SmolJoinHandle<T>;

    fn spawn<F, T>(future: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        SmolJoinHandle(smol::spawn(future))
    }
}

/// Wrapper around smol's Task.
pub struct SmolJoinHandle<T>(smol::Task<T>);

impl<T> Future for SmolJoinHandle<T> {
    type Output = T;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::pin::Pin::new(&mut self.0).poll(cx)
    }
}

impl<T: Send + 'static> SmolJoinHandle<T> {
    /// Cancel the task.
    ///
    /// Note: smol's Task is cancelled when dropped, but this method
    /// provides an explicit way to signal cancellation intent.
    pub fn abort(&self) {
        // smol doesn't have an explicit abort - tasks are cancelled when dropped
        // This is a no-op for API compatibility
    }
}

/// Internal instant type for smol.
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

/// Sleep for the specified duration using smol.
pub async fn sleep_impl(duration: Duration) {
    smol::Timer::after(duration).await;
}

/// Run a future with a timeout using smol.
pub async fn timeout_impl<F, T>(duration: Duration, future: F) -> Result<T, TimedOut>
where
    F: Future<Output = T>,
{
    use futures::future::Either;

    let timeout_future = smol::Timer::after(duration);

    futures::pin_mut!(future);
    futures::pin_mut!(timeout_future);

    match futures::future::select(future, timeout_future).await {
        Either::Left((result, _)) => Ok(result),
        Either::Right((_, _)) => Err(TimedOut),
    }
}

/// Spawn a task using smol.
pub fn spawn<F, T>(future: F) -> SmolJoinHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    SmolSpawner::spawn(future)
}
