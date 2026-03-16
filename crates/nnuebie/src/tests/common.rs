use crate::nnue::NNUEProbe;
use crate::types::{Color, Piece};
use crate::uci::{calculate_material, to_centipawns};
use crate::NnueNetworks;
use std::sync::Arc;
use std::thread;

pub const BIG_NETWORK: &str = "archive/nnue/networks/nn-1c0000000000.nnue";
pub const SMALL_NETWORK: &str = "archive/nnue/networks/nn-37f18f62d772.nnue";
pub const TEST_STACK_SIZE_BYTES: usize = 32 * 1024 * 1024;

pub fn run_with_large_stack<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    thread::Builder::new()
        .stack_size(TEST_STACK_SIZE_BYTES)
        .spawn(f)
        .expect("failed to spawn test thread")
        .join()
        .expect("test thread panicked");
}

pub fn run_with_large_stack_ret<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    thread::Builder::new()
        .stack_size(TEST_STACK_SIZE_BYTES)
        .spawn(f)
        .expect("failed to spawn test thread")
        .join()
        .expect("test thread panicked")
}

pub fn new_probe() -> NNUEProbe {
    NNUEProbe::new(BIG_NETWORK, SMALL_NETWORK).expect("load")
}

pub fn load_networks() -> Arc<NnueNetworks> {
    Arc::new(NnueNetworks::new(BIG_NETWORK, SMALL_NETWORK).expect("Failed to load networks"))
}

pub fn parse_probe_fen(fen: &str) -> (Vec<(Piece, usize)>, Color) {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    let board_str = parts[0];
    let side_str = parts[1];

    let mut pieces = Vec::new();
    let mut rank = 7;
    let mut file = 0;

    for c in board_str.chars() {
        if c == '/' {
            rank -= 1;
            file = 0;
        } else if c.is_ascii_digit() {
            file += c.to_digit(10).unwrap() as usize;
        } else {
            let piece = match c {
                'P' => Piece::WhitePawn,
                'N' => Piece::WhiteKnight,
                'B' => Piece::WhiteBishop,
                'R' => Piece::WhiteRook,
                'Q' => Piece::WhiteQueen,
                'K' => Piece::WhiteKing,
                'p' => Piece::BlackPawn,
                'n' => Piece::BlackKnight,
                'b' => Piece::BlackBishop,
                'r' => Piece::BlackRook,
                'q' => Piece::BlackQueen,
                'k' => Piece::BlackKing,
                _ => panic!("Invalid: {}", c),
            };
            pieces.push((piece, rank * 8 + file));
            file += 1;
        }
    }

    let side = if side_str == "w" {
        Color::White
    } else {
        Color::Black
    };
    (pieces, side)
}

pub fn pieces_to_internal(pieces: &[(Piece, usize)]) -> Vec<(usize, usize, usize)> {
    pieces
        .iter()
        .map(|(p, sq)| {
            let pt = p.piece_type();
            let color = p.color().unwrap_or(Color::White).index();
            (*sq, pt, color)
        })
        .collect()
}

pub fn to_cp(pieces: &[(Piece, usize)], side: Color, internal: i32) -> i32 {
    let internal_vec = pieces_to_internal(pieces);
    let material = calculate_material(&internal_vec);
    let cp = to_centipawns(internal, material);
    if side == Color::Black {
        -cp
    } else {
        cp
    }
}

pub fn get_startpos_pieces() -> Vec<(Piece, usize)> {
    vec![
        (Piece::WhiteRook, 0),
        (Piece::WhiteKnight, 1),
        (Piece::WhiteBishop, 2),
        (Piece::WhiteQueen, 3),
        (Piece::WhiteKing, 4),
        (Piece::WhiteBishop, 5),
        (Piece::WhiteKnight, 6),
        (Piece::WhiteRook, 7),
        (Piece::WhitePawn, 8),
        (Piece::WhitePawn, 9),
        (Piece::WhitePawn, 10),
        (Piece::WhitePawn, 11),
        (Piece::WhitePawn, 12),
        (Piece::WhitePawn, 13),
        (Piece::WhitePawn, 14),
        (Piece::WhitePawn, 15),
        (Piece::BlackPawn, 48),
        (Piece::BlackPawn, 49),
        (Piece::BlackPawn, 50),
        (Piece::BlackPawn, 51),
        (Piece::BlackPawn, 52),
        (Piece::BlackPawn, 53),
        (Piece::BlackPawn, 54),
        (Piece::BlackPawn, 55),
        (Piece::BlackRook, 56),
        (Piece::BlackKnight, 57),
        (Piece::BlackBishop, 58),
        (Piece::BlackQueen, 59),
        (Piece::BlackKing, 60),
        (Piece::BlackBishop, 61),
        (Piece::BlackKnight, 62),
        (Piece::BlackRook, 63),
    ]
}
