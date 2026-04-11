//! Accumulator: the heart of NNUE incremental computation
//!
//! The accumulator stores the transformed feature sums for both perspectives.
//! For each perspective, it contains:
//! - accumulation: [half_dims] i16 values (sum of weights for active features + bias)
//! - psqt_accumulation: [8] i32 values (PSQT scores per bucket)
//!
//! In this simplified version, we always refresh from scratch (no incremental updates).

use crate::features::{explain_feature, make_index, BLACK, PSQT_BUCKETS, WHITE};
use crate::network::FeatureTransformer;
use crate::types::{square_name, Piece};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
fn avx2_available() -> bool {
    std::arch::is_x86_feature_detected!("avx2")
}

#[cfg(not(target_arch = "x86_64"))]
fn avx2_available() -> bool {
    false
}

fn add_feature_scalar(acc: &mut [i16], weights: &[i16]) {
    for (slot, &weight) in acc.iter_mut().zip(weights.iter()) {
        *slot = slot.wrapping_add(weight);
    }
}

fn add_psqt_scalar(acc: &mut [i32; PSQT_BUCKETS], weights: &[i32]) {
    for (slot, &weight) in acc.iter_mut().zip(weights.iter()) {
        *slot += weight;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn add_feature_avx2(acc: &mut [i16], weights: &[i16]) {
    let mut i = 0;
    while i + 16 <= acc.len() {
        let a = _mm256_loadu_si256(acc.as_ptr().add(i) as *const __m256i);
        let w = _mm256_loadu_si256(weights.as_ptr().add(i) as *const __m256i);
        _mm256_storeu_si256(
            acc.as_mut_ptr().add(i) as *mut __m256i,
            _mm256_add_epi16(a, w),
        );
        i += 16;
    }

    for (slot, &weight) in acc[i..].iter_mut().zip(weights[i..].iter()) {
        *slot = slot.wrapping_add(weight);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn add_psqt_avx2(acc: &mut [i32; PSQT_BUCKETS], weights: &[i32]) {
    let current = _mm256_loadu_si256(acc.as_ptr() as *const __m256i);
    let delta = _mm256_loadu_si256(weights.as_ptr() as *const __m256i);
    _mm256_storeu_si256(
        acc.as_mut_ptr() as *mut __m256i,
        _mm256_add_epi32(current, delta),
    );
}

fn add_feature(acc: &mut [i16], weights: &[i16]) {
    if avx2_available() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            add_feature_avx2(acc, weights);
            return;
        }
    }

    add_feature_scalar(acc, weights);
}

fn add_psqt(acc: &mut [i32; PSQT_BUCKETS], weights: &[i32]) {
    if avx2_available() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            add_psqt_avx2(acc, weights);
            return;
        }
    }

    add_psqt_scalar(acc, weights);
}

/// Per-perspective accumulator data
pub struct Accumulator {
    /// Feature transformer output: [2][half_dims] (one per perspective)
    pub accumulation: [Vec<i16>; 2],
    /// PSQT scores per bucket: [2][8] (one per perspective)
    pub psqt_accumulation: [[i32; PSQT_BUCKETS]; 2],
}

impl Accumulator {
    pub fn new(half_dims: usize) -> Self {
        Self {
            accumulation: [vec![0i16; half_dims], vec![0i16; half_dims]],
            psqt_accumulation: [[0i32; PSQT_BUCKETS]; 2],
        }
    }

    /// Refresh the accumulator from scratch given the current piece list
    /// This is the non-incremental version - just sum up all active features
    pub fn refresh(
        &mut self,
        pieces: &[(Piece, usize)], // (piece, square)
        king_squares: [usize; 2],  // [white_king_sq, black_king_sq]
        ft: &FeatureTransformer,
        verbose: bool,
    ) {
        let half_dims = ft.half_dims;

        if verbose {
            println!();
            println!("[3] BUILDING ACCUMULATOR");
            println!("{}", "-".repeat(70));
            println!(
                "    King squares: White={}, Black={}",
                square_name(king_squares[WHITE]),
                square_name(king_squares[BLACK])
            );
        }

        // Initialize with biases
        for perspective in 0..2 {
            for i in 0..half_dims {
                self.accumulation[perspective][i] = ft.biases[i];
            }
            for i in 0..PSQT_BUCKETS {
                self.psqt_accumulation[perspective][i] = 0;
            }
        }

        if verbose {
            println!("    Initialized with biases (first 8 values):");
            println!(
                "      White perspective: {:?}",
                &self.accumulation[WHITE][0..8]
            );
            println!(
                "      Black perspective: {:?}",
                &self.accumulation[BLACK][0..8]
            );
            println!();
            println!("    Computing features for each piece:");
        }

        // Add weights for each active feature.
        // Kings are included: HalfKA uses the king square as context and also encodes kings
        // as regular piece features via the king piece bucket.
        for (piece, square) in pieces {
            let pc = piece.stockfish_code();

            // Compute feature index for both perspectives
            let white_idx = make_index(WHITE, *square, pc, king_squares[WHITE]);
            let black_idx = make_index(BLACK, *square, pc, king_squares[BLACK]);

            if verbose {
                println!("      {} on {}:", piece.symbol(), square_name(*square));
                println!(
                    "{}",
                    explain_feature(WHITE, *square, *piece, king_squares[WHITE])
                );
                println!(
                    "{}",
                    explain_feature(BLACK, *square, *piece, king_squares[BLACK])
                );
            }

            // Add weights to accumulator (White perspective)
            let weight_offset_w = white_idx * half_dims;
            add_feature(
                &mut self.accumulation[WHITE],
                &ft.weights[weight_offset_w..weight_offset_w + half_dims],
            );

            // Add weights to accumulator (Black perspective)
            let weight_offset_b = black_idx * half_dims;
            add_feature(
                &mut self.accumulation[BLACK],
                &ft.weights[weight_offset_b..weight_offset_b + half_dims],
            );

            // Add PSQT weights
            let psqt_offset_w = white_idx * PSQT_BUCKETS;
            let psqt_offset_b = black_idx * PSQT_BUCKETS;
            add_psqt(
                &mut self.psqt_accumulation[WHITE],
                &ft.psqt_weights[psqt_offset_w..psqt_offset_w + PSQT_BUCKETS],
            );
            add_psqt(
                &mut self.psqt_accumulation[BLACK],
                &ft.psqt_weights[psqt_offset_b..psqt_offset_b + PSQT_BUCKETS],
            );
        }

        if verbose {
            println!();
            println!("    Final accumulator (first 8 values):");
            println!(
                "      White perspective: {:?}",
                &self.accumulation[WHITE][0..8]
            );
            println!(
                "      Black perspective: {:?}",
                &self.accumulation[BLACK][0..8]
            );
            println!();
            println!("    PSQT accumulation:");
            println!(
                "      White perspective: {:?}",
                &self.psqt_accumulation[WHITE]
            );
            println!(
                "      Black perspective: {:?}",
                &self.psqt_accumulation[BLACK]
            );
        }
    }
}
