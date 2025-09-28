use pext::PAWN_ATTACKS;
use pext::KING_ATTACKS;
use utilities::board::PrintAsBoard;

use crate::{
    game::Game,
    piece::Piece::*,
};

impl Game {
    fn generate_king_moves(&self, _pinned: u64, _check_mask: u64) {
        // King moves ignore pinned and check_mask parameters because:
        // 1. Kings can't be pinned (would be in check instead)
        // 2. Kings must avoid moving into check regardless of check_mask
        
        let king_sq = self.friendly_board(King).trailing_zeros() as usize;
        let friendly_pieces = self.get_all_friendlies();
        
        // Get all possible king moves (one square in any direction)
        let possible_moves = KING_ATTACKS[king_sq] & !friendly_pieces;
        
        // Filter out moves that would put king in check
        let legal_moves = self.filter_safe_king_moves(possible_moves, king_sq);
        
        println!("King Moves for king on {king_sq}:");
        legal_moves.print();
        
        // Handle castling separately
        let castling_moves = self.generate_castling_moves();
        if castling_moves != 0 {
            println!("Castling moves:");
            castling_moves.print();
        }
    }
    
    /// Filters out king moves that would put the king in check
    fn filter_safe_king_moves(&self, possible_moves: u64, king_sq: usize) -> u64 {
        let mut safe_moves = 0u64;
        let mut moves = possible_moves;
        
        while moves != 0 {
            let to_sq = moves.trailing_zeros() as usize;
            moves &= moves - 1;
            
            if self.is_square_safe_for_king(to_sq, king_sq) {
                safe_moves |= 1u64 << to_sq;
            }
        }
        
        safe_moves
    }
    
    /// Checks if a square is safe for the king to move to
    fn is_square_safe_for_king(&self, to_sq: usize, from_sq: usize) -> bool {
        // Temporarily move the king and check if the new position is attacked
        let enemy_pieces = self.get_all_enemies();
        let all_pieces = (self.white_occupied | self.black_occupied) 
            & !(1u64 << from_sq)  // Remove king from current position
            | (1u64 << to_sq);    // Add king to new position
        
        // Check if any enemy piece can attack the new king position
        !self.is_square_attacked_by_enemy(to_sq, enemy_pieces, all_pieces)
    }
    
    /// Checks if a square is attacked by enemy pieces
    fn is_square_attacked_by_enemy(&self, square: usize, enemy_pieces: u64, all_pieces: u64) -> bool {
        // Check for enemy pawn attacks
        if (PAWN_ATTACKS[self.turn as usize][square] & self.enemy_board(Pawn)) != 0 {
            return true;
        }
        
        // Check for enemy knight attacks
        if (KING_ATTACKS[square] & self.enemy_board(Knight)) != 0 {
            return true;
        }
        
        // Check for enemy king attacks (kings can't be adjacent)
        if (KING_ATTACKS[square] & self.enemy_board(King)) != 0 {
            return true;
        }
        
        // Check for enemy sliding piece attacks (rooks, bishops, queens)
        self.is_square_attacked_by_sliders(square, enemy_pieces, all_pieces)
    }
    
