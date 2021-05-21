use std::collections::HashMap;
use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;
use tokio::net::UdpSocket;

pub struct Proxy {
    socket: UdpSocket,

    conn_track: HashMap<SocketAddr, UdpSocket>,
}

#[async_trait]
impl super::Proxy for Proxy {
    async fn listen(bind: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(bind).await?;

        return Ok(Self {
            socket,
            conn_track: HashMap::new(),
        });
    }

    async fn run(mut self: Box<Self>, target: SocketAddr) -> Result<()> {
        unimplemented!()
    }
}