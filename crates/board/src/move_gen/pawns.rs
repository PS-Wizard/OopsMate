use std::arch::x86_64::_pext_u64;

use raw::THROUGH;
use raw::{PAWN_ATTACKS, ROOK_ATTACKS, ROOK_MASKS};
use types::moves::MoveCollector;
use types::moves::{Move, MoveType::*};
use types::others::Color::*;
use types::others::Piece::*;

use crate::Position;

impl Position {
    #[inline(always)]
    /// Takes in a pin mask, a check mask and a mutable reference to the move collector, generats
    /// all valid **legal** moves for the pawns
    pub fn generate_pawn_moves(&self, collector: &mut MoveCollector, pinned: u64, check_mask: u64) {
        if self.side_to_move == White {
            self.generate_white_pawn_moves(collector, pinned, check_mask);
        } else {
            self.generate_black_pawn_moves(collector, pinned, check_mask);
        }
    }

    /// Generats all pawn moves for the white side, seperated because white's and black's pawn
    /// moves are different
    fn generate_white_pawn_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        let king_sq = self.our(King).0.trailing_zeros() as usize;
        let pawns = self.our(Pawn).0;
        let empty = !(self.all_pieces[0].0 | self.all_pieces[1].0);
        let enemies = self.them().0;

        // Process each pawn individually to handle pins
        let mut pawn_bb = pawns;
        while pawn_bb != 0 {
            let from = pawn_bb.trailing_zeros() as usize;
            pawn_bb &= pawn_bb - 1;

            let is_pinned = (pinned >> from) & 1 != 0;
            let pin_ray = if is_pinned {
                THROUGH[king_sq][from]
            } else {
                0xFFFFFFFFFFFFFFFFu64
            };

            // Single push
            let push_to = from + 8;
            if push_to < 64 && (empty >> push_to) & 1 != 0 {
                let push_target = 1u64 << push_to;
                if (push_target & pin_ray & check_mask) != 0 {
                    if push_to >= 56 {
                        // Promotion
                        collector.push(Move::new(from, push_to, PromotionQueen));
                        collector.push(Move::new(from, push_to, PromotionRook));
                        collector.push(Move::new(from, push_to, PromotionBishop));
                        collector.push(Move::new(from, push_to, PromotionKnight));
                    } else {
                        collector.push(Move::new(from, push_to, Quiet));
                    }
                }

                // Double push (only if single push was legal and pawn is on rank 2)
                if from >= 8 && from < 16 && push_to < 64 {
                    let double_to = from + 16;
                    let double_target = 1u64 << double_to;
                    if (empty >> double_to) & 1 != 0 && (double_target & pin_ray & check_mask) != 0
                    {
                        collector.push(Move::new(from, double_to, DoublePush));
                    }
                }
            }

            // Captures
            let attacks = PAWN_ATTACKS[0][from] & enemies;
            let mut legal_attacks = attacks & pin_ray & check_mask;
            while legal_attacks != 0 {
                let to = legal_attacks.trailing_zeros() as usize;
                legal_attacks &= legal_attacks - 1;

                if to >= 56 {
                    // Capture promotion
                    collector.push(Move::new(from, to, CapturePromotionQueen));
                    collector.push(Move::new(from, to, CapturePromotionRook));
                    collector.push(Move::new(from, to, CapturePromotionBishop));
                    collector.push(Move::new(from, to, CapturePromotionKnight));
                } else {
                    collector.push(Move::new(from, to, Capture));
                }
            }
        }

