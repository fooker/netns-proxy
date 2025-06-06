use std::fmt;
use std::fmt::Debug;
use std::net::{Ipv6Addr, SocketAddr};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::{Parser, ValueEnum};
use nix::sched::{setns, CloneFlags};
use nix::unistd::{setgid, setuid, Gid, Uid};
use tracing::{debug, info, trace, Level};

mod sctp;
mod tcp;
mod udp;

#[async_trait]
pub trait Proxy: Debug {
    async fn listen(bind: SocketAddr) -> Result<Self>
    where
        Self: Sized;
    async fn run(self: Box<Self>, target: SocketAddr) -> Result<()>;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
#[allow(clippy::upper_case_acronyms)]
enum Proto {
    TCP,
    UDP,
    SCTP,
}

impl fmt::Display for Proto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Proto::TCP => "tcp",
            Proto::UDP => "udp",
            Proto::SCTP => "sctp",
        })
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[deny(missing_docs)]
struct Opts {
    /// The namespace to open connections from. Can be namespace name or full path
    netns: PathBuf,

    /// The network protocol to use
    #[clap(short, long, value_enum, default_value = "tcp")]
    proto: Proto,

    /// Target to forward requests to
    target: SocketAddr,

    /// Listen on incoming requests
    #[clap(short, long)]
    bind: Option<SocketAddr>,

    /// Verbose mode
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_max_level(match opts.verbose {
            0 => Level::WARN,
            1 => Level::INFO,
            2 => Level::DEBUG,
            3.. => Level::TRACE,
        })
        .init();

    // Open listening socket for proxy in current namespace
    let bind = opts
        .bind
        .unwrap_or_else(|| SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), opts.target.port()));
    info!("Listening on {}:{}", opts.proto, bind);

    let proxy: Box<dyn Proxy> = match opts.proto {
        Proto::TCP => Box::new(tcp::Proxy::listen(bind).await?),
        Proto::UDP => Box::new(udp::Proxy::listen(bind).await?),
        Proto::SCTP => Box::new(sctp::Proxy::listen(bind).await?),
    };
    trace!("Proxy created: {:?}", proxy);

    // Open network namespace path and switch over
    let netns = Path::new("/var/run/netns").join(opts.netns);
    debug!("Using netns: {}", netns.display());

    let netns = tokio::fs::File::open(&netns)
        .await
        .with_context(|| format!("Could not open network namespace file: {netns:?}"))?;

    setns(netns, CloneFlags::CLONE_NEWNET).context("Switching network namespace failed")?;
    trace!("Network namespace switched");

    // Dropping privileges
    setgid(Gid::current()).context("Failed to drop group privileges")?;
    trace!("Group privileges dropped");
    setuid(Uid::current()).context("Failed to drop user privileges")?;
    trace!("User privileges dropped");

    // Start forwarding incoming connections
    proxy.run(opts.target).await?;

    return Ok(());
}
