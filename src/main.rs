use std::net::{Ipv6Addr, SocketAddr};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::{AppSettings, Clap};
use nix::sched::{CloneFlags, setns};
use nix::unistd::{Gid, setgid, setuid, Uid};

mod tcp;
mod udp;
mod sctp;

#[async_trait]
pub trait Proxy {
    async fn listen(bind: SocketAddr) -> Result<Self> where Self: Sized;
    async fn run(self: Box<Self>, target: SocketAddr) -> Result<()>;
}

#[derive(Clap, Debug)]
enum Proto {
    TCP,
    UDP,
    SCTP,
}

#[derive(Clap, Debug)]
#[clap(setting = AppSettings::ColoredHelp)]
#[clap(author, about, version)]
#[deny(missing_docs)]
struct Opts {
    /// The namespace to open connections from. Can be namespace name or full path
    netns: PathBuf,

    /// The network protocol to use
    #[clap(short, long, arg_enum, default_value = "tcp")]
    proto: Proto,

    /// Target to forward requests to
    target: SocketAddr,

    /// Listen on incoming requests
    #[clap(short, long)]
    bind: Option<SocketAddr>,

    /// Verbose mode
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    // Open listening socket for proxy in current namespace
    let bind = opts.bind.unwrap_or_else(|| SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), opts.target.port()));
    let proxy: Box<dyn Proxy> = match opts.proto {
        Proto::TCP => Box::new(tcp::Proxy::listen(bind).await?),
        Proto::UDP => Box::new(udp::Proxy::listen(bind).await?),
        Proto::SCTP => Box::new(sctp::Proxy::listen(bind).await?),
    };

    // Open network namespace path and switch over
    let netns = Path::new("/var/run/netns").join(opts.netns);
    let netns = tokio::fs::File::open(&netns).await
        .with_context(|| format!("Could not open network namespace file: {:?}", netns))?;
    let netns = netns.as_raw_fd();

    setns(netns, CloneFlags::CLONE_NEWNET)
        .context("Switching network namespace failed")?;

    // Dropping privileges
    setgid(Gid::current())
        .context("Failed to drop group privileges")?;
    setuid(Uid::current())
        .context("Failed to drop group privileges")?;

    // Start forwarding incoming connections
    proxy.run(opts.target).await?;

    return Ok(());
}
