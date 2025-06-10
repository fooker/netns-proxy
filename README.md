# netns-proxy

A simple and slim proxy to forward ports from and into linux network
namespaces.


## Overview

`netns-proxy` is a plain and simple proxy for TCP, UDP and SCTP that allows to
accept connections in one network namespace and forwards them into another.

This allows to expose ports that are only accessible in one namespace to a
second one. This provides a lightweight alternative to connecting namespace with
`veth` interfaces which require address management and firewall rules for
forwarding.

`netns-proxy` does its job by opening a socket when it is started and switches
to the target namespace before opening connections to the exposed server. This
allows the accepting socket to exist in the original namespace while forwarded
connection happen in the specified namespace.


## Installation

You can install the latest released version directly from [crates.io]:

```bash
cargo install netns-proxy
```


## How to Build

### Prerequisites

* Rust toolchain (rustc ≥ 1.60, cargo)
* Linux with setns(2) support (kernel ≥ 3.0)
* (Optional) Nix if you prefer reproducible builds

### From Source

```bash
# Clone the source
git clone https://github.com/fooker/netns-proxy.git
cd netns-proxy

cargo build --release
```

The binary will be in `target/release/netns-proxy`.

### With nix

Requires [nix](https://nixos.org/).

```bash
# Clone the source
git clone https://github.com/fooker/netns-proxy.git
cd netns-proxy

nix-build
```


## Usage

```
netns-proxy [OPTIONS] <netns> <target>
```

* `<netns>` is the name of the network namespace from which forwarded connections
are created.
* `<target>` the target host and port to forward connections to.

See `netns-proxy --help` for full details and options.


## Getting Help

Feel free to open an [Issue](https://github.com/fooker/netns-proxy/issues) or
write me a [Mail](mailto:fooker@lab.sh).


## Contributing

:+1: Thanks for your help to improve the project! Pleas don't hesitate to
create an [Issue](https://github.com/fooker/netns-proxy/issues) if you find
something is off or even consider creating a patch and propose a Pull-Request.


## License

The project is licensed under the [MIT license](./LICENSE).
