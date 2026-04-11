//! Full evaluation pipeline.

use crate::accumulator::Accumulator;
use crate::features::{BLACK, PSQT_BUCKETS, WHITE};
use crate::layers::{
    affine_forward, clipped_relu, sparse_affine_forward, sqr_clipped_relu, transform_features,
};
use crate::network::Network;
use crate::types::{piece_value, Color, Piece, PAWN_VALUE};

const OUTPUT_SCALE: i32 = 16;
const WEIGHT_SCALE_BITS: i32 = 6;

#[allow(dead_code)]
pub struct EvalDetails {
    pub psqt_score: i32,
    pub positional_score: i32,
    pub raw_score: i32,
    pub centipawns: i32,
}

pub fn evaluate_position(
    big_network: &Network,
    small_network: &Network,
    pieces: &[(Piece, usize)],
    side_to_move: Color,
    rule50: i32,
    verbose: bool,
) -> (i32, EvalDetails) {
    let stm = side_to_move.index();
    let king_squares = king_squares(pieces);
    let piece_count = pieces.len();
    let bucket = if piece_count > 0 {
        ((piece_count - 1) / 4).min(PSQT_BUCKETS - 1)
    } else {
        0
    };

    let (pawn_count, non_pawn_material) = count_material(pieces);
    let simple_eval = PAWN_VALUE * (pawn_count[stm] - pawn_count[1 - stm])
        + (non_pawn_material[stm] - non_pawn_material[1 - stm]);
    let use_small = simple_eval.abs() > 962;

    if verbose {
        println!();
        println!(
            "    Piece count: {}, Output bucket: {}, simple_eval: {}, use_small: {}",
            piece_count, bucket, simple_eval, use_small
        );
    }

    let (psqt_scaled, positional_scaled, active_label) = if use_small {
        let (psqt, positional) = evaluate_with_network(
            small_network,
            pieces,
            king_squares,
            stm,
            bucket,
            verbose,
            "SMALL",
        );
        let blended = (125 * psqt + 131 * positional) / 128;

        if blended.abs() < 236 {
            if verbose {
                println!();
                println!("    Small-net score is quiet, falling back to big net");
            }
            let (big_psqt, big_positional) = evaluate_with_network(
                big_network,
                pieces,
                king_squares,
                stm,
                bucket,
                verbose,
                "BIG",
            );
            (big_psqt, big_positional, "BIG")
        } else {
            (psqt, positional, "SMALL")
        }
    } else {
        let (psqt, positional) = evaluate_with_network(
            big_network,
            pieces,
            king_squares,
            stm,
            bucket,
            verbose,
            "BIG",
        );
        (psqt, positional, "BIG")
    };

    if verbose {
        println!();
        println!("[6] COMBINING SCORES");
        println!("{}", "-".repeat(70));
        println!("    Active network: {}", active_label);
        println!("    PSQT scaled: {}", psqt_scaled);
        println!("    Positional scaled: {}", positional_scaled);
    }

    let mut nnue_val = (125 * psqt_scaled + 131 * positional_scaled) / 128;

    if verbose {
        println!("    Blended (125*psqt + 131*pos)/128: {}", nnue_val);
    }

    let nnue_complexity = (psqt_scaled - positional_scaled).abs();
    nnue_val -= nnue_val * nnue_complexity / 18000;

    if verbose {
        println!("    After complexity adjustment: {}", nnue_val);
    }

    let total_pawn_count = pawn_count[0] + pawn_count[1];
    let total_npm = non_pawn_material[0] + non_pawn_material[1];
    let material = 535 * total_pawn_count + total_npm;
    let mut score = nnue_val * (77777 + material) / 77777;

    if verbose {
        println!("    Material factor: {}", material);
        println!("    After material scaling: {}", score);
    }

    score -= score * rule50 / 212;
    score = score.clamp(-31753, 31753);

    if verbose {
        println!("    After rule50 adjustment: {}", score);
    }

    let centipawns = to_centipawns(score, pieces);
    let centipawns_white = if stm == BLACK {
        -centipawns
    } else {
        centipawns
    };

    if verbose {
        println!();
        println!("[7] FINAL RESULT");
        println!("{}", "-".repeat(70));
        println!("    Internal score (side to move): {}", score);
        println!("    Centipawns (side to move): {}", centipawns);
        println!("    Centipawns (White's perspective): {}", centipawns_white);
    }

    let details = EvalDetails {
        psqt_score: psqt_scaled,
        positional_score: positional_scaled,
        raw_score: score,
        centipawns,
    };

    (centipawns_white, details)
}

