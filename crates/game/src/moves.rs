#![allow(dead_code)]

use std::arch::x86_64::_pext_u64;

use crate::game::Game;
use pext::{
    BISHOP_ATTACKS, BISHOP_MASKS, FILE_A, FILE_H, KING_ATTACKS, KNIGHT_ATTACKS, RANK_1, RANK_2,
    RANK_4, RANK_5, RANK_7, RANK_8, ROOK_ATTACKS, ROOK_MASKS,
};
use types::{
    flags::{
        FLAG_CAPTURE, FLAG_CASTLE_KINGSIDE, FLAG_CASTLE_QUEENSIDE, FLAG_DOUBLE_PUSH,
        FLAG_EN_PASSANT, FLAG_QUIET, PROMO_BISHOP, PROMO_KNIGHT, PROMO_QUEEN, PROMO_ROOK,
    },
    move_type::MoveList,
    piece_kind::Piece::{self, *},
};

// Castling rights constants
const WHITE_KINGSIDE: u8 = 0b0001;
const WHITE_QUEENSIDE: u8 = 0b0010;
const BLACK_KINGSIDE: u8 = 0b0100;
const BLACK_QUEENSIDE: u8 = 0b1000;

// Castling square masks
const WHITE_KINGSIDE_SQUARES: u64 = 0x0000_0000_0000_0060; // f1, g1
const WHITE_QUEENSIDE_SQUARES: u64 = 0x0000_0000_0000_000E; // b1, c1, d1
const BLACK_KINGSIDE_SQUARES: u64 = 0x6000_0000_0000_0000; // f8, g8
const BLACK_QUEENSIDE_SQUARES: u64 = 0x0E00_0000_0000_0000; // b8, c8, d8

// King and rook starting positions
const WHITE_KING_START: u8 = 4; // e1
const WHITE_ROOK_KINGSIDE: u8 = 7; // h1
const WHITE_ROOK_QUEENSIDE: u8 = 0; // a1
const BLACK_KING_START: u8 = 60; // e8
const BLACK_ROOK_KINGSIDE: u8 = 63; // h8
const BLACK_ROOK_QUEENSIDE: u8 = 56; // a8

impl Game {
    #[inline(always)]
    pub fn get_all_moves(&self, moves: &mut MoveList) {
        let enemies = self.enemy_occupied();
        let friendlies = self.friendly_occupied();
        let occupied = enemies | friendlies;

        self.get_pawn_moves(moves, enemies, friendlies);
        self.get_sliding_moves(moves, enemies, occupied);
        self.get_knight_moves(moves, friendlies, enemies);
        self.get_king_moves(moves, friendlies, enemies, occupied);
    }

