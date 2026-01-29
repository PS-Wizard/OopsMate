use crate::position::Position;
use crate::types::*;
use std::arch::x86_64::_pext_u64;

// Import from raw crate
use raw::{
    line_between, BISHOP_ATTACKS, BISHOP_MASKS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS,
    ROOK_ATTACKS, ROOK_MASKS, THROUGH,
};

// ============================================================================
// ATTACK DETECTION
// ============================================================================

impl Position {
    #[inline(always)]
    pub fn is_square_attacked(&self, sq: usize, by: Color) -> bool {
        let blockers = self.occupied().0;
        self.is_square_attacked_with_blockers(sq, by, blockers)
    }

    #[inline(always)]
    pub fn is_square_attacked_with_blockers(&self, sq: usize, by: Color, blockers: u64) -> bool {
        let attackers = self.colors[by as usize].0;

        // Knights
        if KNIGHT_ATTACKS[sq] & self.pieces[Piece::Knight as usize].0 & attackers != 0 {
            return true;
        }

        // King
        if KING_ATTACKS[sq] & self.pieces[Piece::King as usize].0 & attackers != 0 {
            return true;
        }

        // Pawns
        if PAWN_ATTACKS[by.flip() as usize][sq] & self.pieces[Piece::Pawn as usize].0 & attackers
            != 0
        {
            return true;
        }

        // Bishops/Queens
        let bishop_idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[sq]) as usize };
        if BISHOP_ATTACKS[sq][bishop_idx]
            & (self.pieces[Piece::Bishop as usize].0 | self.pieces[Piece::Queen as usize].0)
            & attackers
            != 0
        {
            return true;
        }

        // Rooks/Queens
        let rook_idx = unsafe { _pext_u64(blockers, ROOK_MASKS[sq]) as usize };
        if ROOK_ATTACKS[sq][rook_idx]
            & (self.pieces[Piece::Rook as usize].0 | self.pieces[Piece::Queen as usize].0)
            & attackers
            != 0
        {
            return true;
        }

        false
    }

    #[inline(always)]
    pub fn is_in_check(&self) -> bool {
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        self.is_square_attacked(king_sq, self.side_to_move.flip())
    }
}

// ============================================================================
// PIN AND CHECK DETECTION
// ============================================================================

pub fn get_constraints(pos: &Position) -> (u64, u64) {
    let king_sq = pos.our(Piece::King).0.trailing_zeros() as usize;
    let us = pos.us().0;
    let them = pos.them().0;
    let occupied = us | them;

    let mut pinned = 0u64;
    let mut checkers = 0u64;

    // Sliding pieces
    let enemy_bishops_queens = pos.their(Piece::Bishop).0 | pos.their(Piece::Queen).0;
    let enemy_rooks_queens = pos.their(Piece::Rook).0 | pos.their(Piece::Queen).0;

    let bishop_rays = BISHOP_ATTACKS[king_sq][0];
    let rook_rays = ROOK_ATTACKS[king_sq][0];

    // Check diagonal pins/checks
    let mut potential = (bishop_rays & enemy_bishops_queens) | (rook_rays & enemy_rooks_queens);
    while potential != 0 {
        let sq = potential.trailing_zeros() as usize;
        potential &= potential - 1;

        let between = line_between(king_sq, sq);
        let pieces_between = between & occupied;

        if pieces_between == 0 {
            checkers |= 1u64 << sq;
        } else if pieces_between.count_ones() == 1 && (pieces_between & us) != 0 {
            pinned |= pieces_between;
        }
    }

    // Knights and pawns
    checkers |= pos.their(Piece::Knight).0 & KNIGHT_ATTACKS[king_sq];
    checkers |= pos.their(Piece::Pawn).0 & PAWN_ATTACKS[pos.side_to_move as usize][king_sq];

    let check_mask = if checkers == 0 {
        !0u64
    } else if checkers.count_ones() == 1 {
        let checker_sq = checkers.trailing_zeros() as usize;
        let mask = line_between(king_sq, checker_sq) | checkers;
        mask
    } else {
        0
    };

    (pinned, check_mask)
}

// ============================================================================
// MOVE GENERATION
// ============================================================================

