//! Stateful NNUE probe implementation.

mod board;
mod evaluate;
mod moves;

use crate::accumulator_stack::AccumulatorStack;
use crate::finny_tables::FinnyTables;
use crate::network::NnueNetworks;
use crate::network::ScratchBuffer;
use crate::types::{Piece, Square};
use std::io;

enum NetworkHandle<'a> {
    Borrowed(&'a NnueNetworks),
    Owned(Box<NnueNetworks>),
}

impl NetworkHandle<'_> {
    #[inline(always)]
    fn as_ref(&self) -> &NnueNetworks {
        match self {
            Self::Borrowed(networks) => networks,
            Self::Owned(networks) => networks,
        }
    }
}

/// Stateful NNUE evaluator backed by immutable network weights.
pub struct NNUEProbe<'a> {
    networks: NetworkHandle<'a>,
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

impl NNUEProbe<'static> {
    /// Loads both networks from disk and creates a probe around them.
    pub fn new(big_path: &str, small_path: &str) -> io::Result<Self> {
        let networks = Box::new(NnueNetworks::new(big_path, small_path)?);
        Ok(Self::from_handle(NetworkHandle::Owned(networks)))
    }
}

impl<'a> NNUEProbe<'a> {
    /// Builds a probe from already-loaded immutable network weights.
    pub fn from_networks(networks: &'a NnueNetworks) -> Self {
        Self::from_handle(NetworkHandle::Borrowed(networks))
    }

    fn from_handle(networks: NetworkHandle<'a>) -> Self {
        let networks_ref = networks.as_ref();
        let scratch_big = ScratchBuffer::new(networks_ref.big_net.feature_transformer.half_dims);
        let scratch_small =
            ScratchBuffer::new(networks_ref.small_net.feature_transformer.half_dims);

        let mut finny_tables = FinnyTables::new();
        finny_tables.clear(
            &networks_ref.big_net.feature_transformer.biases,
            &networks_ref.small_net.feature_transformer.biases,
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
}
