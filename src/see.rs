use std::arch::x86_64::_pext_u64;

use strikes::{
    BISHOP_ATTACKS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, ROOK_ATTACKS,
    ROOK_MASKS,
};

use crate::{Color, Move, MoveType, Piece, Position};

// Piece values for SEE.
// Note: King is set high to prevent the engine from ever trading it.
const SEE_VALUES: [i32; 6] = [100, 300, 300, 500, 900, 20000];

impl Position {
    /// Static Exchange Evaluation
    /// Returns the approximate material score (in centipawns) resulting from the move.
    /// Positive = Good capture. Negative = Bad capture.
    pub fn see(&self, m: &Move) -> i32 {
        let to = m.to();
        let from = m.from();

        // 1. Initial Exchange: Value of the piece being captured
        let mut gain = [0i32; 32];
        let mut d = 0;

        let mut victim_val = match m.move_type() {
            MoveType::EnPassant => SEE_VALUES[Piece::Pawn as usize],
            MoveType::Castle => 0, // Castling is not a capture
            _ => {
                if let Some((p, _)) = self.piece_at(to) {
                    SEE_VALUES[p as usize]
                } else {
                    0 // Quiet move (initial gain is 0)
                }
            }
        };

        // 2. The Attacker: The piece making the move
        // We look up the piece at 'from'. Since piece_at is now O(1), this is fast.
        let (mut attacker_piece, _) = self.piece_at(from).expect("SEE: Moving piece not found");

        // Handle promotion value immediately for the first move
        if m.is_promotion() {
            let promo_val = match m.move_type() {
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
            // We effectively "trade" our pawn for the promoted piece value immediately
            victim_val += promo_val - SEE_VALUES[Piece::Pawn as usize];
            attacker_piece = match m.move_type() {
                MoveType::PromotionQueen | MoveType::CapturePromotionQueen => Piece::Queen,
                _ => attacker_piece,
            };
        }

        gain[d] = victim_val;

        // We simulate the move has happened: 'from' is empty, 'to' is occupied by attacker.
        // But for the loop, we simply remove 'from' from occupancy initially.
        let mut occupancy = (self.occupied().0 ^ (1u64 << from)) | (1u64 << to);

        // Get all attackers to the target square
        let mut attackers = self.attackers_to_board(to, occupancy);

        // The diagonal/orthogonal masks for the target square
        let bishop_mask = BISHOP_MASKS[to];
        let rook_mask = ROOK_MASKS[to];

        // All sliders (Queens + Bishops, Queens + Rooks)
        let bishops_queens = self.pieces[Piece::Bishop as usize].0 | self.pieces[Piece::Queen as usize].0;
        let rooks_queens = self.pieces[Piece::Rook as usize].0 | self.pieces[Piece::Queen as usize].0;

        let mut side = self.side_to_move.flip();

        // the SEE loop
        loop {
            // Find Least Valuable Aggressor (LVA) for the current side
            let (lva_piece, lva_sq) = match self.get_lva(attackers, side) {
                Some(pair) => pair,
                None => break, // No more attackers
            };

            // Speculative Store
            d += 1;

            // The score is: Value of the piece we just captured - The score if we stop now
            gain[d] = SEE_VALUES[attacker_piece as usize] - gain[d - 1];

            // Optimization: If the accumulated score is already hugely positive for the
            // current side, they will stand pat. (Optional cutoff here).

            // Update Attacker for next loop
            attacker_piece = lva_piece;
            side = side.flip();

            // Remove the LVA from occupancy
            occupancy ^= 1u64 << lva_sq;
            attackers ^= 1u64 << lva_sq; // Clear the used attacker

            // Add hidden sliding attackers (X-Rays)
            // Optimization: Only update diagonals if the removed piece was on a diagonal
            if (1u64 << lva_sq) & bishop_mask != 0 {
                let bishop_idx = unsafe { _pext_u64(occupancy, bishop_mask) as usize };
                attackers |= BISHOP_ATTACKS[to][bishop_idx] & bishops_queens;
            }

            // Only update orthogonals if the removed piece was on a rank/file
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

    /// Optimized LVA finder
    /// Instead of checking every piece type on the whole board, we only check
    /// the intersection of our known attackers mask and the piece bitboards.
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

    /// Helper to get attackers bitboard using PEXT
    #[inline(always)]
    pub fn attackers_to_board(&self, sq: usize, occupancy: u64) -> u64 {
        let mut attackers = 0u64;

        // If we want to know if a White Pawn attacks 'sq', that is equivalent to checking if a Black Pawn on 'sq' attacks the white pawn.
        attackers |= PAWN_ATTACKS[Color::Black as usize][sq]
            & self.pieces[Piece::Pawn as usize].0
            & self.colors[Color::White as usize].0;
        attackers |= PAWN_ATTACKS[Color::White as usize][sq]
            & self.pieces[Piece::Pawn as usize].0
            & self.colors[Color::Black as usize].0;

        attackers |= KNIGHT_ATTACKS[sq] & self.pieces[Piece::Knight as usize].0;

        // Kings
        attackers |= KING_ATTACKS[sq] & self.pieces[Piece::King as usize].0;

        // Bishops / Queens
        let bishop_idx = unsafe { _pext_u64(occupancy, BISHOP_MASKS[sq]) as usize };
        attackers |= BISHOP_ATTACKS[sq][bishop_idx]
            & (self.pieces[Piece::Bishop as usize].0 | self.pieces[Piece::Queen as usize].0);

        // Rooks / Queens
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
        // White Pawn e5, Black plays f5.
        // White Takes
        // Gain: 100 (Pawn).
        // Black rook takes back
        // Net: 0.

        let fen = "2k2r2/8/8/4Pp2/8/8/8/7K w - f6 0 1";
        let pos = Position::from_fen(fen).expect("Invalid FEN");

        let m = Move::new("e5".idx(), "f6".idx(), MoveType::EnPassant);

        let score = pos.see(&m);

        assert_eq!(score, 0, "SEE failed: {}", score);
    }

    #[test]
    fn test_promotion_tactics() {
        // White Pawn captures Rook a8 and Promotes to Queen.
        // Gain: R(500).
        // Bonus: P(100) -> Q(900). Net immediate gain: 500 + 800 = 1300.
        // Black King recaptures (KxQ).
        // Loss: Q(900).
        // Total: 1300 - 900 = 400. (We effectively traded Pawn for Rook).
        let fen = "1rk5/P7/8/3p4/8/8/3Q4/7K w - - 0 1";
        let pos = Position::from_fen(fen).expect("Invalid FEN");

        let m = Move::new("a7".idx(), "b8".idx(), MoveType::Capture);

        let score = pos.see(&m);

        assert_eq!(score, 400, "SEE failed: {}", score);
    }

    #[test]
    fn test_queen_takes_protected_pawn() {
        // White Queen on d2, Black Pawn on d5, Black Rook on d8 protecting the pawn.
        let fen = "3r3k/8/8/3p4/8/8/3Q4/7K w - - 0 1";
        let pos = Position::from_fen(fen).expect("Invalid FEN");

        // Move: Queen (d2) captures Pawn (d5)
        let m = Move::new("d2".idx(), "d5".idx(), MoveType::Capture);

        let score = pos.see(&m);

        // Logic:
        // 1. We get +100 for the Pawn.
        // 2. We lose -900 for the Queen (recaptured by Rook).
        // Result should be -800.
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
