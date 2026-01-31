use crate::Move;

const MAX_DEPTH: usize = 128;
const KILLERS_PER_PLY: usize = 2;

/// Killer move table
/// Stores the best quiet moves that caused beta cutoffs at each ply
pub struct KillerTable {
    /// killers[ply][slot] where slot 0 is primary, slot 1 is secondary
    killers: [[Move; KILLERS_PER_PLY]; MAX_DEPTH],
}

impl KillerTable {
    #[inline(always)]
    pub fn new() -> Self {
        KillerTable {
            killers: [[Move(0); KILLERS_PER_PLY]; MAX_DEPTH],
        }
    }

    /// Store a killer move at the given ply
    /// Maintains a sliding window: new killer becomes primary, old primary becomes secondary
    #[inline(always)]
    pub fn store(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_DEPTH {
            return;
        }

        let killers = &mut self.killers[ply];

        // Don't store if it's already the primary killer
        if killers[0].0 == mv.0 {
            return;
        }

        // Shift: secondary <- primary, primary <- new
        killers[1] = killers[0];
        killers[0] = mv;
    }

    /// Check if a move is a killer at the given ply
    #[inline(always)]
    pub fn is_killer(&self, ply: usize, mv: Move) -> bool {
        if ply >= MAX_DEPTH {
            return false;
        }

        let killers = &self.killers[ply];
        killers[0].0 == mv.0 || killers[1].0 == mv.0
    }

    /// Get the primary killer move for a ply (for use as hint in move ordering)
    #[inline(always)]
    pub fn get_primary(&self, ply: usize) -> Option<Move> {
        if ply >= MAX_DEPTH {
            return None;
        }

        let mv = self.killers[ply][0];
        if mv.0 == 0 {
            None
        } else {
            Some(mv)
        }
    }

    /// Get the secondary killer move for a ply
    #[inline(always)]
    pub fn get_secondary(&self, ply: usize) -> Option<Move> {
        if ply >= MAX_DEPTH {
            return None;
        }

        let mv = self.killers[ply][1];
        if mv.0 == 0 {
            None
        } else {
            Some(mv)
        }
    }

    /// Clear all killer moves (call at start of new search)
    #[inline(always)]
    pub fn clear(&mut self) {
        for ply in 0..MAX_DEPTH {
            self.killers[ply] = [Move(0); KILLERS_PER_PLY];
        }
    }
}

impl Default for KillerTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test_history {
    use super::*;
    use crate::types::MoveType;

    #[test]
    fn test_killer_storage() {
        let mut killers = KillerTable::new();
        let mv1 = Move::new(12, 20, MoveType::Quiet);
        let mv2 = Move::new(13, 21, MoveType::Quiet);
        let mv3 = Move::new(14, 22, MoveType::Quiet);

        // Store first killer at ply 5
        killers.store(5, mv1);
        assert!(killers.is_killer(5, mv1));
        assert_eq!(killers.get_primary(5), Some(mv1));

        // Store second killer - mv1 should become secondary
        killers.store(5, mv2);
        assert!(killers.is_killer(5, mv1));
        assert!(killers.is_killer(5, mv2));
        assert_eq!(killers.get_primary(5), Some(mv2));
        assert_eq!(killers.get_secondary(5), Some(mv1));

        // Store third killer - mv1 should be evicted
        killers.store(5, mv3);
        assert!(!killers.is_killer(5, mv1));
        assert!(killers.is_killer(5, mv2));
        assert!(killers.is_killer(5, mv3));
        assert_eq!(killers.get_primary(5), Some(mv3));
        assert_eq!(killers.get_secondary(5), Some(mv2));
    }

    #[test]
    fn test_killer_no_duplicate_primary() {
        let mut killers = KillerTable::new();
        let mv = Move::new(12, 20, MoveType::Quiet);

        killers.store(5, mv);
        assert_eq!(killers.get_primary(5), Some(mv));
        assert_eq!(killers.get_secondary(5), None);

        // Storing the same move again shouldn't change anything
        killers.store(5, mv);
        assert_eq!(killers.get_primary(5), Some(mv));
        assert_eq!(killers.get_secondary(5), None);
    }

    #[test]
    fn test_killer_clear() {
        let mut killers = KillerTable::new();
        let mv = Move::new(12, 20, MoveType::Quiet);

        killers.store(5, mv);
        assert!(killers.is_killer(5, mv));

        killers.clear();
        assert!(!killers.is_killer(5, mv));
        assert_eq!(killers.get_primary(5), None);
    }

    #[test]
    fn test_killer_different_plies() {
        let mut killers = KillerTable::new();
        let mv1 = Move::new(12, 20, MoveType::Quiet);
        let mv2 = Move::new(13, 21, MoveType::Quiet);

        killers.store(5, mv1);
        killers.store(7, mv2);

        assert!(killers.is_killer(5, mv1));
        assert!(!killers.is_killer(5, mv2));
        assert!(!killers.is_killer(7, mv1));
        assert!(killers.is_killer(7, mv2));
    }
}
