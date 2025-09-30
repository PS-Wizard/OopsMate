use crate::Position;
use raw::KING_ATTACKS;
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

        // Regular king moves
        let attacks = KING_ATTACKS[king_sq] & !friendly;

        // Captures
        let mut capture_bb = attacks & enemy;
        while capture_bb != 0 {
            let to = capture_bb.trailing_zeros() as usize;
            capture_bb &= capture_bb - 1;
            collector.push(Move::new(king_sq, to, Capture));
        }

        // Quiet moves
        let mut quiet_bb = attacks & !enemy;
        while quiet_bb != 0 {
            let to = quiet_bb.trailing_zeros() as usize;
            quiet_bb &= quiet_bb - 1;
            collector.push(Move::new(king_sq, to, Quiet));
        }

        // Castling
        self.generate_castling_moves(collector);
    }

    fn generate_castling_moves(&self, collector: &mut MoveCollector) {
        // Can't castle if in check
        if self.is_in_check() {
            return;
        }

        let all_pieces = self.all_pieces[0].0 | self.all_pieces[1].0;

        if self.side_to_move == White {
            // White kingside: e1-g1, rook on h1
            if self.castling_rights.can_castle_kingside(White) {
                // Check f1 and g1 are empty
                if (all_pieces & 0x60) == 0 {
                    // Check f1 and g1 not attacked
                    if !self.is_square_attacked("f1".idx()) && !self.is_square_attacked("g1".idx())
                    {
                        let m = Move::new(4, 6, Castle);
                        collector.push(m); // e1 to g1
                    }
                }
            }

            // White queenside: e1-c1, rook on a1
            if self.castling_rights.can_castle_queenside(White) {
                // Check b1, c1, d1 are empty
                if (all_pieces & 0x0E) == 0 {
                    // bits for b1, c1, d1
                    // Check c1 and d1 not attacked (b1 can be attacked)
                    if !self.is_square_attacked("c1".idx()) && !self.is_square_attacked("d1".idx())
                    {
                        let m = Move::new(4, 2, Castle);
                        collector.push(m); // e1 to c1
                    }
                }
            }
        } else {
            // Black kingside: e8-g8, rook on h8
            if self.castling_rights.can_castle_kingside(Black) {
                // Check f8 and g8 are empty
                if (all_pieces & 0x6000000000000000) == 0 {
                    // Check f8 and g8 not attacked
                    if !self.is_square_attacked("f8".idx()) && !self.is_square_attacked("g8".idx())
                    {
                        let m = Move::new(60, 62, Castle);
                        collector.push(m); // e8 to g8
                    }
                }
            }

            // Black queenside: e8-c8, rook on a8
            if self.castling_rights.can_castle_queenside(Black) {
                // Check b8, c8, d8 are empty
                if (all_pieces & 0x0E00000000000000) == 0 {
                    // Check c8 and d8 not attacked
                    if !self.is_square_attacked("c8".idx()) && !self.is_square_attacked("d8".idx())
                    {
                        let m = Move::new(60, 58, Castle);
                        collector.push(m); // e8 to g8
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

    #[inline(always)]
    pub fn is_other_side_in_check(&self) -> bool {
        let king_sq = self.their(King).0.trailing_zeros() as usize;
        self.is_square_attacked(king_sq)
    }
}

#[cfg(test)]
mod king {
    use types::moves::MoveCollector;

    use crate::Position;

    #[test]
    fn test_kings() {
        // starting pos expected 0 moves
        let g = Position::new();
        let mut c = MoveCollector::new();
        g.generate_king_moves(&mut c);
        assert_eq!(0, c.len());
        c.clear();

        // expected 2 moves, first from e1 to f1 and then e1 to g1 castling
        let g = Position::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1");
        g.generate_king_moves(&mut c);
        assert_eq!(2, c.len());
        c.clear();

        // expected 2 moves, the castling is blocked, but since we are only generating pseudo legal
        // rn the king can move to f2 and f1
        let g = Position::new_from_fen("rnbqkrn1/ppppp1pp/8/8/8/7P/PPPPP1P1/RNBQK2R w KQkq - 0 1");
        g.generate_king_moves(&mut c);
        assert_eq!(2, c.len());
        c.clear();

        // A King in check shouldnt be able to castle, expected 3 moves
        let g = Position::new_from_fen("rn2k1nr/ppppp1pp/8/8/1q6/7P/PPP1P1P1/RNBQK2R w KQkq - 0 1");
        g.generate_king_moves(&mut c);
        assert_eq!(3, c.len());
    }
}
