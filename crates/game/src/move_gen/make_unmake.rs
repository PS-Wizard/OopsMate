#![allow(dead_code)]
use crate::{
    game::Game,
    piece::PieceKind,
    pins_checks::move_type::{Move, mv_flags::*},
};

/// Stores all reversible information needed to unmake a move
#[derive(Copy, Clone, Debug)]
pub struct GameState {
    pub castling_rights: u8,
    pub en_passant: u8,
    pub halfmove_clock: u16,
    pub captured_piece: PieceKind,
}

impl Game {
    /// Make a move and return the previous game state for unmaking
    #[inline]
    pub fn make_move(&mut self, mv: Move) -> GameState {
        let from = mv.from_sq() as usize;
        let to = mv.to_sq() as usize;
        let flags = mv.flags();

        // Save current state
        let state = GameState {
            castling_rights: self.castling_rights,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            captured_piece: self.piece_at(to),
        };

        // Get the moving piece
        let moving_piece = self.piece_at(from);
        let is_pawn = (moving_piece as usize) % 6 == 0; // Pawn is index 0 in each color

        // Check if it's a capture (regular capture or promotion capture)
        let is_capture = (flags == CAPT)
            || (flags == PROMO_CAPT_QUEEN)
            || (flags == PROMO_CAPT_ROOK)
            || (flags == PROMO_CAPT_BISHOP)
            || (flags == PROMO_CAPT_KNIGHT);

        if is_capture && state.captured_piece == PieceKind::None {
            panic!("Capture flag set but no piece at destination square {}", to);
        }
        // Update halfmove clock
        if is_pawn || is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // Clear en passant
        self.en_passant = 0;

        // Handle captures (but not en passant, handled separately)
        if is_capture && (flags != ENPASS) {
            self.remove_piece(state.captured_piece, to);
        }

        // Handle en passant
        if flags == ENPASS {
            let capture_sq = if self.turn == 0 {
                to - 8 // White captures down
            } else {
                to + 8 // Black captures up
            };
            let enemy_pawn = if self.turn == 0 {
                PieceKind::BlackPawn
            } else {
                PieceKind::WhitePawn
            };
            self.remove_piece(enemy_pawn, capture_sq);
        }

        // Handle castling
        if flags == CASTLE {
            // Determine rook movement based on king movement
            let (rook_from, rook_to) = match to {
                6 => (7, 5),    // White kingside: h1 -> f1
                2 => (0, 3),    // White queenside: a1 -> d1
                62 => (63, 61), // Black kingside: h8 -> f8
                58 => (56, 59), // Black queenside: a8 -> d8
                _ => unreachable!("Invalid castling target square"),
            };

            let rook_piece = if self.turn == 0 {
                PieceKind::WhiteRook
            } else {
                PieceKind::BlackRook
            };

            self.move_piece(rook_piece, rook_from, rook_to);
        }

        // Check if it's a promotion
        let is_promotion = (flags == PROMO_QUEEN)
            || (flags == PROMO_ROOK)
            || (flags == PROMO_BISHOP)
            || (flags == PROMO_KNIGHT)
            || (flags == PROMO_CAPT_QUEEN)
            || (flags == PROMO_CAPT_ROOK)
            || (flags == PROMO_CAPT_BISHOP)
            || (flags == PROMO_CAPT_KNIGHT);

        // Move the piece (or promote)
        if is_promotion {
            // Remove pawn, add promoted piece
            self.remove_piece(moving_piece, from);

            // Determine which piece to promote to
            let promo_piece = match flags {
                PROMO_QUEEN | PROMO_CAPT_QUEEN => {
                    if self.turn == 0 {
                        PieceKind::WhiteQueen
                    } else {
                        PieceKind::BlackQueen
                    }
                }
                PROMO_ROOK | PROMO_CAPT_ROOK => {
                    if self.turn == 0 {
                        PieceKind::WhiteRook
                    } else {
                        PieceKind::BlackRook
                    }
                }
                PROMO_BISHOP | PROMO_CAPT_BISHOP => {
                    if self.turn == 0 {
                        PieceKind::WhiteBishop
                    } else {
                        PieceKind::BlackBishop
                    }
                }
                PROMO_KNIGHT | PROMO_CAPT_KNIGHT => {
                    if self.turn == 0 {
                        PieceKind::WhiteKnight
                    } else {
                        PieceKind::BlackKnight
                    }
                }
                _ => unreachable!("Invalid promotion flag"),
            };
            self.add_piece(promo_piece, to);
        } else {
            self.move_piece(moving_piece, from, to);
        }

        // Update castling rights
        self.update_castling_rights(from, to, moving_piece);

        // Set en passant square for double pawn push
        if is_pawn {
            let move_distance = (to as i8 - from as i8).abs();
            if move_distance == 16 {
                // Double pawn push
                self.en_passant = if self.turn == 0 {
                    (from + 8) as u8
                } else {
                    (from - 8) as u8
                };
            }
        }

        // Update fullmove counter
        if self.turn == 1 {
            self.fullmove += 1;
        }

        // Switch turn
        self.flip_turn();

        state
    }

