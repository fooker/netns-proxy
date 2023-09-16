use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug)]
pub struct Proxy {}

#[async_trait]
impl super::Proxy for Proxy {
    async fn listen(bind: SocketAddr) -> Result<Self> {
        unimplemented!()
    }

    async fn run(mut self: Box<Self>, target: SocketAddr) -> Result<()> {
        todo!()
    }
}