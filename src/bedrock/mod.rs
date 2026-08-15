//! Bedrock protocol packet handling
//!
//! The module handles Minecraft Bedrock Edition protocol packets,
//! including login and game packets.

pub mod batch;
pub mod login;
pub mod responses;
pub mod version;

pub use batch::{compress_batch, decompress_batch, parse_batch};
pub use responses::*;
pub use version::{
    get_latest_protocol, get_latest_version, resolve_version, set_fallback_protocol,
    set_fallback_version,
};
