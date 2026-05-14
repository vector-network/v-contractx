//! v-contractx: contract runtime and projection framework for Vector Network.
//!
//! The kernel remains deterministic by using integer accounting, append-only
//! records, and explicit projection / reconstruction receipts.

pub mod access;
pub mod certification;
pub mod contract;
pub mod errors;
pub mod projection;
pub mod reconstruction;
pub mod record;
pub mod runtime;
pub mod types;
pub mod utils;

pub use access::*;
pub use certification::*;
pub use contract::*;
pub use errors::*;
pub use projection::*;
pub use reconstruction::*;
pub use record::*;
pub use runtime::*;
pub use types::*;