fn evaluate_with_network(
    network: &Network,
    pieces: &[(Piece, usize)],
    king_squares: [usize; 2],
    stm: usize,
    bucket: usize,
    verbose: bool,
    label: &str,
) -> (i32, i32) {
    let mut accumulator = Accumulator::new(network.feature_transformer.half_dims);
    accumulator.refresh(pieces, king_squares, &network.feature_transformer, verbose);

    let mut transformed_features = vec![0u8; network.feature_transformer.half_dims];
    transform_features(
        &accumulator.accumulation,
        stm,
        &mut transformed_features,
        verbose,
    );

    if verbose {
        println!();
        println!("[5] FORWARD PASS ({label}: FC0 -> FC1 -> FC2)");
        println!("{}", "-".repeat(70));
    }

    let mut fc_0_out = vec![0i32; 16];
    if verbose {
        let non_zero_count = transformed_features.iter().filter(|&&x| x != 0).count();
        let sum: u32 = transformed_features.iter().map(|&x| x as u32).sum();
        println!(
            "    FC0 input stats: {} non-zero values, sum={}",
            non_zero_count, sum
        );
        println!("    FC0 biases: {:?}", &network.fc_0[bucket].biases);
        println!(
            "    FC0 weights (first 64): {:?}",
            &network.fc_0[bucket].weights[0..64]
        );
    }
    sparse_affine_forward(&network.fc_0[bucket], &transformed_features, &mut fc_0_out);

    if verbose {
        println!("    FC0 output (16 values): {:?}", &fc_0_out);
    }

    let mut fc_1_in = vec![0u8; 32];
    sqr_clipped_relu(&fc_0_out, &mut fc_1_in[0..16]);
    clipped_relu(&fc_0_out, &mut fc_1_in[15..31]);
    fc_1_in[30] = 0;
    fc_1_in[31] = 0;

    if verbose {
        println!("    After dual activation (fc_1_in, 32 values):");
        println!("      [0..16]  (squared):  {:?}", &fc_1_in[0..16]);
        println!("      [16..32] (linear):   {:?}", &fc_1_in[16..32]);
    }

    let mut fc_1_out = vec![0i32; 32];
    affine_forward(&network.fc_1[bucket], &fc_1_in, &mut fc_1_out);

    if verbose {
        println!("    FC1 output (32 values): {:?}", &fc_1_out);
    }

    let mut ac_1_out = vec![0u8; 32];
    clipped_relu(&fc_1_out, &mut ac_1_out);

    if verbose {
        println!("    After ClippedReLU: {:?}", &ac_1_out);
    }

    let mut fc_2_out = vec![0i32; 1];
    affine_forward(&network.fc_2[bucket], &ac_1_out, &mut fc_2_out);

    if verbose {
        println!("    FC2 output (raw positional): {}", fc_2_out[0]);
    }

    let residual = fc_0_out[15];
    let fwd_out = (residual as i64 * (600 * OUTPUT_SCALE) as i64
        / (127 * (1 << WEIGHT_SCALE_BITS)) as i64) as i32;

    if verbose {
        println!("    Residual (fc_0_out[15]): {}", residual);
        println!("    Residual scaled: {}", fwd_out);
    }

    let positional = fc_2_out[0] + fwd_out;

    let us = stm;
    let them = 1 - stm;
    let psqt = (accumulator.psqt_accumulation[us][bucket]
        - accumulator.psqt_accumulation[them][bucket])
        / 2;

    if verbose {
        println!(
            "    PSQT[us][bucket={}]: {}",
            bucket, accumulator.psqt_accumulation[us][bucket]
        );
        println!(
            "    PSQT[them][bucket={}]: {}",
            bucket, accumulator.psqt_accumulation[them][bucket]
        );
        println!("    PSQT score (raw): {}", psqt);
    }

    (psqt / OUTPUT_SCALE, positional / OUTPUT_SCALE)
}

fn king_squares(pieces: &[(Piece, usize)]) -> [usize; 2] {
    let mut king_squares = [0usize; 2];
    for (piece, square) in pieces {
        match piece {
            Piece::WhiteKing => king_squares[WHITE] = *square,
            Piece::BlackKing => king_squares[BLACK] = *square,
            _ => {}
        }
    }
    king_squares
}

fn count_material(pieces: &[(Piece, usize)]) -> ([i32; 2], [i32; 2]) {
    let mut pawn_count = [0i32; 2];
    let mut non_pawn_material = [0i32; 2];

    for (piece, _) in pieces {
        if let Some(color) = piece.color() {
            let side = color.index();
            if piece.piece_type() == 1 {
                pawn_count[side] += 1;
            } else if !piece.is_king() {
                non_pawn_material[side] += piece_value(*piece);
            }
        }
    }

    (pawn_count, non_pawn_material)
}

fn calculate_material(pieces: &[(Piece, usize)]) -> i32 {
    let mut material = 0;
    for (piece, _) in pieces {
        match piece {
            Piece::WhitePawn | Piece::BlackPawn => material += 1,
            Piece::WhiteKnight | Piece::BlackKnight => material += 3,
            Piece::WhiteBishop | Piece::BlackBishop => material += 3,
            Piece::WhiteRook | Piece::BlackRook => material += 5,
            Piece::WhiteQueen | Piece::BlackQueen => material += 9,
            _ => {}
        }
    }
    material
}

fn win_rate_params(material: i32) -> (f64, f64) {
    let m = (material.clamp(17, 78) as f64) / 58.0;

    let as_coeffs = [-13.50030198, 40.92780883, -36.82753545, 386.83004070];
    let bs_coeffs = [96.53354896, -165.79058388, 90.89679019, 49.29561889];

    let a = (((as_coeffs[0] * m + as_coeffs[1]) * m + as_coeffs[2]) * m) + as_coeffs[3];
    let b = (((bs_coeffs[0] * m + bs_coeffs[1]) * m + bs_coeffs[2]) * m) + bs_coeffs[3];

    (a, b)
}

fn to_centipawns(value: i32, pieces: &[(Piece, usize)]) -> i32 {
    let material = calculate_material(pieces);
    let (a, _b) = win_rate_params(material);
    (100.0 * (value as f64) / a).round() as i32
}
