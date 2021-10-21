<!--lint disable no-literal-urls-->
<div align="center">
  <img 
    alt="mystery"
    src="./logo.svg" 
    width="200px"
  />
</div>
<br/>
<div align="center">
  <strong>WebRTC Server implemented by ❤️ Rust + Node.js</strong>
</div>
<div align="center">
  <img src="https://img.shields.io/github/workflow/status/mycrl/mystery/cargo-test"/>
  <img src="https://img.shields.io/github/license/mycrl/mystery"/>
  <img src="https://img.shields.io/github/issues/mycrl/mystery"/>
  <img src="https://img.shields.io/github/stars/mycrl/mystery"/>
</div>
<br/>
<br/>

mystery is a WebRTC server solution implemented using Rust and supports the SFU/MCU model. Compared with other ongoing projects, the current project prioritizes WebRTC one-to-many live broadcasting, but this does not mean that the project will give up peer-to-peer two-way dialogue.

## Table of contents

* [roadmap](#roadmap)
* [building](#building)
  * [prerequisites](#prerequisites)
  * [build workspace](#build-workspace)
  * [docker compose](#docker-compose)

## Roadmap

Important: The project was developed by myself. This is just my side project, so the development progress will be slower. If you are looking for the mature and highly supported webrtc component of rust instead of the media control center implementation, you can follow this project: [webrtc.rs](https://webrtc.rs/)

### Base protocols support: 

* [x] [turn](https://github.com/mycrl/mystery/tree/dev/bin/turn) (add support for session node grouping)
* [x] [stun](https://github.com/mycrl/mystery/tree/dev/stun) (superfast parser! The throughput of a single thread is as high as 3Gib/s! 30 million stun packets can be processed in one second!)
* [x] [rtp](https://github.com/mycrl/mystery/tree/dev/rtp) (lock the rtp version to rfc3550)
* [x] [sdp](https://github.com/mycrl/mystery/tree/dev/sdp) (partial support of the protocol)
* [ ] [doing] [rtcp](https://github.com/mycrl/mystery/tree/dev/rtcp)
* [ ] [srtp](https://github.com/mycrl/mystery/tree/dev/srtp)
* [ ] [srtcp](https://github.com/mycrl/mystery/tree/dev/srtcp)
* [ ] [doing] [dtls](https://github.com/mycrl/mystery/tree/dev/dtls) (the encryption process is not clear)

### Peripheral components:

* [ ] [ice](https://github.com/mycrl/mystery/tree/dev/ice)
* [ ] [sfu](https://github.com/mycrl/mystery/tree/dev/sfu)
* [ ] [mcu](https://github.com/mycrl/mystery/tree/dev/mcu)
* [ ] [control](https://github.com/mycrl/mystery/tree/dev/control) (node.js driver, cluster control center)
* [ ] [media codec](https://github.com/mycrl/mystery/tree/dev/codec) (ffmpeg or intel media sdk?)


## Building

### Prerequisites

You need to install the Rust toolchain, if you have already installed it, you can skip it, [Install Rust](https://www.rust-lang.org/tools/install), then get the source code:

```bash
git clone https://github.com/mycrl/mystery
```

And, you need to install the openssl toolchain.

#### Windows

If you have [chocolatey](https://chocolatey.org/install) installed you can install openssl via a single command i.e.

```bash
choco install openssl
```

#### Linux

```bash
sudo apt-get install libssl-dev
```

#### Macos

```bash
brew install openssl
```

### Build workspace

Compile the entire workspace in release mode:

```bash
cd mystery
cargo build --release
```

After the compilation is complete, you can find the binary file in the "target/release" directory.

### Docker compose

Use docker-compose to start all services:

```bash
cd mystery
docker-compose up -d
```

## Code style

The coding style of this project may not conform to the community style or the habits of most people, but it conforms to my own style. I have paranoid requirements for the code format, I know this is a bad habit, and the current project is also independently developed and maintained by me. If you have more suggestions, you can tell me.

## License

[GPL](./LICENSE)
Copyright (c) 2020 Mr.Panda.
