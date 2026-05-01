use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

mod emit;
mod geometry;
mod leapers;
mod sliders;

pub fn generate() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = PathBuf::from(env::var("OUT_DIR")?).join("tables.rs");
    let out = BufWriter::new(File::create(out_path)?);

    let pawn_attacks = leapers::pawns::generate();
    let knight_attacks = leapers::knights::generate();
    let king_attacks = leapers::kings::generate();
    let rook_masks = sliders::rook::generate_masks();
    let bishop_masks = sliders::bishop::generate_masks();
    let (rook_offsets, rook_attacks) =
        sliders::table::generate(&rook_masks, sliders::rook::attacks);
    let (bishop_offsets, bishop_attacks) =
        sliders::table::generate(&bishop_masks, sliders::bishop::attacks);
    let between = geometry::between::generate();
    let through = geometry::through::generate();

    emit::write_tables(
        out,
        &pawn_attacks,
        &knight_attacks,
        &king_attacks,
        &rook_masks,
        &bishop_masks,
        &rook_offsets,
        &bishop_offsets,
        &rook_attacks,
        &bishop_attacks,
        &between,
        &through,
    )?;

    Ok(())
}

#[inline(always)]
pub(crate) fn on_board(rank: i32, file: i32) -> bool {
    (0..8).contains(&rank) && (0..8).contains(&file)
}

#[inline(always)]
pub(crate) fn alignment_step(from: usize, to: usize) -> Option<(i32, i32)> {
    if from == to {
        return None;
    }

    let from_rank = (from / 8) as i32;
    let from_file = (from % 8) as i32;
    let to_rank = (to / 8) as i32;
    let to_file = (to % 8) as i32;
    let rank_diff = to_rank - from_rank;
    let file_diff = to_file - from_file;

    if rank_diff == 0 {
        return Some((0, file_diff.signum()));
    }

    if file_diff == 0 {
        return Some((rank_diff.signum(), 0));
    }

    if rank_diff.abs() == file_diff.abs() {
        return Some((rank_diff.signum(), file_diff.signum()));
    }

    None
}