impl Position {
    pub fn generate_moves(&self, collector: &mut MoveCollector) {
        let (pinned, check_mask) = get_constraints(self);

        self.gen_pawn_moves(collector, pinned, check_mask);
        self.gen_knight_moves(collector, pinned, check_mask);
        self.gen_bishop_moves(collector, pinned, check_mask);
        self.gen_rook_moves(collector, pinned, check_mask);
        self.gen_queen_moves(collector, pinned, check_mask);
        self.gen_king_moves(collector);
    }

    // ========================================================================
    // PAWN MOVES
    // ========================================================================

    fn gen_pawn_moves(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        match self.side_to_move {
            Color::White => self.gen_white_pawns(collector, pinned, check_mask),
            Color::Black => self.gen_black_pawns(collector, pinned, check_mask),
        }
    }

    fn gen_white_pawns(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        let pawns = self.our(Piece::Pawn).0;
        let empty = !self.occupied().0;
        let enemies = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;

        let mut bb = pawns;
        let enemy_king = self.their(Piece::King).0;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let pin_ray = if (pinned >> from) & 1 != 0 {
                THROUGH[king_sq][from]
            } else {
                !0u64
            };

            // Single push
            let to = from + 8;
            if to < 64 && (empty >> to) & 1 != 0 {
                let target = 1u64 << to;
                if (target & pin_ray & check_mask) != 0 {
                    if to >= 56 {
                        collector.push(Move::new(from, to, MoveType::PromotionQueen));
                        collector.push(Move::new(from, to, MoveType::PromotionRook));
                        collector.push(Move::new(from, to, MoveType::PromotionBishop));
                        collector.push(Move::new(from, to, MoveType::PromotionKnight));
                    } else {
                        collector.push(Move::new(from, to, MoveType::Quiet));
                    }
                }
            }

            // Double push (check independently!)
            if from >= 8 && from < 16 {
                let to2 = from + 16;
                let target2 = 1u64 << to2;
                let single_to = from + 8;
                // Both intermediate square AND destination must be empty
                if (empty >> single_to) & 1 != 0
                    && (empty >> to2) & 1 != 0
                    && (target2 & pin_ray & check_mask) != 0
                {
                    collector.push(Move::new(from, to2, MoveType::DoublePush));
                }
            }

            // Captures
            let mut attacks = PAWN_ATTACKS[0][from] & enemies & !enemy_king & pin_ray & check_mask;
            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                if to >= 56 {
                    collector.push(Move::new(from, to, MoveType::CapturePromotionQueen));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionRook));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionBishop));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionKnight));
                } else {
                    collector.push(Move::new(from, to, MoveType::Capture));
                }
            }
        }

        // En passant
        if let Some(ep_sq) = self.en_passant {
            self.gen_en_passant(collector, pinned, check_mask, ep_sq as usize);
        }
    }

    fn gen_black_pawns(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        let pawns = self.our(Piece::Pawn).0;
        let empty = !self.occupied().0;
        let enemies = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;

        let mut bb = pawns;
        let enemy_king = self.their(Piece::King).0;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let pin_ray = if (pinned >> from) & 1 != 0 {
                THROUGH[king_sq][from]
            } else {
                !0u64
            };

            // Single push
            if from >= 8 {
                let to = from - 8;
                if (empty >> to) & 1 != 0 {
                    let target = 1u64 << to;
                    if (target & pin_ray & check_mask) != 0 {
                        if to < 8 {
                            collector.push(Move::new(from, to, MoveType::PromotionQueen));
                            collector.push(Move::new(from, to, MoveType::PromotionRook));
                            collector.push(Move::new(from, to, MoveType::PromotionBishop));
                            collector.push(Move::new(from, to, MoveType::PromotionKnight));
                        } else {
                            collector.push(Move::new(from, to, MoveType::Quiet));
                        }
                    }
                }
            }

            // Double push (check independently!)
            if from >= 48 && from < 56 {
                let to2 = from - 16;
                let target2 = 1u64 << to2;
                let single_to = from - 8;
                // Both intermediate square AND destination must be empty
                if (empty >> single_to) & 1 != 0
                    && (empty >> to2) & 1 != 0
                    && (target2 & pin_ray & check_mask) != 0
                {
                    collector.push(Move::new(from, to2, MoveType::DoublePush));
                }
            }

            // Captures
            let mut attacks = PAWN_ATTACKS[1][from] & enemies & !enemy_king & pin_ray & check_mask;
            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                if to < 8 {
                    collector.push(Move::new(from, to, MoveType::CapturePromotionQueen));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionRook));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionBishop));
                    collector.push(Move::new(from, to, MoveType::CapturePromotionKnight));
                } else {
                    collector.push(Move::new(from, to, MoveType::Capture));
                }
            }
        }

        // En passant
        if let Some(ep_sq) = self.en_passant {
            self.gen_en_passant(collector, pinned, check_mask, ep_sq as usize);
        }
    }

    fn gen_en_passant(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
        ep_sq: usize,
    ) {
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let captured_sq = if self.side_to_move == Color::White {
            ep_sq - 8
        } else {
            ep_sq + 8
        };

        let ep_target = 1u64 << ep_sq;
        let captured_bit = 1u64 << captured_sq;

        if (ep_target & check_mask) == 0 && (captured_bit & check_mask) == 0 {
            return;
        }

        let pawns = self.our(Piece::Pawn).0;
        let color_idx = self.side_to_move as usize;

        let mut bb = pawns;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            if (PAWN_ATTACKS[color_idx][from] & ep_target) == 0 {
                continue;
            }

            if (pinned >> from) & 1 != 0 {
                if (ep_target & THROUGH[king_sq][from]) == 0 {
                    continue;
                }
            }

            // Check horizontal discovered attack
            let king_rank = king_sq / 8;
            let from_rank = from / 8;

            if king_rank == from_rank && from_rank == captured_sq / 8 {
                let occupied = self.occupied().0;
                let after_ep = occupied & !(1u64 << from) & !(1u64 << captured_sq) | ep_target;

                let rook_idx = unsafe { _pext_u64(after_ep, ROOK_MASKS[king_sq]) as usize };
                let rook_attacks = ROOK_ATTACKS[king_sq][rook_idx];
                let enemy_rooks_queens = self.their(Piece::Rook).0 | self.their(Piece::Queen).0;

                if (rook_attacks & enemy_rooks_queens) != 0 {
                    continue;
                }
            }

            collector.push(Move::new(from, ep_sq, MoveType::EnPassant));
        }
    }

    // ========================================================================
    // KNIGHT MOVES
    // ========================================================================

    #[inline(always)]
    fn gen_knight_moves(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        let knights = self.our(Piece::Knight).0 & !pinned;
        let us = self.us().0;
        let them = self.them().0;

        let mut bb = knights;
        let enemy_king = self.their(Piece::King).0;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let mut attacks = KNIGHT_ATTACKS[from] & !us & !enemy_king & check_mask;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                let move_type = if (them >> to) & 1 != 0 {
                    MoveType::Capture
                } else {
                    MoveType::Quiet
                };
                collector.push(Move::new(from, to, move_type));
            }
        }
    }

    // ========================================================================
    // BISHOP MOVES
    // ========================================================================

    #[inline(always)]
    fn gen_bishop_moves(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        let bishops = self.our(Piece::Bishop).0;
        let blockers = self.occupied().0;
        let us = self.us().0;
        let them = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;

        let mut bb = bishops;
        let enemy_king = self.their(Piece::King).0;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[from]) as usize };

            let mut attacks = BISHOP_ATTACKS[from][idx] & !us & !enemy_king;

            if (pinned >> from) & 1 != 0 {
                attacks &= THROUGH[king_sq][from];
            }
            attacks &= check_mask;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                let move_type = if (them >> to) & 1 != 0 {
                    MoveType::Capture
                } else {
                    MoveType::Quiet
                };
                collector.push(Move::new(from, to, move_type));
            }
        }
    }

    // ========================================================================
    // ROOK MOVES
    // ========================================================================

    #[inline(always)]
    fn gen_rook_moves(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        let rooks = self.our(Piece::Rook).0;
        let blockers = self.occupied().0;
        let us = self.us().0;
        let them = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;

        let mut bb = rooks;
        let enemy_king = self.their(Piece::King).0;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let idx = unsafe { _pext_u64(blockers, ROOK_MASKS[from]) as usize };

            let mut attacks = ROOK_ATTACKS[from][idx] & !us & !enemy_king;

            if (pinned >> from) & 1 != 0 {
                attacks &= THROUGH[king_sq][from];
            }
            attacks &= check_mask;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                let move_type = if (them >> to) & 1 != 0 {
                    MoveType::Capture
                } else {
                    MoveType::Quiet
                };
                collector.push(Move::new(from, to, move_type));
            }
        }
    }

    // ========================================================================
    // QUEEN MOVES
    // ========================================================================

    #[inline(always)]
    fn gen_queen_moves(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        let queens = self.our(Piece::Queen).0;
        let blockers = self.occupied().0;
        let us = self.us().0;
        let them = self.them().0;
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;

        let mut bb = queens;
        let enemy_king = self.their(Piece::King).0;
        while bb != 0 {
            let from = bb.trailing_zeros() as usize;
            bb &= bb - 1;

            let bishop_idx = unsafe { _pext_u64(blockers, BISHOP_MASKS[from]) as usize };
            let rook_idx = unsafe { _pext_u64(blockers, ROOK_MASKS[from]) as usize };
            let mut attacks = (BISHOP_ATTACKS[from][bishop_idx] | ROOK_ATTACKS[from][rook_idx])
                & !us
                & !enemy_king;

            if (pinned >> from) & 1 != 0 {
                attacks &= THROUGH[king_sq][from];
            }
            attacks &= check_mask;

            while attacks != 0 {
                let to = attacks.trailing_zeros() as usize;
                attacks &= attacks - 1;

                let move_type = if (them >> to) & 1 != 0 {
                    MoveType::Capture
                } else {
                    MoveType::Quiet
                };
                collector.push(Move::new(from, to, move_type));
            }
        }
    }

    // ========================================================================
    // KING MOVES
    // ========================================================================

    fn gen_king_moves(&self, collector: &mut MoveCollector) {
        let king_sq = self.our(Piece::King).0.trailing_zeros() as usize;
        let us = self.us().0;
        let them = self.them().0;
        let enemy = self.side_to_move.flip();

        let blockers_without_king = self.occupied().0 & !(1u64 << king_sq);

        let enemy_king = self.their(Piece::King).0;
        let mut attacks = KING_ATTACKS[king_sq] & !us & !enemy_king;
        while attacks != 0 {
            let to = attacks.trailing_zeros() as usize;
            attacks &= attacks - 1;

            if !self.is_square_attacked_with_blockers(to, enemy, blockers_without_king) {
                let move_type = if (them >> to) & 1 != 0 {
                    MoveType::Capture
                } else {
                    MoveType::Quiet
                };
                collector.push(Move::new(king_sq, to, move_type));
            }
        }

        // Castling
        if !self.is_in_check() {
            self.gen_castling(collector, king_sq, enemy);
        }
    }

    fn gen_castling(&self, collector: &mut MoveCollector, king_sq: usize, enemy: Color) {
        let occupied = self.occupied().0;

        match self.side_to_move {
            Color::White => {
                // Kingside
                if self.castling_rights.can_castle_kingside(Color::White) {
                    if (occupied & 0x60) == 0
                        && !self.is_square_attacked(5, enemy)
                        && !self.is_square_attacked(6, enemy)
                    {
                        collector.push(Move::new(king_sq, 6, MoveType::Castle));
                    }
                }
                // Queenside
                if self.castling_rights.can_castle_queenside(Color::White) {
                    if (occupied & 0x0E) == 0
                        && !self.is_square_attacked(3, enemy)
                        && !self.is_square_attacked(2, enemy)
                    {
                        collector.push(Move::new(king_sq, 2, MoveType::Castle));
                    }
                }
            }
            Color::Black => {
                // Kingside
                if self.castling_rights.can_castle_kingside(Color::Black) {
                    if (occupied & 0x6000000000000000) == 0
                        && !self.is_square_attacked(61, enemy)
                        && !self.is_square_attacked(62, enemy)
                    {
                        collector.push(Move::new(king_sq, 62, MoveType::Castle));
                    }
                }
                // Queenside
                if self.castling_rights.can_castle_queenside(Color::Black) {
                    if (occupied & 0x0E00000000000000) == 0
                        && !self.is_square_attacked(59, enemy)
                        && !self.is_square_attacked(58, enemy)
                    {
                        collector.push(Move::new(king_sq, 58, MoveType::Castle));
                    }
                }
            }
        }
    }
}
