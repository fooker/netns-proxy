use std::net::SocketAddr;

use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};

pub struct Proxy {
    socket:  TcpListener,
}

#[async_trait]
impl super::Proxy for Proxy {
    async fn listen(bind: SocketAddr) -> Result<Self> {
        let socket = TcpListener::bind(bind).await
            .with_context(|| format!("Failed to bind to TCP socket: {}", bind))?;

        return Ok(Self {
            socket
        })
    }

    async fn run(mut self: Box<Self>, target: SocketAddr) -> Result<()> {
        loop {
            let result: Result<()> = (|| async {
                let (mut client, _) = self.socket.accept().await
                    .context("Failed to accept connection")?;

                let mut remote = TcpStream::connect(target).await
                    .with_context(|| format!("Failed to connect to target: {}", target))?;

                tokio::io::copy_bidirectional(&mut client, &mut remote).await?;

                return Ok(());
            })().await;

            if let Err(err) = result {
                eprintln!("Error: {:?}", err);
            }
        }
    }
}