    /// Checks if a square is attacked by enemy sliding pieces
    fn is_square_attacked_by_sliders(&self, square: usize, _enemy_pieces: u64, all_pieces: u64) -> bool {
        use crate::pins_checks::{RAY_ATTACKS, direction_consts::*};
        
        // Check all 8 directions for attacking sliders
        for direction in [TOP, TOP_RIGHT, RIGHT, BOTTOM_RIGHT, BOTTOM, BOTTOM_LEFT, LEFT, TOP_LEFT] {
            let ray = RAY_ATTACKS[direction][square];
            let pieces_on_ray = ray & all_pieces;
            
            if pieces_on_ray != 0 {
                // Find the first piece in this direction
                let first_piece_sq = if self.is_direction_increasing(direction) {
                    pieces_on_ray.trailing_zeros() as usize
                } else {
                    63 - pieces_on_ray.leading_zeros() as usize
                };
                
                // Check if it's an enemy slider that can attack in this direction
                if self.is_enemy_slider_attacking_square(first_piece_sq, direction) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Checks if an enemy piece at the given square can attack in the given direction
    fn is_enemy_slider_attacking_square(&self, piece_sq: usize, direction: usize) -> bool {
        use crate::piece::PieceKind::*;
        use crate::pins_checks::direction_consts::*;
        
        let piece = self.piece_at(piece_sq);
        
        // Check if it's an enemy piece
        let is_enemy = match self.turn {
            0 => matches!(piece, BlackRook | BlackBishop | BlackQueen),  // White's turn, black is enemy
            1 => matches!(piece, WhiteRook | WhiteBishop | WhiteQueen),  // Black's turn, white is enemy
            _ => false,
        };
        
        if !is_enemy {
            return false;
        }
        
        // Check if this piece type can attack in this direction
        match direction {
            TOP | BOTTOM | RIGHT | LEFT => {
                matches!(piece, WhiteRook | BlackRook | WhiteQueen | BlackQueen)
            }
            TOP_RIGHT | BOTTOM_LEFT | TOP_LEFT | BOTTOM_RIGHT => {
                matches!(piece, WhiteBishop | BlackBishop | WhiteQueen | BlackQueen)
            }
            _ => false,
        }
    }
    
    fn is_direction_increasing(&self, direction: usize) -> bool {
        use crate::pins_checks::direction_consts::*;
        match direction {
            TOP | TOP_RIGHT | RIGHT | TOP_LEFT => true,
            BOTTOM | BOTTOM_LEFT | LEFT | BOTTOM_RIGHT => false,
            _ => true,
        }
    }
    
    /// Generates castling moves if legal
    fn generate_castling_moves(&self) -> u64 {
        let mut castling_moves = 0u64;
        
        if self.turn == 0 {
            // White castling
            if (self.castling_rights & 0b0001) != 0 {  // White kingside
                if self.can_castle_kingside() {
                    castling_moves |= 1u64 << 6; // g1
                }
            }
            if (self.castling_rights & 0b0010) != 0 {  // White queenside
                if self.can_castle_queenside() {
                    castling_moves |= 1u64 << 2; // c1
                }
            }
        } else {
            // Black castling
            if (self.castling_rights & 0b0100) != 0 {  // Black kingside
                if self.can_castle_kingside() {
                    castling_moves |= 1u64 << 62; // g8
                }
            }
            if (self.castling_rights & 0b1000) != 0 {  // Black queenside
                if self.can_castle_queenside() {
                    castling_moves |= 1u64 << 58; // c8
                }
            }
        }
        
        castling_moves
    }
    
    fn can_castle_kingside(&self) -> bool {
        let king_sq = self.friendly_board(King).trailing_zeros() as usize;
        let all_pieces = self.white_occupied | self.black_occupied;
        
        let (f_sq, g_sq) = if self.turn == 0 { (5, 6) } else { (61, 62) };  // f1,g1 or f8,g8
        
        // Check squares are empty
        if (all_pieces & ((1u64 << f_sq) | (1u64 << g_sq))) != 0 {
            return false;
        }
        
        // Check king and squares it passes through are not in check
        self.is_square_safe_for_king(king_sq, king_sq) &&
        self.is_square_safe_for_king(f_sq, king_sq) &&
        self.is_square_safe_for_king(g_sq, king_sq)
    }
    
    fn can_castle_queenside(&self) -> bool {
        let king_sq = self.friendly_board(King).trailing_zeros() as usize;
        let all_pieces = self.white_occupied | self.black_occupied;
        
        let (b_sq, c_sq, d_sq) = if self.turn == 0 { (1, 2, 3) } else { (57, 58, 59) };  // b1,c1,d1 or b8,c8,d8
        
        // Check squares are empty (b square doesn't need to be safe, just empty)
        if (all_pieces & ((1u64 << b_sq) | (1u64 << c_sq) | (1u64 << d_sq))) != 0 {
            return false;
        }
        
        // Check king and squares it passes through are not in check
        self.is_square_safe_for_king(king_sq, king_sq) &&
        self.is_square_safe_for_king(c_sq, king_sq) &&
        self.is_square_safe_for_king(d_sq, king_sq)
    }
}

#[cfg(test)]
mod test_king_legal {
    use crate::{game::Game, pins_checks::pin_check_finder::find_pins_n_checks};

    #[test]
    fn test_king_legal() {
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", // Starting position
            "8/8/8/8/8/8/8/4K3 w - - 0 1", // King alone
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", // Castling test
            "8/8/8/3qk3/8/8/8/3K4 w - - 0 1", // King under attack
            "k7/8/8/5q2/8/8/8/R3K2R w KQ - 0 1", // King Shouldnt be able to castle kingside
            "k7/8/8/2r2q2/8/8/8/R3K2R w KQ - 0 1", // King Shouldnt be able to castle both king or
                                                   // queen side
            "k7/8/8/8/8/8/8/1R2K2R w K - 0 1", // No queen side castling cause the rook moved or
                                               // somethin
        ];
        
        for position in positions {
            println!("================");
            let g = Game::from_fen(position);
            let (pinned, _checking, check_mask) = find_pins_n_checks(&g);
            println!("Position: {}", position);
            g.generate_king_moves(pinned, check_mask);
            println!("================");
        }
    }
}
