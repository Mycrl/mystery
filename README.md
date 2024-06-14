<!--lint disable no-literal-urls-->
<div align="center">
  <img src="./logo.svg" width="200px"/>
</div>
<br/>
<div align="center">
  <strong>TURN Server implemented by ❤️ Rust</strong>
</div>
<div align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/mycrl/turn-rs/tests.yml?branch=main"/>
  <img src="https://img.shields.io/github/license/mycrl/turn-rs"/>
  <img src="https://img.shields.io/github/issues/mycrl/turn-rs"/>
  <img src="https://img.shields.io/github/stars/mycrl/turn-rs"/>
</div>
<div align="center">
  <sup>RFC: https://datatracker.ietf.org/doc/html/rfc8656</sup>
</div>
<br/>
<br/>

A pure Rust implementation of the turn server. Compared with coturn, the advantage is to provide better performance. Single-threaded decoding speed up to 5Gib/s, forwarding delay is less than 35 microseconds. However, it does not provide as rich as coturn feature support, this project is more focused on the core business, do not need to touch the complex configuration items, almost out of the box.

## How do I choose?

#### turn-rs

If you are not familiar with coturn configuration items and are annoyed by the complexity of coturn configuration items, then you should use this project, and similarly, if you want better performance performance and lower memory footprint, you can also use this project. turn-rs configuration is easy, and the external api is very simple, and is good enough for core business support.

#### coturn

If you have extensive standard support requirements for turn servers and need more integrated services and ecological support, then you should choose coturn.


## Table of contents

* [features](#features)
* [usage](#usage)
  * [docker](#docker)  
  * [linux service](#linux-service)
* [building](#building)


## Features

- Only long-term authentication mechanisms are used.
- Static authentication lists can be used in configuration files.
- Only virtual ports are always allocated and no real system ports are occupied.
- The transport layer supports tcp and udp protocols, and supports binding multiple network cards or interfaces.
- The REST API can be used so that the turn server can proactively notify the external service of events and use external authentication mechanisms, and the external can also proactively control the turn server and manage the session.

## Usage

First, Get the compiled binaries from [github release](https://github.com/mycrl/turn-rs/releases).

Start with configuration file:

```bash
turn-server --config=/etc/turn-server/config.toml
```

Please check the example configuration file for details: [turn-server.toml](./turn-server.toml)


#### Docker

```bash
docker pull ghcr.io/mycrl/turn-server
```
The custom configuration file overrides the `/etc/turn-server/config.toml` path inside the image through `-v`.

#### Linux service

```
./install-service.sh
```

This will compile the project and install and start the service.


## Building

#### Prerequisites

You need to install the Rust toolchain, if you have already installed it, you can skip it, [Install Rust](https://www.rust-lang.org/tools/install), then get the source code:

```bash
git clone https://github.com/mycrl/turn-rs
```

#### Build workspace

Compile the entire workspace in release mode:

```bash
cd turn-rs
cargo build --release
```

After the compilation is complete, you can find the binary file in the `"target/release"` directory.


## License

[GPL3.0](./LICENSE)
Copyright (c) 2022 Mr.Panda.
