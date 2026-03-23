//! phxtool — high-level operations for Halo Wars game assets.
//!
//! Provides orchestration logic on top of the `ensemble-formats` crates
//! (era, xmb, ugx, ddx, ecf, etc.) to implement the workflows found in
//! KornnerStudios' PhxTool.

pub mod era_ops;
pub mod ugx_ops;
pub mod wwise_ops;
pub mod xmb_ops;

mod error;
pub use error::{Error, Result};
