//! Cloud integration module for syncing secrets to cloud platforms.
//!
//! # Supported Platforms
//!
//! - Vercel (via Vercel CLI)

pub mod vercel;

pub use vercel::{detect_project_id, push_secrets_to_vercel};