    #[inline(always)]
    fn get_pawn_moves(&self, moves: &mut MoveList, enemies: u64, friendlies: u64) {
        let pawns = self.friendly_board(Pawns);
        if pawns == 0 {
            return;
        }

        let empty = !(enemies | friendlies);

        // White pawns
        if self.turn == 0 {
            let single = (pawns << 8) & empty;
            let double = ((pawns & RANK_2) << 16) & empty;
            let left_attacks = ((pawns & !FILE_A) << 7) & enemies;
            let right_attacks = ((pawns & !FILE_H) << 9) & enemies;

            // Regular moves (non-promotion)
            let mut regular_single = single & !RANK_8;
            let mut regular_double = double & !RANK_8;
            let mut regular_left = left_attacks & !RANK_8;
            let mut regular_right = right_attacks & !RANK_8;

            // Promotion moves
            let mut promo_single = single & RANK_8;
            let mut promo_left = left_attacks & RANK_8;
            let mut promo_right = right_attacks & RANK_8;

            // Process regular moves
            while regular_single != 0 {
                let to = regular_single.trailing_zeros() as u8;
                regular_single &= regular_single - 1;
                moves.push_quiet(to - 8, to, FLAG_QUIET);
            }
            while regular_double != 0 {
                let to = regular_double.trailing_zeros() as u8;
                regular_double &= regular_double - 1;
                moves.push_quiet(to - 16, to, FLAG_DOUBLE_PUSH);
            }
            while regular_left != 0 {
                let to = regular_left.trailing_zeros() as u8;
                regular_left &= regular_left - 1;
                moves.push_capture(to - 7, to, FLAG_CAPTURE);
            }
            while regular_right != 0 {
                let to = regular_right.trailing_zeros() as u8;
                regular_right &= regular_right - 1;
                moves.push_capture(to - 9, to, FLAG_CAPTURE);
            }

            // Process promotions
            while promo_single != 0 {
                let to = promo_single.trailing_zeros() as u8;
                promo_single &= promo_single - 1;
                let from = to - 8;
                moves.push_promotion(from, to, PROMO_QUEEN);
                moves.push_promotion(from, to, PROMO_ROOK);
                moves.push_promotion(from, to, PROMO_BISHOP);
                moves.push_promotion(from, to, PROMO_KNIGHT);
            }
            while promo_left != 0 {
                let to = promo_left.trailing_zeros() as u8;
                promo_left &= promo_left - 1;
                let from = to - 7;
                moves.push_promotion(from, to, PROMO_QUEEN);
                moves.push_promotion(from, to, PROMO_ROOK);
                moves.push_promotion(from, to, PROMO_BISHOP);
                moves.push_promotion(from, to, PROMO_KNIGHT);
            }
            while promo_right != 0 {
                let to = promo_right.trailing_zeros() as u8;
                promo_right &= promo_right - 1;
                let from = to - 9;
                moves.push_promotion(from, to, PROMO_QUEEN);
                moves.push_promotion(from, to, PROMO_ROOK);
                moves.push_promotion(from, to, PROMO_BISHOP);
                moves.push_promotion(from, to, PROMO_KNIGHT);
            }

            // En passant
            if self.enpassant_sq != 0 {
                let ep_square = 1u64 << self.enpassant_sq;
                let mut ep_attacks = (((pawns & RANK_5 & !FILE_A) << 7)
                    | ((pawns & RANK_5 & !FILE_H) << 9))
                    & ep_square;
                while ep_attacks != 0 {
                    let to = ep_attacks.trailing_zeros() as u8;
                    ep_attacks &= ep_attacks - 1;
                    let from = if (1u64 << to) & ((pawns & RANK_5) << 7) != 0 {
                        to - 7
                    } else {
                        to - 9
                    };
                    moves.push_capture(from, to, FLAG_EN_PASSANT);
                }
            }
        } else {
            // Black pawns - mirror of white logic
            let single = (pawns >> 8) & empty;
            let double = ((pawns & RANK_7) >> 16) & empty;
            let left_attacks = ((pawns & !FILE_H) >> 7) & enemies;
            let right_attacks = ((pawns & !FILE_A) >> 9) & enemies;

            let mut regular_single = single & !RANK_1;
            let mut regular_double = double & !RANK_1;
            let mut regular_left = left_attacks & !RANK_1;
            let mut regular_right = right_attacks & !RANK_1;

            let mut promo_single = single & RANK_1;
            let mut promo_left = left_attacks & RANK_1;
            let mut promo_right = right_attacks & RANK_1;

            while regular_single != 0 {
                let to = regular_single.trailing_zeros() as u8;
                regular_single &= regular_single - 1;
                moves.push_quiet(to + 8, to, FLAG_QUIET);
            }
            while regular_double != 0 {
                let to = regular_double.trailing_zeros() as u8;
                regular_double &= regular_double - 1;
                moves.push_quiet(to + 16, to, FLAG_DOUBLE_PUSH);
            }
            while regular_left != 0 {
                let to = regular_left.trailing_zeros() as u8;
                regular_left &= regular_left - 1;
                moves.push_capture(to + 7, to, FLAG_CAPTURE);
            }
            while regular_right != 0 {
                let to = regular_right.trailing_zeros() as u8;
                regular_right &= regular_right - 1;
                moves.push_capture(to + 9, to, FLAG_CAPTURE);
            }

            while promo_single != 0 {
                let to = promo_single.trailing_zeros() as u8;
                promo_single &= promo_single - 1;
                let from = to + 8;
                moves.push_promotion(from, to, PROMO_QUEEN);
                moves.push_promotion(from, to, PROMO_ROOK);
                moves.push_promotion(from, to, PROMO_BISHOP);
                moves.push_promotion(from, to, PROMO_KNIGHT);
            }
            while promo_left != 0 {
                let to = promo_left.trailing_zeros() as u8;
                promo_left &= promo_left - 1;
                let from = to + 7;
                moves.push_promotion(from, to, PROMO_QUEEN);
                moves.push_promotion(from, to, PROMO_ROOK);
                moves.push_promotion(from, to, PROMO_BISHOP);
                moves.push_promotion(from, to, PROMO_KNIGHT);
            }
            while promo_right != 0 {
                let to = promo_right.trailing_zeros() as u8;
                promo_right &= promo_right - 1;
                let from = to + 9;
                moves.push_promotion(from, to, PROMO_QUEEN);
                moves.push_promotion(from, to, PROMO_ROOK);
                moves.push_promotion(from, to, PROMO_BISHOP);
                moves.push_promotion(from, to, PROMO_KNIGHT);
            }

            if self.enpassant_sq != 0 {
                let ep_square = 1u64 << self.enpassant_sq;
                let mut ep_attacks = (((pawns & RANK_4 & !FILE_H) >> 7)
                    | ((pawns & RANK_4 & !FILE_A) >> 9))
                    & ep_square;
                while ep_attacks != 0 {
                    let to = ep_attacks.trailing_zeros() as u8;
                    ep_attacks &= ep_attacks - 1;
                    let from = if (1u64 << to) & ((pawns & RANK_4) >> 9) != 0 {
                        to + 9
                    } else {
                        to + 7
                    };
                    moves.push_capture(from, to, FLAG_EN_PASSANT);
                }
            }
        }
    }