    /// Unmake a move using the saved game state
    #[inline]
    pub fn unmake_move(&mut self, mv: Move, state: GameState) {
        // Switch turn back
        self.flip_turn();

        let from = mv.from_sq() as usize;
        let to = mv.to_sq() as usize;
        let flags = mv.flags();

        // Restore state
        self.castling_rights = state.castling_rights;
        self.en_passant = state.en_passant;
        self.halfmove_clock = state.halfmove_clock;

        // Get the piece that's currently at 'to' square
        let moved_piece_kind = self.piece_at(to);

        // Undo castling
        if flags == CASTLE {
            let (rook_from, rook_to) = match to {
                6 => (7, 5),    // White kingside
                2 => (0, 3),    // White queenside
                62 => (63, 61), // Black kingside
                58 => (56, 59), // Black queenside
                _ => unreachable!("Invalid castling target square"),
            };

            let rook_piece = if self.turn == 0 {
                PieceKind::WhiteRook
            } else {
                PieceKind::BlackRook
            };

            self.move_piece(rook_piece, rook_to, rook_from);
        }

        // Check if it's a promotion
        let is_promotion = (flags == PROMO_QUEEN)
            || (flags == PROMO_ROOK)
            || (flags == PROMO_BISHOP)
            || (flags == PROMO_KNIGHT)
            || (flags == PROMO_CAPT_QUEEN)
            || (flags == PROMO_CAPT_ROOK)
            || (flags == PROMO_CAPT_BISHOP)
            || (flags == PROMO_CAPT_KNIGHT);

        // Undo promotion
        if is_promotion {
            self.remove_piece(moved_piece_kind, to);
            let original_pawn = if self.turn == 0 {
                PieceKind::WhitePawn
            } else {
                PieceKind::BlackPawn
            };
            self.add_piece(original_pawn, from);
        } else {
            // Regular move back
            self.move_piece(moved_piece_kind, to, from);
        }

        // Restore captured piece
        if flags == ENPASS {
            // Restore en passant captured pawn
            let capture_sq = if self.turn == 0 { to - 8 } else { to + 8 };
            let enemy_pawn = if self.turn == 0 {
                PieceKind::BlackPawn
            } else {
                PieceKind::WhitePawn
            };
            self.add_piece(enemy_pawn, capture_sq);
        } else if state.captured_piece != PieceKind::None {
            // Restore normal captured piece
            self.add_piece(state.captured_piece, to);
        }

        // Restore fullmove counter
        if self.turn == 1 {
            self.fullmove -= 1;
        }
    }

    /// Update castling rights based on piece movement
    #[inline]
    fn update_castling_rights(&mut self, from: usize, to: usize, piece: PieceKind) {
        // Remove castling rights if king moves
        match piece {
            PieceKind::WhiteKing => {
                self.castling_rights &= 0b1100; // Remove white castling (KQ)
            }
            PieceKind::BlackKing => {
                self.castling_rights &= 0b0011; // Remove black castling (kq)
            }
            _ => {}
        }

        // Remove castling rights if rook moves or is captured
        match from {
            0 => self.castling_rights &= 0b1101,  // a1 rook (Q)
            7 => self.castling_rights &= 0b1110,  // h1 rook (K)
            56 => self.castling_rights &= 0b0111, // a8 rook (q)
            63 => self.castling_rights &= 0b1011, // h8 rook (k)
            _ => {}
        }

        match to {
            0 => self.castling_rights &= 0b1101,
            7 => self.castling_rights &= 0b1110,
            56 => self.castling_rights &= 0b0111,
            63 => self.castling_rights &= 0b1011,
            _ => {}
        }
    }
}

