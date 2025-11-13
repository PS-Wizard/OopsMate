use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::LazyLock;
use types::others::{CastleRights, Color, Piece};

pub struct ZobristTables {
    pub pieces: [[u64; 64]; 12],
    pub castling: [u64; 4],
    pub en_passant: [u64; 8],
    pub side_to_move: u64,
}

impl ZobristTables {
    pub fn generate() -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(0x5EED_C0DE_CAFE_BABE);
        
        // Generate Piece Square Tables
        let mut pieces = [[0u64; 64]; 12];
        for piece in 0..12 {
            for square in 0..64 {
                pieces[piece][square] = rng.r#gen();
            } }
        
        // Generate Castling Rights
        let mut castling_rights = [0u64; 4];
        for i in 0..4 {
            castling_rights[i] = rng.r#gen();
        }
        
        // Generate En-Passant files
        let mut enpassant_files = [0u64; 8];
        for i in 0..8 {
            enpassant_files[i] = rng.r#gen();
        }
        
        Self {
            pieces,
            castling: castling_rights,
            en_passant: enpassant_files,
            side_to_move: rng.r#gen(),
        }
    }
    
    /// Get the Zobrist for a piece on a square
    #[inline(always)]
    pub fn piece(&self, piece: Piece, color: Color, square: usize) -> u64 {
        let piece_idx = Self::piece_index(piece, color);
        self.pieces[piece_idx][square]
    }
    
    /// Convert Piece + Color to table index
    #[inline(always)]
    fn piece_index(piece: Piece, color: Color) -> usize {
        let base = piece as usize;
        match color {
            Color::White => base,
            Color::Black => base + 6,
        }
    }
    
    /// Get Zobrist key for castling rights
    #[inline(always)]
    pub fn castling_key(&self, rights: CastleRights) -> u64 {
        let mut key = 0u64;
        
        if rights.0 & CastleRights::WHITE_KING.0 != 0 {
            key ^= self.castling[0];
        }
        if rights.0 & CastleRights::WHITE_QUEEN.0 != 0 {
            key ^= self.castling[1];
        }
        if rights.0 & CastleRights::BLACK_KING.0 != 0 {
            key ^= self.castling[2];
        }
        if rights.0 & CastleRights::BLACK_QUEEN.0 != 0 {
            key ^= self.castling[3];
        }
        
        key
    }
    
    #[inline(always)]
    pub fn en_passant_key(&self, file: usize) -> u64 {
        self.en_passant[file]
    }
}

pub static ZOBRIST: LazyLock<ZobristTables> = LazyLock::new(|| {
    ZobristTables::generate()
});