    #[inline(always)]
    fn get_sliding_moves(&self, moves: &mut MoveList, enemies: u64, occupied: u64) {
        // Rooks
        let mut rooks = self.friendly_board(Rooks);
        while rooks != 0 {
            let from = rooks.trailing_zeros() as u8;
            rooks ^= 1u64 << from; // Faster than &= rooks - 1

            let attacks = unsafe {
                ROOK_ATTACKS[from as usize][_pext_u64(occupied, ROOK_MASKS[from as usize]) as usize]
            };

            let mut quiet_moves = attacks & !occupied;
            let mut captures = attacks & enemies;

            while quiet_moves != 0 {
                let to = quiet_moves.trailing_zeros() as u8;
                quiet_moves ^= 1u64 << to;
                moves.push_quiet(from, to, FLAG_QUIET);
            }
            while captures != 0 {
                let to = captures.trailing_zeros() as u8;
                captures ^= 1u64 << to;
                moves.push_capture(from, to, FLAG_CAPTURE);
            }
        }

        // Bishops
        let mut bishops = self.friendly_board(Bishops);
        while bishops != 0 {
            let from = bishops.trailing_zeros() as u8;
            bishops ^= 1u64 << from;

            let attacks = unsafe {
                BISHOP_ATTACKS[from as usize]
                    [_pext_u64(occupied, BISHOP_MASKS[from as usize]) as usize]
            };

            let mut quiet_moves = attacks & !occupied;
            let mut captures = attacks & enemies;

            while quiet_moves != 0 {
                let to = quiet_moves.trailing_zeros() as u8;
                quiet_moves ^= 1u64 << to;
                moves.push_quiet(from, to, FLAG_QUIET);
            }
            while captures != 0 {
                let to = captures.trailing_zeros() as u8;
                captures ^= 1u64 << to;
                moves.push_capture(from, to, FLAG_CAPTURE);
            }
        }

        // Queens
        let mut queens = self.friendly_board(Queens);
        while queens != 0 {
            let from = queens.trailing_zeros() as u8;
            queens ^= 1u64 << from;

            let attacks = unsafe {
                ROOK_ATTACKS[from as usize][_pext_u64(occupied, ROOK_MASKS[from as usize]) as usize]
                    | BISHOP_ATTACKS[from as usize]
                        [_pext_u64(occupied, BISHOP_MASKS[from as usize]) as usize]
            };

            let mut quiet_moves = attacks & !occupied;
            let mut captures = attacks & enemies;

            while quiet_moves != 0 {
                let to = quiet_moves.trailing_zeros() as u8;
                quiet_moves ^= 1u64 << to;
                moves.push_quiet(from, to, FLAG_QUIET);
            }
            while captures != 0 {
                let to = captures.trailing_zeros() as u8;
                captures ^= 1u64 << to;
                moves.push_capture(from, to, FLAG_CAPTURE);
            }
        }
    }

