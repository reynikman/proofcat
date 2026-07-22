//! Pure Rust offload core.
//!
//! The audited implementation has one source of truth and no Tauri dependency.
//! The desktop crate imports this crate as an adapter; the headless CLI imports
//! the same public module.

#[path = "../../../src-tauri/src/offload/mod.rs"]
pub mod offload;
