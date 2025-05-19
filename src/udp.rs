use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::{Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{debug, error, span, trace, Level};

const BUFFER_SIZE: usize = 64 * 1024;
const TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug)]
pub struct Proxy {
    socket: Arc<UdpSocket>,
}

#[async_trait]
impl super::Proxy for Proxy {
    async fn listen(bind: SocketAddr) -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind(bind).await?);
        debug!("Created UDP socket");

        return Ok(Self { socket });
    }

    async fn run(mut self: Box<Self>, target: SocketAddr) -> Result<()> {
        let mut buffer = [0u8; BUFFER_SIZE];

        let conn_track: Arc<Mutex<HashMap<SocketAddr, Arc<UdpSocket>>>> = Default::default();

        loop {
            trace!("Waiting for next packet");

            let (size, remote) = self.socket.recv_from(&mut buffer).await?;
            let buffer = &buffer[0..size];
            trace!("Received UDP packet from client: {} bytes", size);

            let _ = span!(Level::DEBUG, "Session", remote = ?remote).entered();

            let upstream = {
                match conn_track.lock().await.entry(remote) {
                    Entry::Vacant(entry) => {
                        debug!("New UDP session: {}", remote);

                        let upstream = Arc::new(
                            UdpSocket::bind(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0))
                                .await
                                .context("Failed to bind upstream socket")?,
                        );
                        upstream.connect(target).await?;
                        trace!("Upstream socket created: {}", upstream.local_addr()?);

                        {
                            let conn_track = conn_track.clone();
                            let upstream = upstream.clone();
                            let socket = self.socket.clone();

                            tokio::spawn(async move {
                                let mut buffer = [0u8; BUFFER_SIZE];

                                loop {
                                    trace!("Waiting for next packet");

                                    match tokio::time::timeout(TIMEOUT, upstream.recv(&mut buffer))
                                        .await
                                    {
                                        Ok(Ok(size)) => {
                                            trace!("Packet received from upstream: {}", size);
                                            trace!("Responding to: {}", remote);

                                            socket
                                                .send_to(&buffer[0..size], &remote)
                                                .await
                                                .context("Failed to send response")?;
                                        }

                                        Ok(Err(err)) => {
                                            error!("Failed to receive from upstream: {}", err);
                                            bail!(err);
                                        }

                                        Err(_) => {
                                            trace!("UDP session timeout");

                                            conn_track.lock().await.remove(&remote);

                                            break;
                                        }
                                    }
                                }

                                Ok::<(), anyhow::Error>(())
                            });
                        }

                        entry.insert(upstream).clone()
                    }

                    Entry::Occupied(entry) => {
                        trace!("Reuse existing socket");
                        entry.into_mut().clone()
                    }
                }
            };

            upstream.send(buffer).await?;
        }
    }
}
