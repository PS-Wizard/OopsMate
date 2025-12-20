use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::sync::Once;

static INIT: Once = Once::new();

// FFI declarations matching the C library
#[repr(C)]
#[derive(Debug)]
pub struct DirtyPiece {
    pub dirty_num: c_int,
    pub pc: [c_int; 3],
    pub from: [c_int; 3],
    pub to: [c_int; 3],
}

#[repr(C, align(64))]
#[derive(Debug)]
pub struct Accumulator {
    pub accumulation: [[i16; 256]; 2],
    pub computed_accumulation: c_int,
}

#[repr(C)]
#[derive(Debug)]
pub struct NNUEData {
    pub accumulator: Accumulator,
    pub dirty_piece: DirtyPiece,
}

impl NNUEData {
    pub fn new() -> Self {
        NNUEData {
            accumulator: Accumulator {
                accumulation: [[0; 256]; 2],
                computed_accumulation: 0,
            },
            dirty_piece: DirtyPiece {
                dirty_num: 0,
                pc: [0; 3],
                from: [0; 3],
                to: [0; 3],
            },
        }
    }
}

impl Default for NNUEData {
    fn default() -> Self {
        Self::new()
    }
}

// C piece representation (matching the library)
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CPiece {
    Blank = 0,
    WKing = 1,
    WQueen = 2,
    WRook = 3,
    WBishop = 4,
    WKnight = 5,
    WPawn = 6,
    BKing = 7,
    BQueen = 8,
    BRook = 9,
    BBishop = 10,
    BKnight = 11,
    BPawn = 12,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CColor {
    White = 0,
    Black = 1,
}

unsafe extern "C" {
    fn nnue_init(eval_file: *const c_char);

    fn nnue_evaluate(player: c_int, pieces: *mut c_int, squares: *mut c_int) -> c_int;

    fn nnue_evaluate_incremental(
        player: c_int,
        pieces: *mut c_int,
        squares: *mut c_int,
        nnue_data: *mut *mut NNUEData,
    ) -> c_int;

    fn nnue_evaluate_fen(fen: *const c_char) -> c_int;
}

pub struct NNUEProbe {
    initialized: bool,
}

impl NNUEProbe {
    pub fn new() -> Self {
        NNUEProbe { initialized: false }
    }

    /// Initialize the NNUE library with the evaluation file
    /// Note: This can only be called once per process due to C library limitations
    pub fn init(&mut self, eval_file_path: &str) -> Result<(), String> {
        let c_path = CString::new(eval_file_path).map_err(|e| format!("Invalid path: {}", e))?;

        let result = Ok(());
        INIT.call_once(|| unsafe {
            nnue_init(c_path.as_ptr());
        });

        self.initialized = true;
        result
    }

    /// Evaluate a position from FEN string
    /// Returns score in centipawns relative to side to move
    pub fn evaluate_fen(&self, fen: &str) -> Result<i32, String> {
        if !self.initialized {
            return Err("NNUE not initialized".to_string());
        }

        let c_fen = CString::new(fen).map_err(|e| format!("Invalid FEN: {}", e))?;

        let score = unsafe { nnue_evaluate_fen(c_fen.as_ptr()) };

        Ok(score)
    }

    /// Evaluate a position with piece and square arrays
    /// pieces[0] = white king piece code, squares[0] = white king square
    /// pieces[1] = black king piece code, squares[1] = black king square
    /// pieces[n+1] = 0 marks end of arrays
    /// Squares: A1=0, B1=1, ..., H8=63
    pub fn evaluate(
        &self,
        player: CColor,
        pieces: &mut [i32],
        squares: &mut [i32],
    ) -> Result<i32, String> {
        if !self.initialized {
            return Err("NNUE not initialized".to_string());
        }

        if pieces.len() != squares.len() {
            return Err("pieces and squares arrays must have same length".to_string());
        }

        let score =
            unsafe { nnue_evaluate(player as c_int, pieces.as_mut_ptr(), squares.as_mut_ptr()) };

        Ok(score)
    }

    /// Incremental evaluation with history
    /// nnue_data[0] = current position
    /// nnue_data[1] = position at ply-1
    /// nnue_data[2] = position at ply-2
    pub fn evaluate_incremental(
        &self,
        player: CColor,
        pieces: &mut [i32],
        squares: &mut [i32],
        nnue_data: &mut [*mut NNUEData; 3],
    ) -> Result<i32, String> {
        if !self.initialized {
            return Err("NNUE not initialized".to_string());
        }

        if pieces.len() != squares.len() {
            return Err("pieces and squares arrays must have same length".to_string());
        }

        let score = unsafe {
            nnue_evaluate_incremental(
                player as c_int,
                pieces.as_mut_ptr(),
                squares.as_mut_ptr(),
                nnue_data.as_mut_ptr(),
            )
        };

        Ok(score)
    }
}

impl Default for NNUEProbe {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fen_evaluation() {
        let mut probe = NNUEProbe::new();

        // Update this path to your actual NNUE file location
        probe.init("assets/nn-04cf2b4ed1da.nnue").unwrap();

        // Starting position
        let fen = "8/4k3/8/8/8/2Q5/8/3K4 w - - 0 1";
        let score = probe.evaluate_fen(fen).unwrap();

        println!("FEN position score: {}", score);
    }

    #[test]
    fn test_basic_evaluation() {
        let mut probe = NNUEProbe::new();
        probe.init("assets/nn-04cf2b4ed1da.nnue").unwrap();

        // Starting position pieces
        // Format: [wking, bking, piece1, piece2, ..., 0]
        let mut pieces = vec![
            CPiece::WKing as i32, // White king
            CPiece::BKing as i32, // Black king
            CPiece::WRook as i32, // White rooks
            CPiece::WRook as i32,
            CPiece::WKnight as i32, // White knights
            CPiece::WKnight as i32,
            CPiece::WBishop as i32, // White bishops
            CPiece::WBishop as i32,
            CPiece::WQueen as i32, // White queen
            CPiece::WPawn as i32,  // White pawns
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::BRook as i32, // Black rooks
            CPiece::BRook as i32,
            CPiece::BKnight as i32, // Black knights
            CPiece::BKnight as i32,
            CPiece::BBishop as i32, // Black bishops
            CPiece::BBishop as i32,
            CPiece::BQueen as i32, // Black queen
            CPiece::BPawn as i32,  // Black pawns
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            0, // End marker
        ];

        let mut squares = vec![
            4,  // e1 - White king
            60, // e8 - Black king
            0,  // a1 - White rook
            7,  // h1 - White rook
            1,  // b1 - White knight
            6,  // g1 - White knight
            2,  // c1 - White bishop
            5,  // f1 - White bishop
            3,  // d1 - White queen
            8,  // a2 - White pawns
            9,  // b2
            10, // c2
            11, // d2
            12, // e2
            13, // f2
            14, // g2
            15, // h2
            56, // a8 - Black rook
            63, // h8 - Black rook
            57, // b8 - Black knight
            62, // g8 - Black knight
            58, // c8 - Black bishop
            61, // f8 - Black bishop
            59, // d8 - Black queen
            48, // a7 - Black pawns
            49, // b7
            50, // c7
            51, // d7
            52, // e7
            53, // f7
            54, // g7
            55, // h7
            0,  // End marker
        ];

        let score = probe
            .evaluate(CColor::White, &mut pieces, &mut squares)
            .unwrap();
        println!("Basic evaluation score: {}", score);

        // Should be close to 0 for starting position (slight advantage for white)
        assert!(
            score > -100 && score < 500,
            "Score {} seems unreasonable for starting position",
            score
        );
    }

    #[test]
    fn test_incremental_evaluation() {
        let mut probe = NNUEProbe::new();
        probe.init("assets/nn-04cf2b4ed1da.nnue").unwrap();

        // Allocate NNUE data for 3 plies - use null pointers for previous positions
        // since this is the first position
        let mut nnue0 = Box::new(NNUEData::new());

        let mut nnue_ptrs = [
            &mut *nnue0 as *mut NNUEData,
            std::ptr::null_mut(), // No previous position
            std::ptr::null_mut(), // No previous position
        ];

        // Starting position
        let mut pieces = vec![
            CPiece::WKing as i32, // White king
            CPiece::BKing as i32, // Black king
            CPiece::WRook as i32, // White rooks
            CPiece::WRook as i32,
            CPiece::WKnight as i32, // White knights
            CPiece::WKnight as i32,
            CPiece::WBishop as i32, // White bishops
            CPiece::WBishop as i32,
            CPiece::WQueen as i32, // White queen
            CPiece::WPawn as i32,  // White pawns
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::WPawn as i32,
            CPiece::BRook as i32, // Black rooks
            CPiece::BRook as i32,
            CPiece::BKnight as i32, // Black knights
            CPiece::BKnight as i32,
            CPiece::BBishop as i32, // Black bishops
            CPiece::BBishop as i32,
            CPiece::BQueen as i32, // Black queen
            CPiece::BPawn as i32,  // Black pawns
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            CPiece::BPawn as i32,
            0, // End marker
        ];

        let mut squares = vec![
            4,  // e1 - White king
            60, // e8 - Black king
            0,  // a1 - White rook
            7,  // h1 - White rook
            1,  // b1 - White knight
            6,  // g1 - White knight
            2,  // c1 - White bishop
            5,  // f1 - White bishop
            3,  // d1 - White queen
            8,  // a2 - White pawns
            9,  // b2
            10, // c2
            11, // d2
            12, // e2
            13, // f2
            14, // g2
            15, // h2
            56, // a8 - Black rook
            63, // h8 - Black rook
            57, // b8 - Black knight
            62, // g8 - Black knight
            58, // c8 - Black bishop
            61, // f8 - Black bishop
            59, // d8 - Black queen
            48, // a7 - Black pawns
            49, // b7
            50, // c7
            51, // d7
            52, // e7
            53, // f7
            54, // g7
            55, // h7
            0,  // End marker
        ];

        let score = probe
            .evaluate_incremental(CColor::White, &mut pieces, &mut squares, &mut nnue_ptrs)
            .unwrap();

        println!("Incremental evaluation score: {}", score);

        // Should be close to 0 for starting position (slight advantage for white)
        assert!(
            score > -100 && score < 500,
            "Score {} seems unreasonable for starting position",
            score
        );
    }
}
