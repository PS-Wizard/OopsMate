mod board;
mod evaluate;
mod moves;

use crate::accumulator_stack::AccumulatorStack;
use crate::finny_tables::FinnyTables;
use crate::network::NnueNetworks;
use crate::network::ScratchBuffer;
use crate::types::{Piece, Square};
use std::io;
use std::sync::Arc;

/// Stateful NNUE evaluator backed by shared immutable network weights.
pub struct NNUEProbe {
    pub(super) networks: Arc<NnueNetworks>,
    pub(super) scratch_big: ScratchBuffer,
    pub(super) scratch_small: ScratchBuffer,
    pub(super) pieces: [Piece; 64],
    pub(super) king_squares: [Square; 2],
    pub(super) piece_count: usize,
    pub(super) pawn_count: [i32; 2],
    pub(super) non_pawn_material: [i32; 2],
    pub(super) by_color_bb: [u64; 2],
    pub(super) by_type_bb: [u64; 6],
    pub(super) accumulator_stack: AccumulatorStack,
    pub(super) finny_tables: FinnyTables,
}

impl NNUEProbe {
    /// Loads both networks from disk and creates a probe around them.
    pub fn new(big_path: &str, small_path: &str) -> io::Result<Self> {
        let networks = Arc::new(NnueNetworks::new(big_path, small_path)?);
        Ok(Self::from_networks(networks))
    }

    /// Builds a probe from already-loaded shared network weights.
    pub fn from_networks(networks: Arc<NnueNetworks>) -> Self {
        let scratch_big = ScratchBuffer::new(networks.big_net.feature_transformer.half_dims);
        let scratch_small = ScratchBuffer::new(networks.small_net.feature_transformer.half_dims);

        let mut finny_tables = FinnyTables::new();
        finny_tables.clear(
            &networks.big_net.feature_transformer.biases,
            &networks.small_net.feature_transformer.biases,
        );

        Self {
            networks,
            scratch_big,
            scratch_small,
            pieces: [Piece::None; 64],
            king_squares: [0; 2],
            piece_count: 0,
            pawn_count: [0; 2],
            non_pawn_material: [0; 2],
            by_color_bb: [0; 2],
            by_type_bb: [0; 6],
            accumulator_stack: AccumulatorStack::new(),
            finny_tables,
        }
    }

    pub fn with_networks(networks: Arc<NnueNetworks>) -> io::Result<Self> {
        Ok(Self::from_networks(networks))
    }
}
