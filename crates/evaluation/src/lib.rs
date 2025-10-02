use crate::pst::{api::GamePhase, *};
use board::Position;
use types::others::Piece::{self, *};

mod pst;
/// Trait for evaluating chess positions
pub trait Evaluator {
    /// Evaluate the position from the side to move's perspective
    /// Positive score = side to move is winning
    /// Negative score = side to move is losing
    fn evaluate(&self) -> i32;
    fn evaluate_material_and_pst(&self, is_mg: bool) -> i32;
    fn evaluate_piece(
        &self,
        piece: Piece,
        material_value: i32,
        pst: &[i32; 64],
        is_opponent: bool,
    ) -> i32;
}

// Material values in centipawns (1 pawn = 100), centipawns gud cause i believe the NNUEs respond
// in centipawns too and since ima be switching later, it makes sense .. or atleast for now

static PAWN_VALUE_MG: i32 = 82;
static KNIGHT_VALUE_MG: i32 = 337;
static BISHOP_VALUE_MG: i32 = 365;
static ROOK_VALUE_MG: i32 = 477;
static QUEEN_VALUE_MG: i32 = 1025;

static PAWN_VALUE_EG: i32 = 94;
static KNIGHT_VALUE_EG: i32 = 281;
static BISHOP_VALUE_EG: i32 = 297;
static ROOK_VALUE_EG: i32 = 512;
static QUEEN_VALUE_EG: i32 = 936;

/// A Trait for implementing tapered HCE
impl Evaluator for Position {
    fn evaluate(&self) -> i32 {
        let phase = self.game_phase();
        let mg_score = self.evaluate_material_and_pst(true);
        let eg_score = self.evaluate_material_and_pst(false);
        println!(
            "Got: Middle_game_score: {}, endgame_score: {}",
            mg_score, eg_score
        );

        // Tapered eval: interpolate between middlegame and endgame
        let score = (mg_score * phase + eg_score * (24 - phase)) / 24;
        println!("Got: score: {}", score);
        score
    }

    fn evaluate_material_and_pst(&self, is_mg: bool) -> i32 {
        let mut score = 0;
        let (pawn_val, knight_val, bishop_val, rook_val, queen_val) = if is_mg {
            (
                PAWN_VALUE_MG,
                KNIGHT_VALUE_MG,
                BISHOP_VALUE_MG,
                ROOK_VALUE_MG,
                QUEEN_VALUE_MG,
            )
        } else {
            (
                PAWN_VALUE_EG,
                KNIGHT_VALUE_EG,
                BISHOP_VALUE_EG,
                ROOK_VALUE_EG,
                QUEEN_VALUE_EG,
            )
        };
        let (pawn_pst, knight_pst, bishop_pst, rook_pst, queen_pst, king_pst) = if is_mg {
            (
                &PAWN_PST_MG,
                &KNIGHT_PST_MG,
                &BISHOP_PST_MG,
                &ROOK_PST_MG,
                &QUEEN_PST_MG,
                &KING_PST_MG,
            )
        } else {
            (
                &PAWN_PST_EG,
                &KNIGHT_PST_EG,
                &BISHOP_PST_EG,
                &ROOK_PST_EG,
                &QUEEN_PST_EG,
                &KING_PST_EG,
            )
        };

        score += self.evaluate_piece(Pawn, pawn_val, pawn_pst, false);
        score += self.evaluate_piece(Knight, knight_val, knight_pst, false);
        score += self.evaluate_piece(Bishop, bishop_val, bishop_pst, false);
        score += self.evaluate_piece(Rook, rook_val, rook_pst, false);
        score += self.evaluate_piece(Queen, queen_val, queen_pst, false);
        score += self.evaluate_piece(King, 0, king_pst, false);

        // Their pieces (negative)
        score -= self.evaluate_piece(Pawn, pawn_val, pawn_pst, true);
        score -= self.evaluate_piece(Knight, knight_val, knight_pst, true);
        score -= self.evaluate_piece(Bishop, bishop_val, bishop_pst, true);
        score -= self.evaluate_piece(Rook, rook_val, rook_pst, true);
        score -= self.evaluate_piece(Queen, queen_val, queen_pst, true);
        score -= self.evaluate_piece(King, 0, king_pst, true);

        score
    }

    fn evaluate_piece(
        &self,
        piece: Piece,
        material_value: i32,
        pst: &[i32; 64],
        is_opponent: bool,
    ) -> i32 {
        let mut score = 0;
        let bb = if is_opponent {
            self.their(piece)
        } else {
            self.our(piece)
        };

        let mut pieces = bb.0;
        while pieces != 0 {
            let sq = pieces.trailing_zeros() as usize;
            pieces &= pieces - 1; // Clear the bit

            // Flip square for black pieces (PST is from white's perspective)
            let pst_sq = if is_opponent { sq ^ 56 } else { sq };

            score += material_value + pst[pst_sq];
        }

        score
    }
}
