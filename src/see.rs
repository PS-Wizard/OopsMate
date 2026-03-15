//! Static exchange evaluation.

use std::arch::x86_64::_pext_u64;

use strikes::{
    BISHOP_ATTACKS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS,
    ROOK_MASKS,
};

use crate::{Color, Move, MoveType, Piece, Position};

const SEE_VALUES: [i32; 6] = [100, 300, 300, 500, 900, 20000];

impl Position {
    /// Computes the static exchange evaluation of `mv` on the current position.
    ///
    /// Positive values indicate a materially favorable exchange sequence for the
    /// side to move, while negative values indicate that the capture sequence is
    /// expected to lose material.
    pub fn see(&self, mv: &Move) -> i32 {
        let to = mv.to();
        let from = mv.from();

        let mut gain = [0i32; 32];
        let mut d = 0;

        let mut victim_val = match mv.move_type() {
            MoveType::EnPassant => SEE_VALUES[Piece::Pawn as usize],
            MoveType::Castle => 0,
            _ => {
                if let Some((p, _)) = self.piece_at(to) {
                    SEE_VALUES[p as usize]
                } else {
                    0
                }
            }
        };

        let (mut attacker_piece, _) = self.piece_at(from).expect("SEE: Moving piece not found");

        if mv.is_promotion() {
            let promo_val = match mv.move_type() {
                MoveType::PromotionQueen | MoveType::CapturePromotionQueen => {
                    SEE_VALUES[Piece::Queen as usize]
                }
                MoveType::PromotionRook | MoveType::CapturePromotionRook => {
                    SEE_VALUES[Piece::Rook as usize]
                }
                MoveType::PromotionBishop | MoveType::CapturePromotionBishop => {
                    SEE_VALUES[Piece::Bishop as usize]
                }
                MoveType::PromotionKnight | MoveType::CapturePromotionKnight => {
                    SEE_VALUES[Piece::Knight as usize]
                }
                _ => 0,
            };
            victim_val += promo_val - SEE_VALUES[Piece::Pawn as usize];
            attacker_piece = match mv.move_type() {
                MoveType::PromotionQueen | MoveType::CapturePromotionQueen => Piece::Queen,
                _ => attacker_piece,
            };
        }

        gain[d] = victim_val;

        let mut occupancy = (self.occupied().0 ^ (1u64 << from)) | (1u64 << to);
        let mut attackers = self.attackers_to_board(to, occupancy);
        let bishop_mask = BISHOP_MASKS[to];
        let rook_mask = ROOK_MASKS[to];
        let bishops_queens =
            self.pieces[Piece::Bishop as usize].0 | self.pieces[Piece::Queen as usize].0;
        let rooks_queens =
            self.pieces[Piece::Rook as usize].0 | self.pieces[Piece::Queen as usize].0;

        let mut side = self.side_to_move.flip();

        while let Some(pair) = self.get_lva(attackers, side) {
            let (lva_piece, lva_sq) = pair;
            d += 1;
            gain[d] = SEE_VALUES[attacker_piece as usize] - gain[d - 1];
            attacker_piece = lva_piece;
            side = side.flip();
            occupancy ^= 1u64 << lva_sq;
            attackers ^= 1u64 << lva_sq;

            if (1u64 << lva_sq) & bishop_mask != 0 {
                let bishop_idx = unsafe { _pext_u64(occupancy, bishop_mask) as usize };
                attackers |= BISHOP_ATTACKS[to][bishop_idx] & bishops_queens;
            }

            if (1u64 << lva_sq) & rook_mask != 0 {
                let rook_idx = unsafe { _pext_u64(occupancy, rook_mask) as usize };
                attackers |= ROOK_ATTACKS[to][rook_idx] & rooks_queens;
            }
            attackers &= occupancy;
        }

        while d > 0 {
            d -= 1;
            gain[d] = -((-gain[d]).max(gain[d + 1]));
        }

        gain[0]
    }