        // En passant - THE TRICKY PART
        if let Some(ep_sq) = self.en_passant {
            self.generate_en_passant_moves(collector, pinned, check_mask, ep_sq as usize);
        }
    }

    /// Takes in a pin mask, a check mask and an enpassant square and generates enpassant move if
    /// it is valid and legal. Handles edge cases as leaving the king in check after enpassant.
    fn generate_en_passant_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
        ep_sq: usize,
    ) {
        let king_sq = self.our(King).0.trailing_zeros() as usize;
        let captured_pawn_sq = if self.side_to_move == White {
            ep_sq - 8
        } else {
            ep_sq + 8
        };

        let ep_target = 1u64 << ep_sq;

        let captured_pawn_bit = 1u64 << captured_pawn_sq;
        if (ep_target & check_mask) == 0 && (captured_pawn_bit & check_mask) == 0 {
            return;
        }

        let pawns = self.our(Pawn).0;
        let color_idx = self.side_to_move as usize;

        let mut pawn_bb = pawns;
        while pawn_bb != 0 {
            let from = pawn_bb.trailing_zeros() as usize;
            pawn_bb &= pawn_bb - 1;

            // Can this pawn capture en passant
            if (PAWN_ATTACKS[color_idx][from] & ep_target) == 0 {
                continue;
            }

            // Check if pawn is pinned - en passant must be along pin ray
            let is_pinned = (pinned >> from) & 1 != 0;
            if is_pinned {
                let pin_ray = THROUGH[king_sq][from];
                if (ep_target & pin_ray) == 0 {
                    continue; // En passant not along pin ray
                }
            }

            // Does en passant expose king to horizontal attack?
            // This is the most complex case - removing both pawns might expose king
            let everyone = self.all_pieces[0].0 | self.all_pieces[1].0;
            let after_ep = everyone & !(1u64 << from) & !(1u64 << captured_pawn_sq) | ep_target;

            // Check for horizontal discovered attacks
            let king_rank = king_sq / 8;
            let from_rank = from / 8;

            if king_rank == from_rank && from_rank == captured_pawn_sq / 8 {
                // All on same rank - check for rook/queen attacks
                let rook_idx = unsafe { _pext_u64(after_ep, ROOK_MASKS[king_sq]) as usize };
                let rook_attacks = ROOK_ATTACKS[king_sq][rook_idx];
                let enemy_rooks_queens = self.their(Rook).0 | self.their(Queen).0;

                if (rook_attacks & enemy_rooks_queens) != 0 {
                    continue; // En passant would expose king
                }
            }

            collector.push(Move::new(from, ep_sq, EnPassant));
        }
    }

    /// Mirror of the white pawn moves, generates all valid moves for black pawns
    fn generate_black_pawn_moves(
        &self,
        collector: &mut MoveCollector,
        pinned: u64,
        check_mask: u64,
    ) {
        let king_sq = self.our(King).0.trailing_zeros() as usize;
        let pawns = self.our(Pawn).0;
        let empty = !(self.all_pieces[0].0 | self.all_pieces[1].0);
        let enemies = self.them().0;

        // Process each pawn individually to handle pins
        let mut pawn_bb = pawns;
        while pawn_bb != 0 {
            let from = pawn_bb.trailing_zeros() as usize;
            pawn_bb &= pawn_bb - 1;

            let is_pinned = (pinned >> from) & 1 != 0;
            let pin_ray = if is_pinned {
                THROUGH[king_sq][from]
            } else {
                0xFFFFFFFFFFFFFFFFu64
            };

            // Single push (black pawns move down: from - 8)
            if from >= 8 {
                let push_to = from - 8;
                if (empty >> push_to) & 1 != 0 {
                    let push_target = 1u64 << push_to;
                    if (push_target & pin_ray & check_mask) != 0 {
                        if push_to < 8 {
                            // Promotion (rank 1)
                            collector.push(Move::new(from, push_to, PromotionQueen));
                            collector.push(Move::new(from, push_to, PromotionRook));
                            collector.push(Move::new(from, push_to, PromotionBishop));
                            collector.push(Move::new(from, push_to, PromotionKnight));
                        } else {
                            collector.push(Move::new(from, push_to, Quiet));
                        }
                    }

                    // Double push (only if single push was legal and pawn is on rank 7)
                    if from >= 48 && from < 56 {
                        let double_to = from - 16;
                        let double_target = 1u64 << double_to;
                        if (empty >> double_to) & 1 != 0
                            && (double_target & pin_ray & check_mask) != 0
                        {
                            collector.push(Move::new(from, double_to, DoublePush));
                        }
                    }
                }
            }

            // Captures
            let attacks = PAWN_ATTACKS[1][from] & enemies;
            let legal_attacks = attacks & pin_ray & check_mask;

            let mut attack_bb = legal_attacks;
            while attack_bb != 0 {
                let to = attack_bb.trailing_zeros() as usize;
                attack_bb &= attack_bb - 1;

                if to < 8 {
                    // Capture promotion (rank 1)
                    collector.push(Move::new(from, to, CapturePromotionQueen));
                    collector.push(Move::new(from, to, CapturePromotionRook));
                    collector.push(Move::new(from, to, CapturePromotionBishop));
                    collector.push(Move::new(from, to, CapturePromotionKnight));
                } else {
                    collector.push(Move::new(from, to, Capture));
                }
            }
        }

        // En passant
        if let Some(ep_sq) = self.en_passant {
            self.generate_en_passant_moves(collector, pinned, check_mask, ep_sq as usize);
        }
    }
}

