//! Framework-specific adapters for Heisenberg
//!
//! This module provides helper functions and utilities for integrating Heisenberg
//! with specific web frameworks that don't use the Tower ecosystem.

#[cfg(feature = "actix")]
pub mod actix;

#[cfg(feature = "rocket")]
pub mod rocket;
