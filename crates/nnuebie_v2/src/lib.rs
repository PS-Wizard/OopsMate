#[cfg(not(target_arch = "x86_64"))]
compile_error!("nnuebie_v2 requires x86_64");

pub mod context;
pub mod eval;
pub mod network;
pub mod oopsmate_core;

mod aligned;
mod arch;
mod constants;
mod features;
mod finny;
mod layers;
mod layout;
mod loader;
mod simd256;
mod transform;
mod update;

pub use context::NnueContext;
pub use eval::EvalOutput;
pub use network::{NnueNetworks, PositionInputs};
