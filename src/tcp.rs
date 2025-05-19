use std::net::SocketAddr;

use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, span, trace, warn, Level};

#[derive(Debug)]
pub struct Proxy {
    socket: TcpListener,
}

#[async_trait]
impl super::Proxy for Proxy {
    async fn listen(bind: SocketAddr) -> Result<Self> {
        let socket = TcpListener::bind(bind)
            .await
            .with_context(|| format!("Failed to bind to TCP socket: {bind}"))?;
        debug!("Created TCP socket");

        return Ok(Self { socket });
    }

    async fn run(mut self: Box<Self>, target: SocketAddr) -> Result<()> {
        loop {
            let result: Result<()> = async {
                trace!("Awaiting new connection");

                let (mut client, remote) = self
                    .socket
                    .accept()
                    .await
                    .context("Failed to accept connection")?;
                debug!("New client connected: {}", remote);

                let _ = span!(Level::DEBUG, "Client connection", connection = ?remote).entered();

                let mut upstream = TcpStream::connect(target)
                    .await
                    .with_context(|| format!("Failed to connect to target: {target}"))?;
                trace!("Upstream connection established");

                tokio::spawn(async move {
                    match tokio::io::copy_bidirectional(&mut client, &mut upstream).await {
                        Ok(_) => {
                            trace!("Upstream connection closed");
                        }
                        Err(err) => {
                            warn!("Upstream connection failed: {}", err);
                        }
                    }
                });

                Ok(())
            }
            .await;

            if let Err(err) = result {
                eprintln!("Error: {err:?}");
            }
        }
    }
}
