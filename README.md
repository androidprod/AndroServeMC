<div align="center">

<img src="res/app.ico" width="80" alt="AndroServeMC Logo" />

# AndroServeMC - Rust Edition

**A Minecraft Bedrock Edition Server implemented from scratch in Rust, built on a RakNet + Bedrock protocol stack**

[![Rust Edition](https://img.shields.io/badge/Edition-Rust-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow?style=flat-square)](LICENSE)
[![Async Runtime](https://img.shields.io/badge/Runtime-Tokio-blue?style=flat-square)](https://tokio.rs/)
[![Platform](https://img.shields.io/badge/Platform-Win%20%7C%20Linux%20%7C%20macOS-lightgrey?style=flat-square)](#building)
[![Protocol](https://img.shields.io/badge/Bedrock%20Protocol-Dynamic-green?style=flat-square)](#configuration)

**English** | [日本語](README.ja.md)

</div>

---

## Overview

**AndroServeMC** implements the Minecraft Bedrock Edition network stack from the ground up — from raw UDP sockets through the RakNet handshake to Bedrock Login parsing, the game-start sequence, chunk data, and in-game chat.

The Rust edition focuses on:
- **Performance**: Async/await with Tokio; a background resend/congestion-control worker handles reliable frame retransmission
- **Safety**: Rust's ownership and type system eliminate buffer overflow and use-after-free bug classes
- **Portability**: A single codebase that builds on Windows, Linux, and macOS
- **Maintainability**: Modular design split into `bedrock`, `raknet`, `network`, `crypto`, and `util` crates

> ⚠️ **Disclaimer**: This is an unofficial server implementation for research and testing. Offline-mode (unencrypted) clients only; it is not intended for commercial use or public deployment. Use at your own risk.

---

## Features

### ✅ Implemented

| Category | Feature |
|----------|---------|
| **RakNet** | Unconnected Ping/Pong (MOTD with version string) |
| | Open Connection Request/Reply 1 & 2 |
| | Connection Request / Connection Request Accepted |
| | Connected Ping/Pong |
| | Frame Set packet parsing & reliable-frame splitting |
| | ACK/NAK handling with retransmission & congestion control |
| **Bedrock** | `Login` packet reception & parsing (multiple format variants, including new preview protocols) |
| | zlib raw deflate compression / decompression |
| | JWT token parsing (chain data extraction) |
| | Player name & device metadata extraction |
| | `NetworkSettings` response sent back to the client |
| **Gameplay** | `PlayStatus(Success)` + `ResourcePacksInfo` login flow |
| | `ResourcePackStack` / `StartGame` / `BiomeDefinitionList` / `AvailableEntityIdentifiers` |
| | Flat Overworld chunks around spawn |
| | `ChunkRadiusUpdated` + `NetworkChunkPublisherUpdate` |
| | In-game chat (text packet broadcast) |
| | Join / leave messages |
| **Infrastructure** | Tokio async runtime for concurrent connections |
| | Per-client session state management |
| | External IP discovery via STUN |
| | C++-style colored logging (tracing framework) |
| | Protocol → version table with automatic fallback for unknown/newer protocols |
| | `config.jsonc` (JSONC, comments supported) for configuration |
| | clap-based CLI with Windows-style `/flag` argument support |
| **Build** | Cargo cross-platform build |
| | Windows icon embedding via `build.rs` (`winres`) |

### 🔲 Not Yet Implemented

| Feature | Notes |
|---------|-------|
| Encrypted connection support | Required for Xbox Live online sessions |
| Persistence | World saving / loading |
| Entities & world generation | Basic flat chunks only |
| Command system | `/` commands not yet implemented |

---

## How It Works

```
[BE Client connects]
        │
        ▼
[RakNet Handshake]
  Unconnected Ping → Pong (MOTD + version)
  OCR1 → OCReply1
  OCR2 → OCReply2
  ConnectionRequest → ConnectionRequestAccepted
  (reliable frames, ACK/NAK + retransmission)
        │
        ▼
[Bedrock Layer]
  NetworkSettings request → NetworkSettings response (zlib)
  Login packet received (zlib compressed)
  Chain Data (JWT) → player name
  PlayStatus(Success) + ResourcePacksInfo
  ResourcePackClientResponse → ResourcePackStack
  ResourcePackClientResponse (Completed) → StartGame / Biomes / Entities
        │
        ▼
[In Game]
  RequestChunkRadius → flat Overworld chunks + publisher update
  SetLocalPlayerAsInitialized → "joined" broadcast
  Text packets → chat broadcast to all players
```

---

## Project Structure

```
src/
├── main.rs               # Binary entry point (CLI, config, STUN, server loop)
├── lib.rs                # Library root & public module exports
├── cli.rs                # clap CLI definitions (--logs, --filter, --debug, ...)
├── error.rs              # Error types & Result alias
├── bedrock/              # Minecraft Bedrock protocol
│   ├── mod.rs           # Re-exports & latest version/protocol lookup
│   ├── version.rs       # Protocol ↔ version table + fallback for unknown protocols
│   ├── login.rs         # Login packet parsing & JWT chain/player extraction
│   ├── batch.rs         # Batch packet handling
│   └── responses.rs     # Outgoing responses (NetworkSettings, StartGame, ...)
├── raknet/               # RakNet protocol implementation
│   ├── mod.rs           # RakNet structures & configuration
│   ├── constants.rs     # Packet IDs & constants
│   ├── protocol.rs      # Frame/datagram types
│   └── server/          # RakNet server
│       ├── mod.rs       # Server state, packet routing, resend worker
│       ├── session.rs   # Per-client session state
│       ├── handshake.rs # Unconnected PING/PONG, OCR1/2 replies
│       ├── frames.rs    # Frame set encode/parse, reliability, splitting
│       └── bedrock.rs   # Bedrock packet handling (login, chat, chunks)
├── crypto/               # Cryptographic utilities
│   ├── mod.rs
│   └── jwt.rs           # JWT parsing & Base64 decoding
├── network/              # Network abstraction
│   ├── mod.rs           # Network config & wrapper
│   └── udp.rs           # UDP socket (async Tokio)
└── util/                 # Utilities
    ├── mod.rs
    ├── buffer.rs        # Binary buffer (little/big endian)
    ├── config.rs        # config.jsonc load/save & JSONC comment stripping
    ├── logger.rs        # C++-style colored tracing formatter
    └── stun.rs          # STUN external IP discovery

build.rs                 # Windows icon embedding
res/
└── app.ico              # Windows application icon
```

---

## Building

### Prerequisites

- **Rust**: 1.70 or later (install from [rustup.rs](https://rustup.rs))
- **Cargo**: Included with Rust

### Windows / Linux / macOS

```bash
cargo build --release
```

- Windows output: `target/release/androservemc.exe` (icon embedded)
- Linux/macOS output: `target/release/androservemc`

The `windows` and `winres` dependencies are used **only on Windows**; other platforms build without them.

### Build Options

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (optimized, smaller binary)
cargo build --release
```

---

## Usage

```
Usage: androservemc [OPTIONS]

Options:
  -c, --config          Enable loading version/protocol from config.jsonc
                        (default; kept for CLI compatibility)
      --filter <NAME>   Filter players by name (substring match)
      --logs [<level>]  Set log level: 0=error, 1=warn, 2=info, 3=debug, 4=trace.
                        Without a value this enables debug verbosity (3)
  -d, --debug           Backward-compatible alias for --logs 3
  -h, --help            Print help
  -V, --version         Print version
```

Windows-style `/flag` arguments are also accepted (e.g. `/help`, `/logs 3`).

### Running the Server

```bash
# Start with default settings (info-level logs)
./androservemc
# or on Windows
androservemc.exe

# Debug verbosity
./androservemc --logs
./androservemc --logs 3
./androservemc --debug        # same as --logs 3

# Full trace output
./androservemc --logs 4

# Only log errors
./androservemc --logs 0
```

Default log level is `info` (2).

### First Run

On first execution a `config.jsonc` is auto-generated **in the same directory as the binary**:

```jsonc
{
  // Minecraft Bedrock version string shown in the server list.
  "version": "1.26.21",
  // Bedrock protocol number. Keep this aligned with the client version.
  "protocol": 975,
  // UDP port to listen on.
  "port": 19132,
  // Local bind address. 0.0.0.0 listens on all network interfaces.
  "bind_addr": "0.0.0.0"
}
```

**Example paths**:
- Windows: `C:\path\to\androservemc.exe` → `C:\path\to\config.jsonc`
- Linux: `/usr/local/bin/androservemc` → `/usr/local/bin/config.jsonc`

### Configuration

| Setting | Type | Default | Purpose |
|---------|------|---------|---------|
| `port` | int | 19132 | UDP listen port (must be reachable) |
| `bind_addr` | string | "0.0.0.0" | Bind address (0.0.0.0 = all interfaces) |
| `protocol` | int | 975 | Bedrock protocol number |
| `version` | string | "1.26.21" | Version string shown in the server list / MOTD |

JSONC comments are supported. Missing/invalid `protocol`/`version` values are automatically normalized to the latest supported version.

### Connecting from Minecraft

1. Open Minecraft Bedrock Edition → **Play** → **Servers** (or **LAN**)
2. Add server: `127.0.0.1` (or your machine's IP)
3. Join — the server walks you through the login flow and spawns you on a flat overworld
4. Use the chat to talk with other connected players

### Logging

The colored log format matches the original C++ tool: `[HH:MM:SS] [LEVEL] message`.

| Level | `--logs` value | Output |
|-------|----------------|--------|
| error | 0 | Errors only |
| warn | 1 | + warnings (e.g. unknown protocol fallback) |
| info | 2 (default) | Startup status, joins, chat messages |
| debug | 3 | Packet-level details |
| trace | 4 | Full frames & low-level traces |

---

## Configuration

### Bedrock Version Protocol Numbers

`protocol` in `config.jsonc` should match your client version. The full mapping table lives in `src/bedrock/version.rs` and covers every version from 0.14.3 (protocol 70) to the latest:

| Bedrock Version | Protocol | Notes |
|-----------------|----------|-------|
| 1.20.0–1.21.0   | 589–685  | 1.20 line |
| 1.21.2–1.21.50  | 686–766  | 1.21 line |
| 1.21.60–1.21.124 | 776–860  | 1.21 line |
| 1.21.130        | 898      | |
| 1.26.0          | 924      | |
| 1.26.10         | 944      | |
| 1.26.21         | 975      | Default / latest known |

Unknown or newer protocols (e.g. preview builds) are not rejected: the server logs a warning (`Unknown Bedrock protocol <N> - falling back to ...`) and continues with the latest known version.

---

## Networking Notes

- Listens on UDP (default port **19132**). Make sure the port is open on your firewall and, if connecting over the internet, forwarded on your router.
- On startup the tool performs a **STUN** request to discover the external IP, which is included in the RakNet handshake.
- Since no encryption is negotiated, only **offline-mode** clients can connect. This server cannot accept Xbox Live (encrypted) online sessions — it is for research/testing only.

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| **tokio** | Async runtime |
| **clap** | CLI parsing (derive) |
| **serde** / **serde_json** | Configuration serialization |
| **base64** | JWT payload decoding |
| **flate2** | zlib compression / decompression |
| **tracing** / **tracing-subscriber** | Structured, colored logging |
| **chrono** | Timestamps for the log formatter |
| **once_cell** | Lazy statics (protocol table) |
| **rand** | GUID generation |
| **thiserror** / **anyhow** | Error handling |
| **windows** | Console virtual-terminal support (Windows only) |
| **winres** | Icon embedding at build time (Windows only) |

---

## Troubleshooting

### Port Already in Use

**Error**: `Address already in use`

**Solution**:
1. Change `port` in `config.jsonc`
2. Or find and stop the conflicting process:
   ```bash
   # Windows PowerShell
   Get-NetTCPConnection -LocalPort 19132

   # Linux
   lsof -i :19132
   netstat -tlnp | grep 19132
   ```

### Clients Can't Connect

**Check**:
1. `port` in config is correct and the server reports it is listening
2. Firewall allows inbound UDP on the port
3. Try connecting from the same machine first (`127.0.0.1`)
4. Use `--logs 3` and look for handshake packets in the output

### Warning "Unknown Bedrock protocol"

The client uses a protocol number that is not in the table. The server automatically falls back to the latest known version (1.26.21 / protocol 975) and continues. Update `protocol` in `config.jsonc` if you want a different target.

### Client Disconnects After "Logging In"

Check the `--logs 3` output. The login flow must complete every step (NetworkSettings → Login → ResourcePacksInfo → ResourcePackStack → StartGame). Missing or malformed packets in any stage will cause the client to disconnect.

---

## Development

```bash
# Run in debug mode with full logging
cargo run -- --logs 3

# Run tests (library + CLI)
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

---

## References

- [Mojang/bedrock-protocol-docs](https://github.com/Mojang/bedrock-protocol-docs)
- [PrismarineJS/bedrock-protocol](https://github.com/PrismarineJS/bedrock-protocol)
- [Sandertv/go-raknet](https://github.com/Sandertv/go-raknet)
- [RakNet Documentation](https://github.com/facebookarchive/RakNet)
- [wiki.vg/Bedrock Protocol](https://minecraft.wiki/w/Bedrock_Edition_protocol)
- [Tokio Guide](https://tokio.rs/)

---

## License

[MIT License](LICENSE)

---

## Contributing

Contributions welcome! Areas of interest:

- Test suite expansion (packet-level fixtures)
- Protocol table updates for new Bedrock releases
- World generation & persistence
- Command system
- Encrypted connection support
- Performance optimization

Please submit PRs with a description and, where relevant, benchmark comparisons.
