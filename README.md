use game::{game::Game, piece::Piece};
use crate::{
    pins_checks::{find_pins_and_checks, get_rook_attacks, get_bishop_attacks, get_queen_attacks},
    moves::{Move, NONE, PROMO_QUEEN, PROMO_ROOK, PROMO_BISHOP, PROMO_KNIGHT, CAPT, CASTLE, ENPASS},
    PAWN_ATTACKS, KING_ATTACKS, KNIGHT_ATTACKS,
};

pub struct MoveGenerator;

impl MoveGenerator {
    pub fn generate_legal_moves(game: &Game) -> Vec<Move> {
        let mut moves = Vec::new();
        
        let (pinned_pieces, checking_pieces, check_mask) = find_pins_and_checks(game);
        let is_in_check = checking_pieces != 0;
        let is_double_check = checking_pieces.count_ones() > 1;
        
        // In double check, only king moves are legal
        if is_double_check {
            Self::generate_king_moves(game, &mut moves);
            return moves;
        }
        
        let our_pieces = game.get_all_friendlies();
        let enemy_pieces = game.get_all_enemies();
        let all_pieces = our_pieces | enemy_pieces;
        
        // Generate moves for each piece type
        Self::generate_pawn_moves(game, &mut moves, pinned_pieces, check_mask);
        Self::generate_knight_moves(game, &mut moves, pinned_pieces, check_mask);
        Self::generate_bishop_moves(game, &mut moves, pinned_pieces, check_mask, all_pieces);
        Self::generate_rook_moves(game, &mut moves, pinned_pieces, check_mask, all_pieces);
        Self::generate_queen_moves(game, &mut moves, pinned_pieces, check_mask, all_pieces);
        Self::generate_king_moves(game, &mut moves);
        
        // Castling (only if not in check)
        if !is_in_check {
            Self::generate_castle_moves(game, &mut moves);
        }
        
        moves
    }
    
    fn generate_pawn_moves(game: &Game, moves: &mut Vec<Move>, pinned_pieces: u64, check_mask: u64) {
        let our_pawns = game.friendly_board(Piece::Pawn);
        let enemy_pieces = game.get_all_enemies();
        let all_pieces = game.get_all_friendlies() | enemy_pieces;
        
        let forward_dir = if game.turn == 0 { 8 } else { -8i8 as u8 as usize };
        let start_rank = if game.turn == 0 { 1 } else { 6 };
        let promo_rank = if game.turn == 0 { 7 } else { 0 };
        
        let mut pawns = our_pawns;
        while pawns != 0 {
            let from_sq = pawns.trailing_zeros() as usize;
            pawns &= pawns - 1; // Clear the least significant bit
            
            let from_bit = 1u64 << from_sq;
            let is_pinned = (pinned_pieces & from_bit) != 0;
            
            // Single pawn push
            let to_sq = from_sq + forward_dir;
            if to_sq < 64 && (all_pieces & (1u64 << to_sq)) == 0 {
                let to_bit = 1u64 << to_sq;
                if !is_pinned || Self::move_along_pin_ray(game, from_sq, to_sq, pinned_pieces) {
                    if (to_bit & check_mask) != 0 {
                        if to_sq / 8 == promo_rank {
                            // Promotion
                            moves.push(Move::new(from_sq as u16, to_sq as u16, PROMO_QUEEN));
                            moves.push(Move::new(from_sq as u16, to_sq as u16, PROMO_ROOK));
                            moves.push(Move::new(from_sq as u16, to_sq as u16, PROMO_BISHOP));
                            moves.push(Move::new(from_sq as u16, to_sq as u16, PROMO_KNIGHT));
                        } else {
                            moves.push(Move::new(from_sq as u16, to_sq as u16, NONE));
                        }
                    }
                }
                
                // Double pawn push
                if from_sq / 8 == start_rank {
                    let double_to_sq = to_sq + forward_dir;
                    if double_to_sq < 64 && (all_pieces & (1u64 << double_to_sq)) == 0 {
                        let double_to_bit = 1u64 << double_to_sq;
                        if !is_pinned || Self::move_along_pin_ray(game, from_sq, double_to_sq, pinned_pieces) {
                            if (double_to_bit & check_mask) != 0 {
                                moves.push(Move::new(from_sq as u16, double_to_sq as u16, NONE));
                            }
                        }
                    }
                }
            }
            
            // Pawn captures
            let attacks = PAWN_ATTACKS[game.turn as usize][from_sq];
            let mut captures = attacks & enemy_pieces;
            
            while captures != 0 {
                let to_sq = captures.trailing_zeros() as usize;
                captures &= captures - 1;
                
                let to_bit = 1u64 << to_sq;
                if !is_pinned || Self::move_along_pin_ray(game, from_sq, to_sq, pinned_pieces) {
                    if (to_bit & check_mask) != 0 {
                        if to_sq / 8 == promo_rank {
                            // Capture promotion
                            moves.push(Move::new(from_sq as u16, to_sq as u16, CAPT | PROMO_QUEEN));
                            moves.push(Move::new(from_sq as u16, to_sq as u16, CAPT | PROMO_ROOK));
                            moves.push(Move::new(from_sq as u16, to_sq as u16, CAPT | PROMO_BISHOP));
                            moves.push(Move::new(from_sq as u16, to_sq as u16, CAPT | PROMO_KNIGHT));
                        } else {
                            moves.push(Move::new(from_sq as u16, to_sq as u16, CAPT));
                        }
                    }
                }
            }
            
            // En passant
            if let Some(ep_sq) = game.en_passant_square() {
                if (attacks & (1u64 << ep_sq)) != 0 {
                    let ep_bit = 1u64 << ep_sq;
                    if !is_pinned || Self::move_along_pin_ray(game, from_sq, ep_sq, pinned_pieces) {
                        if (ep_bit & check_mask) != 0 {
                            moves.push(Move::new(from_sq as u16, ep_sq as u16, ENPASS));
                        }
                    }
                }
            }
        }
    }
    
