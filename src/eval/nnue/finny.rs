use crate::eval::nnue::constants::{BIG_HALF_DIMS, MAX_ACTIVE_FEATURES, PSQT_BUCKETS, SMALL_HALF_DIMS};
use crate::eval::nnue::features::feature_index_from_piece_code;
use crate::eval::nnue::network::FeatureTransformer;
use crate::eval::nnue::update::{
    accum_add, accum_add_into_both, accum_add_sub, accum_add1_sub1_into_both,
    accum_add1_sub2_into_both, accum_add2_sub1_into_both, accum_sub, accum_sub_into_both, psqt_add,
    psqt_add_into_both, psqt_add_sub, psqt_add1_sub1_into_both, psqt_add1_sub2_into_both,
    psqt_add2_sub1_into_both, psqt_sub, psqt_sub_into_both,
};
use crate::eval::nnue::compat::{Color, ColorExt, Piece, PieceExt, Position, PositionExt};

const FINNY_ENTRY_COUNT: usize = 64 * 2;
const PIECES: [Piece; 6] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

#[derive(Clone, Debug)]
struct BoardFacts {
    by_color: [u64; 2],
    by_piece: [u64; 6],
}

impl BoardFacts {
    #[inline(always)]
    fn from_position(position: &Position) -> Self {
        let board = position.board();

        Self {
            by_color: [board.color_bb(Color::White), board.color_bb(Color::Black)],
            by_piece: [
                board.piece_bb(Piece::Pawn),
                board.piece_bb(Piece::Knight),
                board.piece_bb(Piece::Bishop),
                board.piece_bb(Piece::Rook),
                board.piece_bb(Piece::Queen),
                board.piece_bb(Piece::King),
            ],
        }
    }
}

#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub(crate) struct FinnyEntry<const HALF_DIMS: usize> {
    accumulation: [i16; HALF_DIMS],
    psqt: [i32; PSQT_BUCKETS],
    by_color: [u64; 2],
    by_piece: [u64; 6],
}

impl<const HALF_DIMS: usize> FinnyEntry<HALF_DIMS> {
    const ZERO: Self = Self {
        accumulation: [0; HALF_DIMS],
        psqt: [0; PSQT_BUCKETS],
        by_color: [0; 2],
        by_piece: [0; 6],
    };

    #[inline(always)]
    fn clear(&mut self, biases: &[i16]) {
        debug_assert_eq!(biases.len(), HALF_DIMS);
        self.accumulation.copy_from_slice(biases);
        self.psqt.fill(0);
        self.by_color = [0; 2];
        self.by_piece = [0; 6];
    }
}

#[derive(Debug)]
pub(crate) struct FinnyCache<const HALF_DIMS: usize> {
    entries: Box<[FinnyEntry<HALF_DIMS>]>,
}

impl<const HALF_DIMS: usize> FinnyCache<HALF_DIMS> {
    #[inline(always)]
    fn new() -> Self {
        Self {
            entries: vec![FinnyEntry::ZERO; FINNY_ENTRY_COUNT].into_boxed_slice(),
        }
    }

    fn clear(&mut self, biases: &[i16]) {
        for entry in self.entries.iter_mut() {
            entry.clear(biases);
        }
    }

    #[inline(always)]
    fn entry_mut(
        &mut self,
        perspective: Color,
        king_square: crate::eval::nnue::compat::Square,
    ) -> &mut FinnyEntry<HALF_DIMS> {
        debug_assert!(king_square.is_valid());
        &mut self.entries[perspective.index() * 64 + king_square.index()]
    }
}

#[derive(Debug)]
pub(crate) struct FinnyTables {
    pub(crate) big: FinnyCache<{ BIG_HALF_DIMS }>,
    pub(crate) small: FinnyCache<{ SMALL_HALF_DIMS }>,
    big_bias_ptr: usize,
    small_bias_ptr: usize,
}

impl FinnyTables {
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            big: FinnyCache::new(),
            small: FinnyCache::new(),
            big_bias_ptr: 0,
            small_bias_ptr: 0,
        }
    }

    pub(crate) fn prepare(
        &mut self,
        big_transformer: &FeatureTransformer,
        small_transformer: &FeatureTransformer,
    ) {
        let big_bias_ptr = big_transformer.biases.as_ptr() as usize;
        if self.big_bias_ptr != big_bias_ptr {
            self.big.clear(&big_transformer.biases);
            self.big_bias_ptr = big_bias_ptr;
        }

        let small_bias_ptr = small_transformer.biases.as_ptr() as usize;
        if self.small_bias_ptr != small_bias_ptr {
            self.small.clear(&small_transformer.biases);
            self.small_bias_ptr = small_bias_ptr;
        }
    }
}

