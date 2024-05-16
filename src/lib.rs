#![allow(clippy::bool_assert_comparison)]

pub mod fast;
pub mod flatbin;
pub mod slow;
pub mod binc;
mod tests;
pub mod ty;

pub type JsonValue = serde_json::Value;
