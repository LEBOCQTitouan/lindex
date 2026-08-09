#![forbid(unsafe_code)]
//! Application layer — use cases and PORTS (traits). Depends only on `domain`.
//! Adapters (outer) implement these ports; the composition roots wire them.

pub mod ports;
pub mod usecase;
