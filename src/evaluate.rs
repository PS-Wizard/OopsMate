use crate::{Color, Move, MoveType, Piece, Position};
use nnuebie::uci::to_centipawns;
use nnuebie::{Color as NnueColor, MoveDelta, NNUEProbe, NnueNetworks, Piece as NnuePiece};
use std::cell::RefCell;
use std::sync::{Arc, OnceLock};

const BIG_NETWORK_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/crates/nnuebie/archive/nnue/networks/nn-1c0000000000.nnue"
);
const SMALL_NETWORK_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/crates/nnuebie/archive/nnue/networks/nn-37f18f62d772.nnue"
);

static NNUE_NETWORKS: OnceLock<Arc<NnueNetworks>> = OnceLock::new();

thread_local! {
    static THREAD_LOCAL_PROBE: RefCell<Option<NNUEProbe>> = const { RefCell::new(None) };
}

pub type EvalProbe = NNUEProbe;
pub type EvalMoveDelta = MoveDelta;

#[inline(always)]
fn networks() -> Arc<NnueNetworks> {
    NNUE_NETWORKS
        .get_or_init(|| {
            Arc::new(
                NnueNetworks::new(BIG_NETWORK_PATH, SMALL_NETWORK_PATH)
                    .expect("failed to load embedded nnue networks"),
            )
        })
        .clone()
}

#[inline(always)]
fn map_color(color: Color) -> NnueColor {
    match color {
        Color::White => NnueColor::White,
        Color::Black => NnueColor::Black,
    }
}

#[inline(always)]
fn map_piece(piece: Piece, color: Color) -> NnuePiece {
    match (color, piece) {
        (Color::White, Piece::Pawn) => NnuePiece::WhitePawn,
        (Color::White, Piece::Knight) => NnuePiece::WhiteKnight,
        (Color::White, Piece::Bishop) => NnuePiece::WhiteBishop,
        (Color::White, Piece::Rook) => NnuePiece::WhiteRook,
        (Color::White, Piece::Queen) => NnuePiece::WhiteQueen,
        (Color::White, Piece::King) => NnuePiece::WhiteKing,
        (Color::Black, Piece::Pawn) => NnuePiece::BlackPawn,
        (Color::Black, Piece::Knight) => NnuePiece::BlackKnight,
        (Color::Black, Piece::Bishop) => NnuePiece::BlackBishop,
        (Color::Black, Piece::Rook) => NnuePiece::BlackRook,
        (Color::Black, Piece::Queen) => NnuePiece::BlackQueen,
        (Color::Black, Piece::King) => NnuePiece::BlackKing,
    }
}

#[inline(always)]
fn promotion_piece(move_type: MoveType, color: Color) -> NnuePiece {
    let promoted = match move_type {
        MoveType::PromotionKnight | MoveType::CapturePromotionKnight => Piece::Knight,
        MoveType::PromotionBishop | MoveType::CapturePromotionBishop => Piece::Bishop,
        MoveType::PromotionRook | MoveType::CapturePromotionRook => Piece::Rook,
        MoveType::PromotionQueen | MoveType::CapturePromotionQueen => Piece::Queen,
        _ => unreachable!("not a promotion move"),
    };

    map_piece(promoted, color)
}

fn collect_pieces(pos: &Position) -> Vec<(NnuePiece, usize)> {
    let mut pieces = Vec::with_capacity(32);
    for sq in 0..64 {
        if let Some((piece, color)) = pos.board[sq] {
            pieces.push((map_piece(piece, color), sq));
        }
    }
    pieces
}

#[inline(always)]
fn material_count(pos: &Position) -> i32 {
    let pawns = pos.pieces[Piece::Pawn as usize].0.count_ones() as i32;
    let knights = pos.pieces[Piece::Knight as usize].0.count_ones() as i32;
    let bishops = pos.pieces[Piece::Bishop as usize].0.count_ones() as i32;
    let rooks = pos.pieces[Piece::Rook as usize].0.count_ones() as i32;
    let queens = pos.pieces[Piece::Queen as usize].0.count_ones() as i32;

    pawns + 3 * knights + 3 * bishops + 5 * rooks + 9 * queens
}

#[inline(always)]
pub fn new_probe(pos: &Position) -> EvalProbe {
    let mut probe = EvalProbe::from_networks(networks());
    sync_probe(&mut probe, pos);
    probe
}

#[inline(always)]
pub fn sync_probe(probe: &mut EvalProbe, pos: &Position) {
    let pieces = collect_pieces(pos);
    probe.set_position(&pieces, pos.halfmove as i32);
}

