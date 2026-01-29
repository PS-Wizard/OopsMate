use crate::{Piece::*, Position};

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

pub fn evaluate(pos: &Position) -> i32 {
    let mut score = 0;

    score += pos.our(Pawn).0.count_ones() as i32 * PAWN_VALUE;
    score += pos.our(Rook).0.count_ones() as i32 * ROOK_VALUE;
    score += pos.our(Knight).0.count_ones() as i32 * KNIGHT_VALUE;
    score += pos.our(Bishop).0.count_ones() as i32 * BISHOP_VALUE;
    score += pos.our(Queen).0.count_ones() as i32 * QUEEN_VALUE;

    score -= pos.their(Pawn).0.count_ones() as i32 * PAWN_VALUE;
    score -= pos.their(Rook).0.count_ones() as i32 * ROOK_VALUE;
    score -= pos.their(Knight).0.count_ones() as i32 * KNIGHT_VALUE;
    score -= pos.their(Bishop).0.count_ones() as i32 * BISHOP_VALUE;
    score -= pos.their(Queen).0.count_ones() as i32 * QUEEN_VALUE;

    score
}

#[cfg(test)]
mod test_evaluation {
    use super::*;
    use crate::Position;

    #[test]
    fn test_starting_pos() {
        let pos = Position::new();
        // Initial position, in this simple eval counts the same so should be equal
        assert_eq!(0, evaluate(&pos));
    }
}