    #[inline(always)]
    fn get_knight_moves(&self, moves: &mut MoveList, friendlies: u64, enemies: u64) {
        let mut knights = self.friendly_board(Knights);
        while knights != 0 {
            let from = knights.trailing_zeros() as u8;
            knights ^= 1u64 << from;

            let attacks = KNIGHT_ATTACKS[from as usize];
            let mut quiet_moves = attacks & !(friendlies | enemies);
            let mut captures = attacks & enemies;

            while quiet_moves != 0 {
                let to = quiet_moves.trailing_zeros() as u8;
                quiet_moves ^= 1u64 << to;
                moves.push_quiet(from, to, FLAG_QUIET);
            }
            while captures != 0 {
                let to = captures.trailing_zeros() as u8;
                captures ^= 1u64 << to;
                moves.push_capture(from, to, FLAG_CAPTURE);
            }
        }
    }

    #[inline(always)]
    fn get_king_moves(&self, moves: &mut MoveList, friendlies: u64, enemies: u64, occupied: u64) {
        let king = self.friendly_board(King);
        let from = king.trailing_zeros() as u8;

        let attacks = KING_ATTACKS[from as usize];
        let mut quiet_moves = attacks & !(friendlies | enemies);
        let mut captures = attacks & enemies;

        while quiet_moves != 0 {
            let to = quiet_moves.trailing_zeros() as u8;
            quiet_moves ^= 1u64 << to;
            moves.push_quiet(from, to, FLAG_QUIET);
        }
        while captures != 0 {
            let to = captures.trailing_zeros() as u8;
            captures ^= 1u64 << to;
            moves.push_capture(from, to, FLAG_CAPTURE);
        }

        // Add castling moves
        self.get_castling_moves(moves, occupied);
    }

    #[inline(always)]
    fn get_castling_moves(&self, moves: &mut MoveList, occupied: u64) {
        if self.turn == 0 {
            // White castling

            // Kingside castling (O-O)
            if (self.castling_rights & WHITE_KINGSIDE) != 0
                && (occupied & WHITE_KINGSIDE_SQUARES) == 0
            {
                // King is on e1 (4), moves to g1 (6)
                if self.can_castle_through_squares(&[4, 5, 6]) {
                    moves.push_quiet(WHITE_KING_START, 6, FLAG_CASTLE_KINGSIDE);
                }
            }

            // Queenside castling (O-O-O)
            if (self.castling_rights & WHITE_QUEENSIDE) != 0
                && (occupied & WHITE_QUEENSIDE_SQUARES) == 0
            {
                // King is on e1 (4), moves to c1 (2)
                if self.can_castle_through_squares(&[4, 3, 2]) {
                    moves.push_quiet(WHITE_KING_START, 2, FLAG_CASTLE_QUEENSIDE);
                }
            }
        } else {
            // Black castling

            // Kingside castling (O-O)
            if (self.castling_rights & BLACK_KINGSIDE) != 0
                && (occupied & BLACK_KINGSIDE_SQUARES) == 0
            {
                // King is on e8 (60), moves to g8 (62)
                if self.can_castle_through_squares(&[60, 61, 62]) {
                    moves.push_quiet(BLACK_KING_START, 62, FLAG_CASTLE_KINGSIDE);
                }
            }

            // Queenside castling (O-O-O)
            if (self.castling_rights & BLACK_QUEENSIDE) != 0
                && (occupied & BLACK_QUEENSIDE_SQUARES) == 0
            {
                // King is on e8 (60), moves to c8 (58)
                if self.can_castle_through_squares(&[60, 59, 58]) {
                    moves.push_quiet(BLACK_KING_START, 58, FLAG_CASTLE_QUEENSIDE);
                }
            }
        }
    }

