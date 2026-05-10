use super::{GameState, Position};
use crate::CastleRights;

fn dummy_state(hash: u64) -> GameState {
    GameState {
        castling_rights: CastleRights::NONE,
        en_passant: None,
        halfmove: 1,
        plies_from_null: 1,
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
fn fifty_move_draw_does_not_mask_checkmate() {
    let pos = Position::from_fen("4Q2k/6pp/7K/8/8/8/8/8 b - - 100 1").unwrap();

    assert!(pos.is_fifty_move_draw());
    assert!(pos.is_in_check());
}

#[test]
fn detects_repetition_by_hash_in_recent_history() {
    let mut pos = Position::new();
    let h = pos.hash();
    pos.halfmove = 8;
    pos.plies_from_null = 8;
    pos.history = vec![
        dummy_state(11),
        dummy_state(22),
        dummy_state(h),
        dummy_state(44),
    ];

    assert!(pos.is_repetition());
}

#[test]
fn make_and_unmake_restore_plies_from_null() {
    let mut pos = Position::new();
    pos.plies_from_null = 7;

    pos.make_null_move();
    assert_eq!(pos.plies_from_null, 0);

    pos.unmake_null_move();
    assert_eq!(pos.plies_from_null, 7);
}

#[test]
fn ignores_positions_before_halfmove_window() {
    let mut pos = Position::new();
    let h = pos.hash();
    pos.halfmove = 4;
    pos.plies_from_null = 4;
    pos.history = vec![
        dummy_state(h),
        dummy_state(55),
        dummy_state(66),
        dummy_state(77),
        dummy_state(88),
    ];

    assert!(!pos.is_repetition());
}

#[test]
fn ignores_repetition_before_null_move_boundary() {
    let mut pos = Position::new();
    let h = pos.hash();
    pos.halfmove = 8;
    pos.plies_from_null = 2;
    pos.history = vec![
        dummy_state(h),
        dummy_state(11),
        dummy_state(22),
        dummy_state(33),
    ];

    assert!(!pos.is_repetition());
}
