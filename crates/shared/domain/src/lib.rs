//! # Domain Models
//!
//! This crate contains pure domain types with minimal dependencies (`serde`, `bitflags`).
//! Keep it lean: no I/O, networking, or heavy logic—just data and simple helpers.

pub mod config;
pub mod constants;
pub mod entity;
pub mod features;
