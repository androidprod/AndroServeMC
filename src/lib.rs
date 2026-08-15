//! AndroServeMC: Minecraft Bedrock Edition Server implemented in Rust
//!
//! This library provides a complete Rust implementation of a Minecraft
//! Bedrock Edition server built on RakNet + the Bedrock Protocol.
//!
//! # Features
//! - RakNet protocol implementation
//! - Bedrock protocol packet handling
//! - Async/await networking with tokio
//! - JWT and cryptographic operations
//! - Cross-platform support (Windows, Linux, macOS)
//! - In-game chat and player session management

pub mod bedrock;
pub mod crypto;
pub mod error;
pub mod network;
pub mod raknet;
pub mod util;

pub use error::{Error, Result};

/// Initialize the AndroServeMC library with numeric log verbosity.
pub fn init_with_verbosity(verbosity: u8) {
    util::logger::init_with_verbosity(verbosity);
}