    #[inline(always)]
    fn get_lva(&self, attackers: u64, side: Color) -> Option<(Piece, usize)> {
        let side_attackers = attackers & self.colors[side as usize].0;
        if side_attackers == 0 {
            return None;
        }

        let pawns = side_attackers & self.pieces[Piece::Pawn as usize].0;
        if pawns != 0 {
            return Some((Piece::Pawn, pawns.trailing_zeros() as usize));
        }

        let knights = side_attackers & self.pieces[Piece::Knight as usize].0;
        if knights != 0 {
            return Some((Piece::Knight, knights.trailing_zeros() as usize));
        }

        let bishops = side_attackers & self.pieces[Piece::Bishop as usize].0;
        if bishops != 0 {
            return Some((Piece::Bishop, bishops.trailing_zeros() as usize));
        }

        let rooks = side_attackers & self.pieces[Piece::Rook as usize].0;
        if rooks != 0 {
            return Some((Piece::Rook, rooks.trailing_zeros() as usize));
        }

        let queens = side_attackers & self.pieces[Piece::Queen as usize].0;
        if queens != 0 {
            return Some((Piece::Queen, queens.trailing_zeros() as usize));
        }

        let king = side_attackers & self.pieces[Piece::King as usize].0;
        if king != 0 {
            return Some((Piece::King, king.trailing_zeros() as usize));
        }

        None
    }

    #[inline(always)]
    fn attackers_to_board(&self, sq: usize, occupancy: u64) -> u64 {
        let mut attackers = 0u64;

        attackers |= PAWN_ATTACKS[Color::Black as usize][sq]
            & self.pieces[Piece::Pawn as usize].0
            & self.colors[Color::White as usize].0;
        attackers |= PAWN_ATTACKS[Color::White as usize][sq]
            & self.pieces[Piece::Pawn as usize].0
            & self.colors[Color::Black as usize].0;

        attackers |= KNIGHT_ATTACKS[sq] & self.pieces[Piece::Knight as usize].0;
        attackers |= KING_ATTACKS[sq] & self.pieces[Piece::King as usize].0;

        let bishop_idx = unsafe { _pext_u64(occupancy, BISHOP_MASKS[sq]) as usize };
        attackers |= BISHOP_ATTACKS[sq][bishop_idx]
            & (self.pieces[Piece::Bishop as usize].0 | self.pieces[Piece::Queen as usize].0);

        let rook_idx = unsafe { _pext_u64(occupancy, ROOK_MASKS[sq]) as usize };
        attackers |= ROOK_ATTACKS[sq][rook_idx]
            & (self.pieces[Piece::Rook as usize].0 | self.pieces[Piece::Queen as usize].0);

        attackers
    }
}

#[cfg(test)]
mod test_see {
    use utilities::algebraic::Algebraic;

    use crate::{Move, MoveType, Position};

    #[test]
    fn test_en_passant() {
        let fen = "2k2r2/8/8/4Pp2/8/8/8/7K w - f6 0 1";
        let pos = Position::from_fen(fen).expect("Invalid FEN");

        let m = Move::new("e5".idx(), "f6".idx(), MoveType::EnPassant);

        let score = pos.see(&m);

        assert_eq!(score, 0, "SEE failed: {}", score);
    }

    #[test]
    fn test_promotion_tactics() {
        let fen = "1rk5/P7/8/3p4/8/8/3Q4/7K w - - 0 1";
        let pos = Position::from_fen(fen).expect("Invalid FEN");

        let m = Move::new("a7".idx(), "b8".idx(), MoveType::Capture);

        let score = pos.see(&m);

        assert_eq!(score, 400, "SEE failed: {}", score);
    }

    #[test]
    fn test_queen_takes_protected_pawn() {
        let fen = "3r3k/8/8/3p4/8/8/3Q4/7K w - - 0 1";
        let pos = Position::from_fen(fen).expect("Invalid FEN");

        let m = Move::new("d2".idx(), "d5".idx(), MoveType::Capture);

        let score = pos.see(&m);
        assert_eq!(
            score, -800,
            "SEE failed: Expected -800 (QxP, RxQ), got {}",
            score
        );
    }
    #[test]
    fn test_stand_pat() {
        let fen = "b3r3/7q/3n4/8/4P3/3Q4/5N2/4R2B b - - 0 1";
        let pos = Position::from_fen(fen).expect("Invalid FEN");

        let m = Move::new("d6".idx(), "e4".idx(), MoveType::Capture);

        let score = pos.see(&m);
        assert_eq!(
            score, -200,
            "SEE failed: Expected -800 (QxP, RxQ), got {}",
            score
        );
    }
}
