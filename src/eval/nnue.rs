use super::EvalProvider;
use crate::{Color, Move, MoveType, Piece, Position};
use nnuebie::uci::to_centipawns;
use nnuebie::{Color as NnueColor, MoveDelta, NNUEProbe, NnueNetworks, Piece as NnuePiece};
use std::sync::OnceLock;

const BIG_NETWORK_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/crates/nnuebie/archive/nnue/networks/nn-1c0000000000.nnue"
);
const SMALL_NETWORK_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/crates/nnuebie/archive/nnue/networks/nn-37f18f62d772.nnue"
);

static NNUE_NETWORKS: OnceLock<NnueNetworks> = OnceLock::new();

#[derive(Clone)]
pub struct NnueProvider {
    networks: &'static NnueNetworks,
}

impl NnueProvider {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            networks: networks(),
        }
    }
}

impl Default for NnueProvider {
    fn default() -> Self {
        Self::new()
    }
}
impl EvalProvider for NnueProvider {
    type State = NNUEProbe<'static>;
    type Undo = MoveDelta;

    #[inline(always)]
    fn new_state(&self, pos: &Position) -> Self::State {
        let mut probe = NNUEProbe::from_networks(self.networks);
        self.sync(&mut probe, pos);
        probe
    }

    #[inline(always)]
    fn sync(&self, state: &mut Self::State, pos: &Position) {
        let pieces = collect_pieces(pos);
        state.set_position(&pieces, pos.halfmove as i32);
    }

    #[inline(always)]
    fn eval(&self, pos: &Position, state: &mut Self::State) -> i32 {
        let internal = state.evaluate(map_color(pos.side_to_move));
        to_centipawns(internal, material_count(pos))
    }

    #[inline(always)]
    fn update_on_move(&self, state: &mut Self::State, pos: &Position, mv: Move) -> Self::Undo {
        let delta = build_move_delta(pos, mv);
        state.apply_delta(delta);
        delta
    }

    #[inline(always)]
    fn update_on_undo(&self, state: &mut Self::State, undo: Self::Undo) {
        state.undo_delta(undo);
    }

    #[inline(always)]
    fn update_on_null_move(&self, state: &mut Self::State, pos: &Position) {
        state.make_null_move_with_rule50(pos.halfmove as i32 + 1);
    }

    #[inline(always)]
    fn update_on_undo_null(&self, state: &mut Self::State) {
        state.unmake_null_move();
    }
}

fn networks() -> &'static NnueNetworks {
    NNUE_NETWORKS.get_or_init(|| {
        NnueNetworks::new(BIG_NETWORK_PATH, SMALL_NETWORK_PATH)
            .expect("failed to load embedded nnue networks")
    })
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

fn build_move_delta(pos: &Position, mv: Move) -> MoveDelta {
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
    let mut delta = MoveDelta::new(next_rule50);

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
    use super::{EvalProvider, NnueProvider};
    use crate::{Move, MoveType, Position};
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
    fn incremental_move_matches_full_resync_for_e2e4() {
        run_with_large_stack(|| {
            let provider = NnueProvider::new();
            let mut pos = Position::new();
            let mut inc = provider.new_state(&pos);
            let mv = Move::new(12, 28, MoveType::DoublePush);
            let undo = provider.update_on_move(&mut inc, &pos, mv);
            pos.make_move(mv);
            let inc_score = provider.eval(&pos, &mut inc);

            let mut full = provider.new_state(&pos);
            let full_score = provider.eval(&pos, &mut full);
            assert_eq!(inc_score, full_score);

            pos.unmake_move(mv);
            provider.update_on_undo(&mut inc, undo);
            let restored = provider.eval(&pos, &mut inc);
            let mut fresh = provider.new_state(&pos);
            assert_eq!(restored, provider.eval(&pos, &mut fresh));
        });
    }
}
