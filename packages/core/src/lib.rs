//! Shadow Secret - A secure, distributed secret management system.
//!
//! This library provides secure secret loading from SOPS-encrypted files
//! with strict guarantees about memory-only operations.

pub mod vault;
pub mod injector;
pub mod cleaner;
pub mod config;
pub mod init;
pub mod cloud;