pub(crate) fn refresh_from_finny<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    position: &Position,
    cache: &mut FinnyCache<HALF_DIMS>,
    perspective: Color,
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    let board = position.board();
    let king_square = board.king_square(perspective);
    let entry = cache.entry_mut(perspective, king_square);
    let board_facts = BoardFacts::from_position(position);

    let mut removed = [0u32; MAX_ACTIVE_FEATURES];
    let mut added = [0u32; MAX_ACTIVE_FEATURES];
    let mut removed_len = 0usize;
    let mut added_len = 0usize;

    for color in [Color::White, Color::Black] {
        let color_idx = color.index();

        for piece in PIECES {
            let piece_idx = piece.index();
            let old_bb = entry.by_color[color_idx] & entry.by_piece[piece_idx];
            let new_bb = board_facts.by_color[color_idx] & board_facts.by_piece[piece_idx];

            let mut to_remove = old_bb & !new_bb;
            while to_remove != 0 {
                let square = to_remove.trailing_zeros() as u8;
                to_remove &= to_remove - 1;
                removed[removed_len] = feature_index_from_piece_code(
                    perspective,
                    crate::eval::nnue::compat::encode_piece(piece, color),
                    crate::eval::nnue::compat::Square::from_raw(square),
                    king_square,
                );
                removed_len += 1;
            }

            let mut to_add = new_bb & !old_bb;
            while to_add != 0 {
                let square = to_add.trailing_zeros() as u8;
                to_add &= to_add - 1;
                added[added_len] = feature_index_from_piece_code(
                    perspective,
                    crate::eval::nnue::compat::encode_piece(piece, color),
                    crate::eval::nnue::compat::Square::from_raw(square),
                    king_square,
                );
                added_len += 1;
            }
        }
    }

    apply_feature_deltas(
        feature_transformer,
        entry,
        &removed[..removed_len],
        &added[..added_len],
        accumulation,
        psqt,
    );

    entry.by_color = board_facts.by_color;
    entry.by_piece = board_facts.by_piece;
}

#[inline(always)]
fn apply_feature_deltas<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    entry: &mut FinnyEntry<HALF_DIMS>,
    removed: &[u32],
    added: &[u32],
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    if removed.is_empty() && added.is_empty() {
        accumulation.copy_from_slice(&entry.accumulation);
        psqt.copy_from_slice(&entry.psqt);
        return;
    }

    let shared = removed.len().min(added.len());
    let combine_last3 = removed.len().abs_diff(added.len()) == 1 && removed.len() + added.len() > 2;
    let shared_prefix = if combine_last3 || (shared != 0 && removed.len() == added.len()) {
        shared - 1
    } else {
        shared
    };

    for idx in 0..shared_prefix {
        apply_feature_add_sub(
            feature_transformer,
            added[idx] as usize,
            removed[idx] as usize,
            entry,
        );
    }

    let removed_tail = &removed[shared_prefix..];
    let added_tail = &added[shared_prefix..];

    if combine_last3 {
        if removed_tail.len() == 2 {
            debug_assert_eq!(added_tail.len(), 1);
            apply_feature_add1_sub2_into_both(
                feature_transformer,
                added_tail[0] as usize,
                removed_tail[0] as usize,
                removed_tail[1] as usize,
                entry,
                accumulation,
                psqt,
            );
        } else {
            debug_assert_eq!(added_tail.len(), 2);
            debug_assert_eq!(removed_tail.len(), 1);
            apply_feature_add2_sub1_into_both(
                feature_transformer,
                added_tail[0] as usize,
                added_tail[1] as usize,
                removed_tail[0] as usize,
                entry,
                accumulation,
                psqt,
            );
        }
        return;
    }

    if !removed_tail.is_empty() && !added_tail.is_empty() {
        debug_assert_eq!(removed_tail.len(), 1);
        debug_assert_eq!(added_tail.len(), 1);
        apply_feature_add1_sub1_into_both(
            feature_transformer,
            added_tail[0] as usize,
            removed_tail[0] as usize,
            entry,
            accumulation,
            psqt,
        );
        return;
    }

    if let Some((&last_removed, prefix_removed)) = removed_tail.split_last() {
        for &feature_index in prefix_removed {
            apply_feature_sub(feature_transformer, feature_index as usize, entry);
        }
        apply_feature_sub_into_both(
            feature_transformer,
            last_removed as usize,
            entry,
            accumulation,
            psqt,
        );
        return;
    }

    if let Some((&last_added, prefix_added)) = added_tail.split_last() {
        for &feature_index in prefix_added {
            apply_feature_add(feature_transformer, feature_index as usize, entry);
        }
        apply_feature_add_into_both(
            feature_transformer,
            last_added as usize,
            entry,
            accumulation,
            psqt,
        );
        return;
    }

    unreachable!("non-empty Finny delta must materialize a final update");
}

