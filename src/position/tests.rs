use super::{GameState, Position};
use crate::CastleRights;

fn dummy_state(hash: u64) -> GameState {
    GameState {
        castling_rights: CastleRights::NONE,
        en_passant: None,
        halfmove: 1,
        hash,
        captured_piece: None,
    }
}

#[test]
fn detects_fifty_move_draw() {
    let mut pos = Position::new();
    pos.halfmove = 100;
    assert!(pos.is_fifty_move_draw());
}

#[test]
fn detects_repetition_by_hash_in_recent_history() {
    let mut pos = Position::new();
    let h = pos.hash();
    pos.halfmove = 8;
    pos.history = vec![
        dummy_state(11),
        dummy_state(22),
        dummy_state(h),
        dummy_state(44),
    ];

    assert!(pos.is_repetition());
}

#[test]
fn ignores_positions_before_halfmove_window() {
    let mut pos = Position::new();
    let h = pos.hash();
    pos.halfmove = 4;
    pos.history = vec![
        dummy_state(h),
        dummy_state(55),
        dummy_state(66),
        dummy_state(77),
        dummy_state(88),
    ];

    assert!(!pos.is_repetition());
}
