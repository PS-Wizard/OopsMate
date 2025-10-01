use crate::Position;
use raw::{
    BISHOP_ATTACKS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS,
    ROOK_MASKS,
};
use std::arch::x86_64::_pext_u64;
use types::{
    moves::{Move, MoveCollector, MoveType::*},
    others::Color::*,
    others::Piece::*,
};
use utilities::algebraic::Algebraic;

impl Position {
    #[inline(always)]
    pub fn generate_king_moves(&self, collector: &mut MoveCollector) {
        let king_sq = self.our(King).0.trailing_zeros() as usize;
        let friendly = self.us().0;
        let enemy = self.them().0;

        // Get all squares the king could potentially move to
        let potential_moves = KING_ATTACKS[king_sq] & !friendly;

        // Remove king from board when checking attacks this is explained: https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/ under the "Gotcha" for king moves
        let all_pieces_without_king =
            (self.all_pieces[0].0 | self.all_pieces[1].0) & !(1u64 << king_sq);

        // Check each potential move
        let mut temp = potential_moves;
        while temp != 0 {
            let to = temp.trailing_zeros() as usize;
            temp &= temp - 1;

            // Only move to safe squares
            if !self.is_square_under_attack(to, all_pieces_without_king) {
                if (1u64 << to) & enemy != 0 {
                    collector.push(Move::new(king_sq, to, Capture));
                } else {
                    collector.push(Move::new(king_sq, to, Quiet));
                }
            }
        }

        // Castling
        self.generate_castling_moves(collector);
    }

    fn is_square_under_attack(&self, target_sq: usize, blockers: u64) -> bool {
        // Pawn attacks
        if (PAWN_ATTACKS[self.side_to_move as usize][target_sq] & self.their(Pawn).0) != 0 {
            return true;
        }

        // Knight attacks
        if (KNIGHT_ATTACKS[target_sq] & self.their(Knight).0) != 0 {
            return true;
        }

        // King attacks (can't move next to enemy king)
        if (KING_ATTACKS[target_sq] & self.their(King).0) != 0 {
            return true;
        }

        // Rook/Queen attacks
        let rook_idx = unsafe { _pext_u64(blockers, ROOK_MASKS[target_sq]) as usize };
        let rook_attacks = ROOK_ATTACKS[target_sq][rook_idx];
        if (rook_attacks & (self.their(Rook).0 | self.their(Queen).0)) != 0 {
            return true;
        }

        // Bishop/Queen attacks
        let bishop_idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[target_sq]) as usize };
        let bishop_attacks = BISHOP_ATTACKS[target_sq][bishop_idx];
        if (bishop_attacks & (self.their(Bishop).0 | self.their(Queen).0)) != 0 {
            return true;
        }

        false
    }

    fn generate_castling_moves(&self, collector: &mut MoveCollector) {
        // Can't castle if in check
        if self.is_in_check() {
            return;
        }

        let all_pieces = self.all_pieces[0].0 | self.all_pieces[1].0;

        if self.side_to_move == White {
            // White kingside: e1-g1
            if self.castling_rights.can_castle_kingside(White) {
                // f1 and g1 must be empty
                if (all_pieces & 0x60) == 0 {
                    // f1 and g1 must not be attacked
                    if !self.is_square_attacked("f1".idx()) && !self.is_square_attacked("g1".idx())
                    {
                        collector.push(Move::new(4, 6, Castle));
                    }
                }
            }

            // White queenside: e1-c1
            if self.castling_rights.can_castle_queenside(White) {
                // b1, c1, d1 must be empty
                if (all_pieces & 0x0E) == 0 {
                    // c1 and d1 must not be attacked (b1 doesn't matter)
                    if !self.is_square_attacked("c1".idx()) && !self.is_square_attacked("d1".idx())
                    {
                        collector.push(Move::new(4, 2, Castle));
                    }
                }
            }
        } else {
            // Black kingside: e8-g8
            if self.castling_rights.can_castle_kingside(Black) {
                if (all_pieces & 0x6000000000000000) == 0 {
                    if !self.is_square_attacked("f8".idx()) && !self.is_square_attacked("g8".idx())
                    {
                        collector.push(Move::new(60, 62, Castle));
                    }
                }
            }

            // Black queenside: e8-c8
            if self.castling_rights.can_castle_queenside(Black) {
                if (all_pieces & 0x0E00000000000000) == 0 {
                    if !self.is_square_attacked("c8".idx()) && !self.is_square_attacked("d8".idx())
                    {
                        collector.push(Move::new(60, 58, Castle));
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn is_in_check(&self) -> bool {
        let king_sq = self.our(King).0.trailing_zeros() as usize;
        self.is_square_attacked(king_sq)
    }
}

#[cfg(test)]
mod kings {
    use types::moves::MoveCollector;

    use crate::Position;

    #[test]
    fn test_king() {
        // Initial position expected 0 king moves
        let g = Position::new();
        let mut mc = MoveCollector::new();
        g.generate_king_moves(&mut mc);
        assert_eq!(mc.len(), 0);
        mc.clear();

        // Expected 2 moves king: e1f1 , e1g1-castle
        let g = Position::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1");
        g.generate_king_moves(&mut mc);
        assert_eq!(mc.len(), 2);
        mc.clear();

        // Expected 1 moves king: e1f1, cant casle cause of rook intercepting the castling  squares
        let g = Position::new_from_fen("rnbqkbn1/pppppprp/8/8/8/8/PPPPPP1P/RNBQK2R w KQq - 0 1");
        g.generate_king_moves(&mut mc);
        assert_eq!(mc.len(), 1);
        mc.clear();

        // Expected 1 moves king: e1f1, a king in check shouldnt be able to castle
        let g = Position::new_from_fen("rnb1kb1r/ppppppPp/8/q7/8/8/PPP1PPPP/RNBQK2R w KQkq - 0 1");
        g.generate_king_moves(&mut mc);
        assert_eq!(mc.len(), 1);
        mc.clear();
    }
}