#[inline(always)]
fn apply_feature_add<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    feature_index: usize,
    entry: &mut FinnyEntry<HALF_DIMS>,
) {
    let weight_row = feature_index * HALF_DIMS;
    let psqt_row = feature_index * PSQT_BUCKETS;

    accum_add(
        &mut entry.accumulation,
        &feature_transformer.weights[weight_row..weight_row + HALF_DIMS],
    );
    psqt_add(
        &mut entry.psqt,
        &feature_transformer.psqt_weights[psqt_row..psqt_row + PSQT_BUCKETS],
    );
}

#[inline(always)]
fn apply_feature_add_into_both<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    feature_index: usize,
    entry: &mut FinnyEntry<HALF_DIMS>,
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    let weight_row = feature_index * HALF_DIMS;
    let psqt_row = feature_index * PSQT_BUCKETS;

    accum_add_into_both(
        &mut entry.accumulation,
        &feature_transformer.weights[weight_row..weight_row + HALF_DIMS],
        accumulation,
    );
    psqt_add_into_both(
        &mut entry.psqt,
        &feature_transformer.psqt_weights[psqt_row..psqt_row + PSQT_BUCKETS],
        psqt,
    );
}

#[inline(always)]
fn apply_feature_add_sub<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    add_index: usize,
    sub_index: usize,
    entry: &mut FinnyEntry<HALF_DIMS>,
) {
    let add_weight_row = add_index * HALF_DIMS;
    let add_psqt_row = add_index * PSQT_BUCKETS;
    let sub_weight_row = sub_index * HALF_DIMS;
    let sub_psqt_row = sub_index * PSQT_BUCKETS;

    accum_add_sub(
        &mut entry.accumulation,
        &feature_transformer.weights[add_weight_row..add_weight_row + HALF_DIMS],
        &feature_transformer.weights[sub_weight_row..sub_weight_row + HALF_DIMS],
    );
    psqt_add_sub(
        &mut entry.psqt,
        &feature_transformer.psqt_weights[add_psqt_row..add_psqt_row + PSQT_BUCKETS],
        &feature_transformer.psqt_weights[sub_psqt_row..sub_psqt_row + PSQT_BUCKETS],
    );
}

#[inline(always)]
fn apply_feature_add1_sub1_into_both<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    add_index: usize,
    sub_index: usize,
    entry: &mut FinnyEntry<HALF_DIMS>,
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    let add_weight_row = add_index * HALF_DIMS;
    let add_psqt_row = add_index * PSQT_BUCKETS;
    let sub_weight_row = sub_index * HALF_DIMS;
    let sub_psqt_row = sub_index * PSQT_BUCKETS;

    accum_add1_sub1_into_both(
        &mut entry.accumulation,
        &feature_transformer.weights[add_weight_row..add_weight_row + HALF_DIMS],
        &feature_transformer.weights[sub_weight_row..sub_weight_row + HALF_DIMS],
        accumulation,
    );
    psqt_add1_sub1_into_both(
        &mut entry.psqt,
        &feature_transformer.psqt_weights[add_psqt_row..add_psqt_row + PSQT_BUCKETS],
        &feature_transformer.psqt_weights[sub_psqt_row..sub_psqt_row + PSQT_BUCKETS],
        psqt,
    );
}