    fn generate_knight_moves(game: &Game, moves: &mut Vec<Move>, pinned_pieces: u64, check_mask: u64) {
        let our_knights = game.friendly_board(Piece::Knight);
        let our_pieces = game.get_all_friendlies();
        
        let mut knights = our_knights;
        while knights != 0 {
            let from_sq = knights.trailing_zeros() as usize;
            knights &= knights - 1;
            
            // Pinned knights cannot move (knights always change both rank and file)
            if (pinned_pieces & (1u64 << from_sq)) != 0 {
                continue;
            }
            
            let attacks = KNIGHT_ATTACKS[from_sq] & !our_pieces & check_mask;
            Self::add_moves_from_bitboard(moves, from_sq, attacks, game);
        }
    }
    
    fn generate_bishop_moves(game: &Game, moves: &mut Vec<Move>, pinned_pieces: u64, check_mask: u64, all_pieces: u64) {
        let our_bishops = game.friendly_board(Piece::Bishop);
        let our_pieces = game.get_all_friendlies();
        
        let mut bishops = our_bishops;
        while bishops != 0 {
            let from_sq = bishops.trailing_zeros() as usize;
            bishops &= bishops - 1;
            
            let from_bit = 1u64 << from_sq;
            let attacks = get_bishop_attacks(from_sq, all_pieces) & !our_pieces;
            
            let legal_attacks = if (pinned_pieces & from_bit) != 0 {
                Self::filter_pinned_moves(game, from_sq, attacks, pinned_pieces)
            } else {
                attacks
            } & check_mask;
            
            Self::add_moves_from_bitboard(moves, from_sq, legal_attacks, game);
        }
    }
    
    fn generate_rook_moves(game: &Game, moves: &mut Vec<Move>, pinned_pieces: u64, check_mask: u64, all_pieces: u64) {
        let our_rooks = game.friendly_board(Piece::Rook);
        let our_pieces = game.get_all_friendlies();
        
        let mut rooks = our_rooks;
        while rooks != 0 {
            let from_sq = rooks.trailing_zeros() as usize;
            rooks &= rooks - 1;
            
            let from_bit = 1u64 << from_sq;
            let attacks = get_rook_attacks(from_sq, all_pieces) & !our_pieces;
            
            let legal_attacks = if (pinned_pieces & from_bit) != 0 {
                Self::filter_pinned_moves(game, from_sq, attacks, pinned_pieces)
            } else {
                attacks
            } & check_mask;
            
            Self::add_moves_from_bitboard(moves, from_sq, legal_attacks, game);
        }
    }
    
    fn generate_queen_moves(game: &Game, moves: &mut Vec<Move>, pinned_pieces: u64, check_mask: u64, all_pieces: u64) {
        let our_queens = game.friendly_board(Piece::Queen);
        let our_pieces = game.get_all_friendlies();
        
        let mut queens = our_queens;
        while queens != 0 {
            let from_sq = queens.trailing_zeros() as usize;
            queens &= queens - 1;
            
            let from_bit = 1u64 << from_sq;
            let attacks = get_queen_attacks(from_sq, all_pieces) & !our_pieces;
            
            let legal_attacks = if (pinned_pieces & from_bit) != 0 {
                Self::filter_pinned_moves(game, from_sq, attacks, pinned_pieces)
            } else {
                attacks
            } & check_mask;
            
            Self::add_moves_from_bitboard(moves, from_sq, legal_attacks, game);
        }
    }
    