    fn is_square_attacked(&self, square: u8) -> bool {
        let target = 1u64 << square;
        let enemy_turn = self.turn ^ 1; // flip to enemy

        // Pawns
        let pawns = self.boards[(Piece::Pawns as usize) + (enemy_turn as usize * 6)].0;
        let pawn_attacks = if enemy_turn == 0 {
            ((pawns & !FILE_A) << 7) | ((pawns & !FILE_H) << 9)
        } else {
            ((pawns & !FILE_H) >> 7) | ((pawns & !FILE_A) >> 9)
        };
        if pawn_attacks & target != 0 {
            return true;
        }

        // Knights
        let knights = self.boards[(Piece::Knights as usize) + (enemy_turn as usize * 6)].0;
        let mut ktmp = knights;
        while ktmp != 0 {
            let from = ktmp.trailing_zeros() as u8;
            ktmp &= ktmp - 1;
            if KNIGHT_ATTACKS[from as usize] & target != 0 {
                return true;
            }
        }

        // Kings
        let king = self.boards[(Piece::King as usize) + (enemy_turn as usize * 6)].0;
        let from = king.trailing_zeros() as u8;
        if KING_ATTACKS[from as usize] & target != 0 {
            return true;
        }

        // Sliding pieces
        let occupied = self.occupied();
        // Bishops & Queens
        let mut bishops = self.boards[(Piece::Bishops as usize) + (enemy_turn as usize * 6)].0
            | self.boards[(Piece::Queens as usize) + (enemy_turn as usize * 6)].0;
        while bishops != 0 {
            let from = bishops.trailing_zeros() as u8;
            bishops &= bishops - 1;
            let attacks = unsafe {
                BISHOP_ATTACKS[from as usize]
                    [_pext_u64(occupied, BISHOP_MASKS[from as usize]) as usize]
            };
            if attacks & target != 0 {
                return true;
            }
        }

        // Rooks & Queens
        let mut rooks = self.boards[(Piece::Rooks as usize) + (enemy_turn as usize * 6)].0
            | self.boards[(Piece::Queens as usize) + (enemy_turn as usize * 6)].0;
        while rooks != 0 {
            let from = rooks.trailing_zeros() as u8;
            rooks &= rooks - 1;
            let attacks = unsafe {
                ROOK_ATTACKS[from as usize][_pext_u64(occupied, ROOK_MASKS[from as usize]) as usize]
            };
            if attacks & target != 0 {
                return true;
            }
        }

        false
    }

    /// Check if the king can safely castle through the given squares
    /// This is a placeholder - you'll need to implement attack detection
    #[inline(always)]
    fn can_castle_through_squares(&self, squares: &[u8]) -> bool {
        // TODO: Implement proper attack detection
        // For now, return true - you'll need to check:
        // 1. King is not in check on starting square
        // 2. King doesn't pass through check on intermediate squares
        // 3. King doesn't end up in check on destination square

        // This requires implementing an "is_square_attacked" function
        // which checks if enemy pieces can attack a given square
        if self.is_square_attacked(squares[0]) {
            return false;
        }
        for &sq in &squares[1..] {
            if self.is_square_attacked(sq) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod test_generate_moves {
    use crate::game::Game;
    use std::time::Instant;
    use types::{move_type::MoveList, piece_kind::PieceKind};
    use utilities::algebraic::Algebraic;
    #[test]
    fn bench_get_all_moves() {
        let g = Game::new();
        let mut m = MoveList::new();

        let iterations = 10_000_000;
        let start = Instant::now();

        for _ in 0..iterations {
            m.clear();
            g.get_all_moves(&mut m);
        }

        let elapsed = start.elapsed();
        println!(
            "Generated all moves {} times in {:?} ({:.2} ns per call)",
            iterations,
            elapsed,
            (elapsed.as_nanos() as f64) / (iterations as f64)
        );

        m.clear();
        g.get_all_moves(&mut m);
        println!("Generated {} moves from starting position", m.total_count());
        m.print();
    }

    #[test]
    fn test_castling() {
        let mut g = Game::new();
        let mut m = MoveList::new();
        g.boards[PieceKind::WhiteBishop].remove("f1".idx() as usize);
        g.boards[PieceKind::WhiteKnight].remove("g1".idx() as usize);
        g.boards[PieceKind::WhitePawn].remove("f2".idx() as usize);
        // g.boards[PieceKind::BlackRook].set("f6".idx() as usize);
        g.board_map["f2".idx() as usize] = PieceKind::None;
        g.board_map["f6".idx() as usize] = PieceKind::BlackRook;
        g.get_all_moves(&mut m);
        m.print();
    }
}