#[inline(always)]
pub fn evaluate_with_probe(pos: &Position, probe: &mut EvalProbe) -> i32 {
    let internal = probe.evaluate(map_color(pos.side_to_move));
    to_centipawns(internal, material_count(pos))
}

pub fn evaluate(pos: &Position) -> i32 {
    THREAD_LOCAL_PROBE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let probe = slot.get_or_insert_with(|| EvalProbe::from_networks(networks()));
        sync_probe(probe, pos);
        evaluate_with_probe(pos, probe)
    })
}

#[inline(always)]
pub fn apply_move(probe: &mut EvalProbe, pos: &Position, mv: Move) -> EvalMoveDelta {
    let delta = build_move_delta(pos, mv);
    probe.apply_delta(delta);
    delta
}

#[inline(always)]
pub fn undo_move(probe: &mut EvalProbe, delta: EvalMoveDelta) {
    probe.undo_delta(delta);
}

#[inline(always)]
pub fn apply_null_move(probe: &mut EvalProbe, pos: &Position) {
    probe.make_null_move_with_rule50(pos.halfmove as i32 + 1);
}

#[inline(always)]
pub fn undo_null_move(probe: &mut EvalProbe) {
    probe.unmake_null_move();
}

fn build_move_delta(pos: &Position, mv: Move) -> EvalMoveDelta {
    let from = mv.from();
    let to = mv.to();
    let move_type = mv.move_type();
    let (piece, color) = pos
        .piece_at(from)
        .expect("expected moving piece on source square");
    let moving_piece = map_piece(piece, color);
    let capture_resets = mv.is_capture();
    let next_rule50 = if piece == Piece::Pawn || capture_resets {
        0
    } else {
        pos.halfmove as i32 + 1
    };
    let mut delta = EvalMoveDelta::new(next_rule50);

    match move_type {
        MoveType::Quiet | MoveType::DoublePush => {
            delta
                .push_move(from, to, moving_piece, moving_piece)
                .expect("quiet move should fit inside MoveDelta");
        }
        MoveType::Capture => {
            let (captured, captured_color) = pos
                .piece_at(to)
                .expect("expected captured piece on destination square");
            delta
                .push_move(from, to, moving_piece, moving_piece)
                .expect("capture move should fit inside MoveDelta");
            delta
                .push_removal(to, map_piece(captured, captured_color))
                .expect("capture move should fit inside MoveDelta");
        }
        MoveType::EnPassant => {
            let capture_sq = if color == Color::White {
                to - 8
            } else {
                to + 8
            };
            delta
                .push_move(from, to, moving_piece, moving_piece)
                .expect("en passant move should fit inside MoveDelta");
            delta
                .push_removal(capture_sq, map_piece(Piece::Pawn, color.flip()))
                .expect("en passant move should fit inside MoveDelta");
        }
        MoveType::Castle => {
            let (rook_from, rook_to) = match to {
                6 => (7, 5),
                2 => (0, 3),
                62 => (63, 61),
                58 => (56, 59),
                _ => panic!("invalid castle move"),
            };
            let rook_piece = map_piece(Piece::Rook, color);
            delta
                .push_move(from, to, moving_piece, moving_piece)
                .expect("castle move should fit inside MoveDelta");
            delta
                .push_move(rook_from, rook_to, rook_piece, rook_piece)
                .expect("castle move should fit inside MoveDelta");
        }
        MoveType::PromotionKnight
        | MoveType::PromotionBishop
        | MoveType::PromotionRook
        | MoveType::PromotionQueen => {
            delta
                .push_move(from, to, moving_piece, promotion_piece(move_type, color))
                .expect("promotion move should fit inside MoveDelta");
        }
        MoveType::CapturePromotionKnight
        | MoveType::CapturePromotionBishop
        | MoveType::CapturePromotionRook
        | MoveType::CapturePromotionQueen => {
            let (captured, captured_color) = pos
                .piece_at(to)
                .expect("expected captured piece on destination square");
            delta
                .push_move(from, to, moving_piece, promotion_piece(move_type, color))
                .expect("capture promotion move should fit inside MoveDelta");
            delta
                .push_removal(to, map_piece(captured, captured_color))
                .expect("capture promotion move should fit inside MoveDelta");
        }
    }

    delta
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn run_with_large_stack<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(f)
            .expect("failed to spawn test thread")
            .join()
            .expect("test thread panicked");
    }

    #[test]
    #[ignore = "NNUE evaluation requires release build"]
    fn start_position_eval_is_small_white_edge() {
        run_with_large_stack(|| {
            let pos = Position::new();
            let score = evaluate(&pos);
            assert_eq!(score, 7);
        });
    }
}