#[cfg(test)]
mod pawns {
    use types::moves::MoveCollector;

    use crate::{Position, legality::attack_constraints::get_attack_constraints};

    #[test]
    fn test() {
        // initial position should be 16 moves
        let g = Position::new();
        let mut mc = MoveCollector::new();
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_pawn_moves(&mut mc, pinned, check_mask);
        assert_eq!(16, mc.len());
        mc.clear();

        // enpassant on b6, expected 11 moves
        let g =
            Position::new_from_fen("rn2k1nr/p1ppp1pp/8/1pP5/8/7P/PP2P1P1/RNBQK2R w KQkq b6 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_pawn_moves(&mut mc, pinned, check_mask);
        assert_eq!(11, mc.len());
        mc.clear();

        let g =
            Position::new_from_fen("1nb1k1nr/pppppppp/4r3/b7/7q/4P3/2PB1PPP/RN1QKB1R w KQk - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_pawn_moves(&mut mc, pinned, check_mask);
        assert_eq!(6, mc.len());
        mc.clear();

        // enpassant to discovered check, expected moves : 1
        let g = Position::new_from_fen("1n2k1nr/ppp1pppp/8/b1KpP2q/8/8/3B4/RN1Q1B1R w k - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_pawn_moves(&mut mc, pinned, check_mask);
        assert_eq!(1, mc.len());
        mc.clear();

        // Capture on a8 to promotion -> Expected: 4 moves
        let g = Position::new_from_fen("rn2k1nr/pPp1pppp/8/b2p3q/8/8/3B4/RNKQ1B1R w KQkq - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_pawn_moves(&mut mc, pinned, check_mask);
        assert_eq!(4, mc.len());
        mc.clear();

        // Expected 8 moves:
        let g = Position::new_from_fen(
            "rn2k1n1/pPp1pppp/4r3/b2p4/6Bq/2P1P3/1P3PP1/RN1QKB1R w KQq - 0 1",
        );
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_pawn_moves(&mut mc, pinned, check_mask);
        assert_eq!(8, mc.len());
        mc.clear();

        // Expected 0 moves: Diagonal Pin
        let g = Position::new_from_fen("8/6b1/8/3pP3/3K3k/8/8/8 w - - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_pawn_moves(&mut mc, pinned, check_mask);
        assert_eq!(0, mc.len());
        mc.clear();

        // Expected 1 moves: Vertical Pin
        let g = Position::new_from_fen("4r3/8/8/3pP3/7k/4K3/8/8 w - - 0 1");
        let (pinned, _, check_mask) = get_attack_constraints(&g);
        g.generate_pawn_moves(&mut mc, pinned, check_mask);
        assert_eq!(1, mc.len());
    }
}
