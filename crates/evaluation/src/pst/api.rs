use board::Position;
use types::others::Piece::*;

pub trait GamePhase {
    fn game_phase(&self) -> i32;
}

impl GamePhase for Position {
    fn game_phase(&self) -> i32 {
        const KNIGHT_PHASE: i32 = 1;
        const BISHOP_PHASE: i32 = 1;
        const ROOK_PHASE: i32 = 2;
        const QUEEN_PHASE: i32 = 4;

        let mut phase = 0;

        // Count all pieces (both sides)
        phase += (self.our(Knight).0.count_ones() + self.their(Knight).0.count_ones()) as i32
            * KNIGHT_PHASE;
        phase += (self.our(Bishop).0.count_ones() + self.their(Bishop).0.count_ones()) as i32
            * BISHOP_PHASE;
        phase +=
            (self.our(Rook).0.count_ones() + self.their(Rook).0.count_ones()) as i32 * ROOK_PHASE;
        phase += (self.our(Queen).0.count_ones() + self.their(Queen).0.count_ones()) as i32
            * QUEEN_PHASE;

        phase.min(24) // Cap at 24 (opening phase)
    }
}