#[inline(always)]
fn apply_feature_add1_sub2_into_both<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    add_index: usize,
    sub0_index: usize,
    sub1_index: usize,
    entry: &mut FinnyEntry<HALF_DIMS>,
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    let add_weight_row = add_index * HALF_DIMS;
    let add_psqt_row = add_index * PSQT_BUCKETS;
    let sub0_weight_row = sub0_index * HALF_DIMS;
    let sub0_psqt_row = sub0_index * PSQT_BUCKETS;
    let sub1_weight_row = sub1_index * HALF_DIMS;
    let sub1_psqt_row = sub1_index * PSQT_BUCKETS;

    accum_add1_sub2_into_both(
        &mut entry.accumulation,
        &feature_transformer.weights[add_weight_row..add_weight_row + HALF_DIMS],
        &feature_transformer.weights[sub0_weight_row..sub0_weight_row + HALF_DIMS],
        &feature_transformer.weights[sub1_weight_row..sub1_weight_row + HALF_DIMS],
        accumulation,
    );
    psqt_add1_sub2_into_both(
        &mut entry.psqt,
        &feature_transformer.psqt_weights[add_psqt_row..add_psqt_row + PSQT_BUCKETS],
        &feature_transformer.psqt_weights[sub0_psqt_row..sub0_psqt_row + PSQT_BUCKETS],
        &feature_transformer.psqt_weights[sub1_psqt_row..sub1_psqt_row + PSQT_BUCKETS],
        psqt,
    );
}

#[inline(always)]
fn apply_feature_add2_sub1_into_both<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    add0_index: usize,
    add1_index: usize,
    sub_index: usize,
    entry: &mut FinnyEntry<HALF_DIMS>,
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    let add0_weight_row = add0_index * HALF_DIMS;
    let add0_psqt_row = add0_index * PSQT_BUCKETS;
    let add1_weight_row = add1_index * HALF_DIMS;
    let add1_psqt_row = add1_index * PSQT_BUCKETS;
    let sub_weight_row = sub_index * HALF_DIMS;
    let sub_psqt_row = sub_index * PSQT_BUCKETS;

    accum_add2_sub1_into_both(
        &mut entry.accumulation,
        &feature_transformer.weights[add0_weight_row..add0_weight_row + HALF_DIMS],
        &feature_transformer.weights[add1_weight_row..add1_weight_row + HALF_DIMS],
        &feature_transformer.weights[sub_weight_row..sub_weight_row + HALF_DIMS],
        accumulation,
    );
    psqt_add2_sub1_into_both(
        &mut entry.psqt,
        &feature_transformer.psqt_weights[add0_psqt_row..add0_psqt_row + PSQT_BUCKETS],
        &feature_transformer.psqt_weights[add1_psqt_row..add1_psqt_row + PSQT_BUCKETS],
        &feature_transformer.psqt_weights[sub_psqt_row..sub_psqt_row + PSQT_BUCKETS],
        psqt,
    );
}

#[inline(always)]
fn apply_feature_sub<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    feature_index: usize,
    entry: &mut FinnyEntry<HALF_DIMS>,
) {
    let weight_row = feature_index * HALF_DIMS;
    let psqt_row = feature_index * PSQT_BUCKETS;

    accum_sub(
        &mut entry.accumulation,
        &feature_transformer.weights[weight_row..weight_row + HALF_DIMS],
    );
    psqt_sub(
        &mut entry.psqt,
        &feature_transformer.psqt_weights[psqt_row..psqt_row + PSQT_BUCKETS],
    );
}

#[inline(always)]
fn apply_feature_sub_into_both<const HALF_DIMS: usize>(
    feature_transformer: &FeatureTransformer,
    feature_index: usize,
    entry: &mut FinnyEntry<HALF_DIMS>,
    accumulation: &mut [i16; HALF_DIMS],
    psqt: &mut [i32; PSQT_BUCKETS],
) {
    let weight_row = feature_index * HALF_DIMS;
    let psqt_row = feature_index * PSQT_BUCKETS;

    accum_sub_into_both(
        &mut entry.accumulation,
        &feature_transformer.weights[weight_row..weight_row + HALF_DIMS],
        accumulation,
    );
    psqt_sub_into_both(
        &mut entry.psqt,
        &feature_transformer.psqt_weights[psqt_row..psqt_row + PSQT_BUCKETS],
        psqt,
    );
}