// Perft testing function
impl Game {
    /// Perft test - counts leaf nodes at a given depth
    pub fn perft(&mut self, depth: u8) -> u64 {
        use crate::move_gen::MoveGenerator;

        if depth == 0 {
            return 1;
        }

        let mut mg = MoveGenerator::new();
        self.generate_legal_moves(&mut mg);

        if depth == 1 {
            return mg.count as u64;
        }

        let mut nodes = 0u64;
        for i in 0..mg.count {
            let mv = mg.moves[i];
            let state = self.make_move(mv);
            nodes += self.perft(depth - 1);
            self.unmake_move(mv, state);
        }

        nodes
    }

    /// Divide perft - shows move-by-move breakdown
    pub fn perft_divide(&mut self, depth: u8) {
        use crate::move_gen::MoveGenerator;

        let mut mg = MoveGenerator::new();
        self.generate_legal_moves(&mut mg);

        let mut total = 0u64;
        for i in 0..mg.count {
            let mv = mg.moves[i];
            let state = self.make_move(mv);
            let count = if depth <= 1 { 1 } else { self.perft(depth - 1) };
            self.unmake_move(mv, state);

            // Print move in algebraic notation (simplified)
            let from = mv.from_sq();
            let to = mv.to_sq();
            println!(
                "{}{}: {}",
                square_to_algebraic(from as u8),
                square_to_algebraic(to as u8),
                count
            );
            total += count;
        }
        println!("\nTotal: {}", total);
    }
}

/// Convert square index to algebraic notation
fn square_to_algebraic(sq: u8) -> String {
    let file = (sq % 8) as u8;
    let rank = (sq / 8) as u8;
    format!("{}{}", (b'a' + file) as char, rank + 1)
}

#[cfg(test)]
mod tests {
    use crate::{
        game::Game,
        move_gen::MoveGenerator,
        pins_checks::move_type::{Move, mv_flags},
    };

    #[test]
    fn test_perft_starting_position() {
        let mut game = Game::new();

        // Known perft values for starting position
        assert_eq!(game.perft(1), 20);
        assert_eq!(game.perft(2), 400);
        assert_eq!(game.perft(3), 8902);
        assert_eq!(game.perft(4), 197281);
        assert_eq!(game.perft(5), 4865609);
        assert_eq!(game.perft(6), 119060324);
    }

    #[test]
    fn test_perft_kiwipete() {
        // Famous perft test position
        let mut game =
            Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

        assert_eq!(game.perft(1), 48);
        assert_eq!(game.perft(2), 2039);
        assert_eq!(game.perft(3), 97862);
    }
    #[test]
    fn test_perft_kiwipete_divide() {
        let mut game =
            Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        game.perft_divide(3);
    }

    #[test]
    fn test_specific_line() {
        let mut game =
            Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        let state = game.make_move(Move::new(21, 23, mv_flags::CAPT)); // f3h3 (correct indices)
        game.perft_divide(2);
        game.unmake_move(Move::new(37, 55, mv_flags::CAPT), state);
    }

    #[test]
    fn test_position_after_a2a4() {
        let mut game =
            Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        let state = game.make_move(Move::new(8, 24, mv_flags::NONE)); // a2-a4 double push

        let mut mg = MoveGenerator::new();
        game.generate_legal_moves(&mut mg);

        println!("After a2-a4, Black has {} legal moves", mg.count);
        println!("En passant square: {}", game.en_passant);

        // Should Black be able to capture en passant?
        // b4 pawn can capture on a3

        game.unmake_move(Move::new(8, 24, mv_flags::NONE), state);
    }
}
