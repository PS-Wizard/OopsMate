//! Layer primitives used by the NNUE forward pass.

use std::io::{self, Read};

mod activations;
mod affine;
mod sparse;

pub use activations::{ClippedReLU, SqrClippedReLU};
pub use affine::AffineTransform;
pub use sparse::AffineTransformSparseInput;

/// Common interface for NNUE layers and activations.
pub trait Layer {
    type Input;
    type Output;

    /// Propagates one layer worth of values into `output`.
    fn propagate(&self, input: &[Self::Input], output: &mut [Self::Output]);
    /// Reads serialized layer parameters from an `.nnue` stream.
    fn read_parameters<R: Read>(&mut self, reader: &mut R) -> io::Result<()>;
}