    fn generate_king_moves(game: &Game, moves: &mut Vec<Move>) {
        let king_sq = game.friendly_board(Piece::King).trailing_zeros() as usize;
        let our_pieces = game.get_all_friendlies();
        let enemy_pieces = game.get_all_enemies();
        let all_pieces = our_pieces | enemy_pieces;
        
        let attacks = KING_ATTACKS[king_sq] & !our_pieces;
        
        let mut king_moves = attacks;
        while king_moves != 0 {
            let to_sq = king_moves.trailing_zeros() as usize;
            king_moves &= king_moves - 1;
            
            // Check if this square is attacked by enemy
            if !Self::is_square_attacked_by_enemy(game, to_sq, all_pieces & !(1u64 << king_sq)) {
                let flags = if (enemy_pieces & (1u64 << to_sq)) != 0 { CAPT } else { NONE };
                moves.push(Move::new(king_sq as u16, to_sq as u16, flags));
            }
        }
    }
    
    fn generate_castle_moves(game: &Game, moves: &mut Vec<Move>) {
        // Implementation depends on your castle rights tracking
        // This is a simplified version
        if game.can_castle_kingside() {
            let king_sq = if game.turn == 0 { 4 } else { 60 };
            let rook_sq = if game.turn == 0 { 7 } else { 63 };
            let king_to = if game.turn == 0 { 6 } else { 62 };
            
            // Check if path is clear and not attacked
            let path = if game.turn == 0 { 0x60u64 } else { 0x6000000000000000u64 };
            let all_pieces = game.get_all_friendlies() | game.get_all_enemies();
            
            if (path & all_pieces) == 0 &&
               !Self::is_square_attacked_by_enemy(game, king_sq + 1, all_pieces) &&
               !Self::is_square_attacked_by_enemy(game, king_to, all_pieces) {
                moves.push(Move::new(king_sq as u16, king_to as u16, CASTLE));
            }
        }
        
        // Similar for queenside...
    }
    
    // Helper functions
    fn add_moves_from_bitboard(moves: &mut Vec<Move>, from_sq: usize, attacks: u64, game: &Game) {
        let enemy_pieces = game.get_all_enemies();
        let mut bb = attacks;
        while bb != 0 {
            let to_sq = bb.trailing_zeros() as usize;
            bb &= bb - 1;
            
            let flags = if (enemy_pieces & (1u64 << to_sq)) != 0 { CAPT } else { NONE };
            moves.push(Move::new(from_sq as u16, to_sq as u16, flags));
        }
    }
    
    fn move_along_pin_ray(game: &Game, from_sq: usize, to_sq: usize, pinned_pieces: u64) -> bool {
        let king_sq = game.friendly_board(Piece::King).trailing_zeros() as usize;
        
        // Find which direction the pinned piece is from the king
        for direction in 0..8 {
            let ray = RAY_ATTACKS[direction][king_sq];
            if (ray & (1u64 << from_sq)) != 0 {
                // This is the direction - check if to_sq is also on this ray
                return (ray & (1u64 << to_sq)) != 0;
            }
        }
        false
    }
    
    fn filter_pinned_moves(game: &Game, from_sq: usize, attacks: u64, pinned_pieces: u64) -> u64 {
        let king_sq = game.friendly_board(Piece::King).trailing_zeros() as usize;
        
        // Find the ray direction for this pinned piece
        for direction in 0..8 {
            let ray = RAY_ATTACKS[direction][king_sq];
            if (ray & (1u64 << from_sq)) != 0 {
                // Filter attacks to only those on this ray
                return attacks & ray;
            }
        }
        attacks // Fallback (shouldn't happen)
    }
    
    fn is_square_attacked_by_enemy(game: &Game, square: usize, all_pieces: u64) -> bool {
        let enemy_turn = 1 - game.turn;
        
        // Check pawn attacks
        let enemy_pawns = game.enemy_board(Piece::Pawn);
        if (PAWN_ATTACKS[enemy_turn as usize][square] & enemy_pawns) != 0 {
            return true;
        }
        
        // Check knight attacks
        let enemy_knights = game.enemy_board(Piece::Knight);
        if (KNIGHT_ATTACKS[square] & enemy_knights) != 0 {
            return true;
        }
        
        // Check king attacks (for adjacent squares)
        let enemy_king = game.enemy_board(Piece::King);
        if (KING_ATTACKS[square] & enemy_king) != 0 {
            return true;
        }
        
        // Check sliding piece attacks
        let enemy_bishops = game.enemy_board(Piece::Bishop);
        let enemy_rooks = game.enemy_board(Piece::Rook);
        let enemy_queens = game.enemy_board(Piece::Queen);
        
        // Bishop/Queen diagonal attacks
        let bishop_attacks = get_bishop_attacks(square, all_pieces);
        if (bishop_attacks & (enemy_bishops | enemy_queens)) != 0 {
            return true;
        }
        
        // Rook/Queen straight attacks  
        let rook_attacks = get_rook_attacks(square, all_pieces);
        if (rook_attacks & (enemy_rooks | enemy_queens)) != 0 {
            return true;
        }
        
        false
    }
}
