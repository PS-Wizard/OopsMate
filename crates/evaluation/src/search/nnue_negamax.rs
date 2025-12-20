// search/nnue_negamax.rs
use crate::evaluation::nnue_eval::NNUEEvaluator;
use board::Position;
use nnueffi::NNUEData;
use tpt::{Bound, TranspositionTable};
use types::moves::{Move, MoveCollector};

/// Trait for NNUE-based searching
pub trait NNUESearcher {
    /// Search with NNUE evaluation and incremental updates
    fn search_nnue(&mut self, depth: u8, tt: &mut TranspositionTable) -> (Option<Move>, i32);
    
    /// Negamax with NNUE
    fn negamax_nnue(
        &mut self,
        depth: u8,
        alpha: i32,
        beta: i32,
        tt: &mut TranspositionTable,
        nnue_stack: &mut Vec<Box<NNUEData>>,
        ply: usize,
    ) -> i32;
}

impl NNUESearcher for Position {
    fn search_nnue(&mut self, depth: u8, tt: &mut TranspositionTable) -> (Option<Move>, i32) {
        let mut best_move = None;
        let mut best_score = i32::MIN;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;

        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        // Initialize NNUE stack for root position
        let mut nnue_stack: Vec<Box<NNUEData>> = Vec::with_capacity((depth as usize + 1) * 2);
        nnue_stack.push(Box::new(NNUEData::new()));

        // Try TT move first
        let mut tt_move = None;
        if let Some(entry) = tt.probe(self.hash) {
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return (Some(entry.best_move), entry.score),
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => _ = (),
                }
            }
            tt_move = Some(entry.best_move);
        }

        // Search TT move first if available
        if let Some(tt_m) = tt_move {
            if collector.contains(tt_m) {
                nnue_stack.push(Box::new(NNUEData::new()));
                let undo = self.make_move(tt_m);
                let mut score = -self.negamax_nnue(depth - 1, -beta, -alpha, tt, &mut nnue_stack, 1);

                if score > 50000 && self.is_in_check() {
                    score += 1;
                }

                self.unmake_move(tt_m, undo);
                nnue_stack.pop();

                if score > best_score {
                    best_score = score;
                    best_move = Some(tt_m);
                    alpha = alpha.max(score);
                }
            }
        }

        // Search remaining moves
        for i in 0..collector.len() {
            let m = collector[i];
            if Some(m) == tt_move {
                continue;
            }

            nnue_stack.push(Box::new(NNUEData::new()));
            let undo = self.make_move(m);
            let mut score = -self.negamax_nnue(depth - 1, -beta, -alpha, tt, &mut nnue_stack, 1);

            if score > 50000 && self.is_in_check() {
                score += 1;
            }

            self.unmake_move(m, undo);
            nnue_stack.pop();

            if score > best_score {
                best_score = score;
                best_move = Some(m);
                alpha = alpha.max(score);
            }

            if alpha >= beta {
                break;
            }
        }

        let bound = if best_score <= alpha {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        if let Some(mv) = best_move {
            tt.store(self.hash, mv, best_score, depth, bound);
        }

        (best_move, best_score)
    }

    fn negamax_nnue(
        &mut self,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        tt: &mut TranspositionTable,
        nnue_stack: &mut Vec<Box<NNUEData>>,
        ply: usize,
    ) -> i32 {
        let original_alpha = alpha;

        // TT probe
        if let Some(entry) = tt.probe(self.hash) {
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::Lower => alpha = alpha.max(entry.score),
                    Bound::Upper => {
                        if entry.score <= alpha {
                            return entry.score;
                        }
                    }
                }

                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        // Terminal node - evaluate with NNUE
        if depth == 0 {
            // Build NNUE data array for incremental evaluation
            let mut nnue_ptrs = [
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ];
            
            if ply < nnue_stack.len() {
                nnue_ptrs[0] = &mut **nnue_stack.get_mut(ply).unwrap() as *mut NNUEData;
                
                if ply >= 1 {
                    nnue_ptrs[1] = &mut **nnue_stack.get_mut(ply - 1).unwrap() as *mut NNUEData;
                }
                
                if ply >= 2 {
                    nnue_ptrs[2] = &mut **nnue_stack.get_mut(ply - 2).unwrap() as *mut NNUEData;
                }
            }
            
            return self.evaluate_nnue(&mut nnue_ptrs).unwrap_or(0);
        }

        let mut collector = MoveCollector::new();
        self.generate_moves(&mut collector);

        // No legal moves - checkmate or stalemate
        if collector.len() == 0 {
            return if self.is_in_check() {
                -100000 - depth as i32
            } else {
                0
            };
        }

        let mut best_score = i32::MIN;
        let mut best_move = Move::NULL;

        let tt_move = tt.probe(self.hash).map(|e| e.best_move);

        // Try TT move first
        if let Some(tt_m) = tt_move {
            if collector.contains(tt_m) {
                nnue_stack.push(Box::new(NNUEData::new()));
                let undo = self.make_move(tt_m);
                let score = -self.negamax_nnue(depth - 1, -beta, -alpha, tt, nnue_stack, ply + 1);
                self.unmake_move(tt_m, undo);
                nnue_stack.pop();

                if score > best_score {
                    best_score = score;
                    best_move = tt_m;
                }

                alpha = alpha.max(score);
                if alpha >= beta {
                    tt.store(self.hash, best_move, best_score, depth, Bound::Lower);
                    return best_score;
                }
            }
        }

        // Search remaining moves
        for i in 0..collector.len() {
            let m = collector[i];
            if Some(m) == tt_move {
                continue;
            }

            nnue_stack.push(Box::new(NNUEData::new()));
            let undo = self.make_move(m);
            let score = -self.negamax_nnue(depth - 1, -beta, -alpha, tt, nnue_stack, ply + 1);
            self.unmake_move(m, undo);
            nnue_stack.pop();

            if score > best_score {
                best_score = score;
                best_move = m;
            }

            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }

        let bound = if best_score <= original_alpha {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        tt.store(self.hash, best_move, best_score, depth, bound);
        best_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::nnue_eval::init_nnue;
    use board::Position;

    #[test]
    fn test_nnue_search() {
        // Initialize NNUE once
        init_nnue("assets/nn-04cf2b4ed1da.nnue").unwrap();

        let mut pos = Position::from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        ).unwrap();
        
        let mut tt = TranspositionTable::new(64);
        let (best_move, score) = pos.search_nnue(5, &mut tt);

        assert!(best_move.is_some());
        println!("Best move: {:?}, Score: {}", best_move, score);
    }
}
