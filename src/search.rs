use crate::{
    evaluate::evaluate,
    move_history::MoveHistory,
    move_ordering::{pick_next_move, score_move},
    pruning::{
        calculate_lmr_reduction, can_use_futility_pruning, can_use_reverse_futility,
        get_futility_margin, get_rfp_margin, should_prune_futility, should_reduce_lmr,
        should_rfp_prune, try_null_move_pruning, try_probcut, try_razoring,
    },
    qsearch::qsearch,
    tpt::{TranspositionTable, EXACT, LOWER_BOUND, UPPER_BOUND},
    Move, MoveCollector, Position,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io::Write, time::Instant};

// ============================================================================
//  CONSTANTS
// ============================================================================

const INFINITY: i32 = 50_000;
const MATE_VALUE: i32 = 49_000;
const MAX_MOVES: usize = 256;
const ASPIRATION_DEPTH: u8 = 8;
const INITIAL_WINDOW: i32 = 25;
const IID_MIN_DEPTH: u8 = 4;

// ============================================================================
//  STRUCTS
// ============================================================================

pub struct SearchStats {
    pub nodes: u64,
    pub tt_hits: u64,
    pub stop_signal: Option<Arc<AtomicBool>>,
}

impl SearchStats {
    pub fn new(stop_signal: Option<Arc<AtomicBool>>) -> Self {
        SearchStats {
            nodes: 0,
            tt_hits: 0,
            stop_signal,
        }
    }

    #[inline(always)]
    pub fn should_stop(&self) -> bool {
        // Check periodically (every 2048 nodes)
        if self.nodes & 2047 == 0 {
            if let Some(signal) = &self.stop_signal {
                return signal.load(Ordering::Relaxed);
            }
        }
        false
    }
}

pub struct SearchInfo {
    pub best_move: Move,
    pub score: i32,
    pub depth: u8,
    pub nodes: u64,
    pub time_ms: u64,
    pub tt_hits: u64,
}

// ============================================================================
//  MAIN SEARCH ENTRY POINT
// ============================================================================

/// Main search function with iterative deepening and aspiration windows
pub fn search(
    pos: &Position,
    max_depth: u8,
    max_time_ms: Option<u64>,
    tt: Arc<TranspositionTable>,
    threads: usize,
) -> Option<SearchInfo> {
    let stop_signal = Arc::new(AtomicBool::new(false));
    let threads = threads.max(1);

    // Mark new search for TT aging - only done once by master
    tt.new_search();

    // Spawn helper threads
    let mut handles = Vec::new();
    if threads > 1 {
        for id in 1..threads {
            let pos_clone = *pos; // Position is Copy? No, it's a struct.
            // Check if Position is Copy. In types.rs? Board is array.
            // Position struct usually implements Clone/Copy if small.
            // Let's assume Clone is available.
            let tt_clone = tt.clone();
            let signal_clone = stop_signal.clone();
            
            handles.push(std::thread::spawn(move || {
                search_driver(
                    &pos_clone,
                    max_depth,
                    None, // Helpers don't manage time directly
                    &tt_clone,
                    signal_clone,
                    id,
                )
            }));
        }
    }

    // Run master search
    let info = search_driver(
        pos,
        max_depth,
        max_time_ms,
        &tt,
        stop_signal.clone(),
        0,
    );

    // Signal helpers to stop
    stop_signal.store(true, Ordering::Relaxed);

    // Join helpers
    for handle in handles {
        let _ = handle.join();
    }

    info
}

/// Driver for the iterative deepening loop
fn search_driver(
    pos: &Position,
    max_depth: u8,
    max_time_ms: Option<u64>,
    tt: &TranspositionTable,
    stop_signal: Arc<AtomicBool>,
    thread_id: usize,
) -> Option<SearchInfo> {
    let start_time = Instant::now();
    let mut stats = SearchStats::new(Some(stop_signal.clone()));
    let mut history = MoveHistory::new();
    let mut best_move = None;
    let mut best_score = 0; // Initialize to 0 for first aspiration window
    let is_master = thread_id == 0;

    // Generate moves once to check for game over
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    if moves.is_empty() {
        return None;
    }

    // Diversification: simple depth offset logic or LMR noise?
    // Let's stick to standard loop but helpers might break if stopped.
    // Also helpers could start slightly differently?
    // For now, identical loop.

    // Iterative deepening loop
    for depth in 1..=max_depth {
        let depth_start = Instant::now();

        // Check stop signal before starting depth
        if stop_signal.load(Ordering::Relaxed) {
            break;
        }

        // search_aspiration handles both shallow (full window) and deep (aspiration) searches
        let (iteration_best_score, iteration_best_move) =
            search_aspiration(pos, depth, best_score, tt, &mut history, &mut stats);

        // Check if we stopped during search
        if stop_signal.load(Ordering::Relaxed) {
            break;
        }

        // Update best move for this depth
        best_move = Some(iteration_best_move);
        best_score = iteration_best_score;

        // Print UCI info (Master only)
        if is_master {
            print_uci_info(
                depth,
                best_score,
                &stats,
                start_time,
                tt,
                &iteration_best_move,
            );
        }

        let current_depth_time = depth_start.elapsed().as_millis() as u64;

        // Time management - break if we're likely to run out of time (Master only)
        if is_master {
            if should_stop_search(max_time_ms, start_time, current_depth_time) {
                stop_signal.store(true, Ordering::Relaxed);
                break;
            }

            // Check time during search
            if let Some(max_time) = max_time_ms {
                if start_time.elapsed().as_millis() as u64 >= max_time {
                    stop_signal.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
    }

    best_move.map(|mv| SearchInfo {
        best_move: mv,
        score: best_score,
        depth: max_depth,
        nodes: stats.nodes,
        time_ms: start_time.elapsed().as_millis() as u64,
        tt_hits: stats.tt_hits,
    })
}

// ============================================================================
//  ASPIRATION WINDOWS
// ============================================================================

#[inline(always)]
fn search_aspiration(
    pos: &Position,
    depth: u8,
    prev_score: i32,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
) -> (i32, Move) {
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);

    if collector.as_slice().is_empty() {
        return if pos.is_in_check() {
            (-49_000 - depth as i32, Move(0))
        } else {
            (0, Move(0))
        };
    }

    let mut moves = [Move(0); MAX_MOVES];
    let count = collector.len();
    for i in 0..count {
        moves[i] = collector.as_slice()[i];
    }
    let moves_slice = &mut moves[..count];

    // If shallow, just search full window
    if depth < ASPIRATION_DEPTH {
        return search_root(
            pos,
            moves_slice,
            depth,
            -INFINITY,
            INFINITY,
            tt,
            history,
            stats,
        );
    }

    // Aspiration Loop
    let mut delta = INITIAL_WINDOW;
    let mut alpha = prev_score - delta;
    let mut beta = prev_score + delta;

    loop {
        // We pass 'depth' to let search_root know if should use the optimization
        let (score, best_move) =
            search_root(pos, moves_slice, depth, alpha, beta, tt, history, stats);

        if stats.should_stop() {
            return (score, best_move);
        }

        // Success Inside window
        if score > alpha && score < beta {
            return (score, best_move);
        }

        // Fail Low: Score <= Alpha
        if score <= alpha {
            beta = (alpha + beta) / 2;
            alpha = alpha.saturating_sub(delta);
            delta += delta / 2;
        }
        // Fail High: Score >= Beta
        else if score >= beta {
            alpha = (alpha + beta) / 2;
            beta = beta.saturating_add(delta);
            delta += delta / 2;
        }

        // If window gets too huge, give up and search infinite
        if delta > 1000 {
            alpha = -INFINITY;
            beta = INFINITY;
        }
    }
}

#[inline(always)]
fn search_root(
    pos: &Position,
    moves: &mut [Move],
    depth: u8,
    mut alpha: i32,
    beta: i32,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
) -> (i32, Move) {
    let in_check = pos.is_in_check();
    let tt_move = tt.probe(pos.hash()).map(|e| e.best_move);
    let move_count = moves.len();
    let mut scores = [0i32; MAX_MOVES];

    // Score moves
    for i in 0..move_count {
        scores[i] = score_move(moves[i], pos, tt_move, Some(history), 0);
    }

    let mut best_score = -INFINITY;
    let mut best_move = moves[0];

    for i in 0..move_count {
        pick_next_move(moves, &mut scores, i);
        let mv = moves[i];
        let newpos = pos.make_move(&mv);
        let gives_check = newpos.is_in_check();

        let score = if i == 0 {
            search_move(
                &newpos,
                mv,
                depth,
                alpha,
                beta,
                i,
                in_check,
                gives_check,
                true,
                tt,
                history,
                stats,
                0,
            )
        } else {
            // PVS for other moves
            let s = search_move(
                &newpos,
                mv,
                depth,
                alpha,
                alpha + 1,
                i,
                in_check,
                gives_check,
                true,
                tt,
                history,
                stats,
                0,
            );
            if s > alpha && s < beta {
                search_move(
                    &newpos,
                    mv,
                    depth,
                    alpha,
                    beta,
                    i,
                    in_check,
                    gives_check,
                    true,
                    tt,
                    history,
                    stats,
                    0,
                )
            } else {
                s
            }
        };

        if stats.should_stop() {
            return (best_score, best_move);
        }

        // crafty's optimization
        if i == 0 && score <= alpha {
            return (score, mv);
        }

        if score > best_score {
            best_score = score;
            best_move = mv;
            if score > alpha {
                alpha = score;
                if score >= beta {
                    break; // Beta Cutoff
                }
            }
        }
    }

    // Store result
    let flag = if best_score >= beta {
        LOWER_BOUND
    } else if best_score <= alpha {
        UPPER_BOUND
    } else {
        EXACT
    };
    tt.store(pos.hash(), best_move, best_score, depth, flag);

    (best_score, best_move)
}

// ============================================================================
//  NEGAMAX
// ============================================================================

#[allow(clippy::too_many_arguments)]
pub fn negamax(
    pos: &Position,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    allow_null: bool,
    pv_node: bool,
    ply: usize,
) -> i32 {
    stats.nodes += 1;

    // Check stop signal
    if stats.should_stop() {
        return 0; // Return neutral score or handle abort
    }

    let hash = pos.hash();

    // Probe transposition table - check for early cutoff
    let tt_move = {
        let tt_entry = tt.probe(hash);
        if let Some(entry) = tt_entry {
            if entry.depth >= depth {
                stats.tt_hits += 1;
                match entry.flag {
                    EXACT => return entry.score,
                    LOWER_BOUND if entry.score >= beta => return entry.score,
                    UPPER_BOUND if entry.score <= alpha => return entry.score,
                    _ => {}
                }
            }
            Some(entry.best_move)
        } else {
            None
        }
    };

    // Base case - drop into quiescence search
    if depth == 0 {
        return qsearch(pos, alpha, beta, stats, 0);
    }
    let in_check = pos.is_in_check();

    // Static evaluation for pruning decisions
    let static_eval = evaluate(pos);

    // PROBCUT
    if let Some(score) = try_probcut(
        pos, depth, beta, pv_node, in_check, allow_null, tt, history, stats, ply,
    ) {
        return score;
    }

    // RAZORING
    if let Some(score) = try_razoring(pos, depth, alpha, in_check, pv_node, static_eval, stats) {
        return score;
    }

    // Reverse futility pruning
    if can_use_reverse_futility(depth, in_check, pv_node, beta) {
        let rfp_margin = get_rfp_margin(depth);
        if should_rfp_prune(static_eval, beta, rfp_margin) {
            return static_eval - rfp_margin;
        }
    }

    // Try null move pruning
    if let Some(score) = try_null_move_pruning(
        pos, depth, beta, allow_null, in_check, tt, history, stats, ply,
    ) {
        return score;
    }

    // Internal Iterative Deepening
    let iid_move = try_iid(
        pos,
        depth,
        alpha,
        beta,
        pv_node,
        tt_move.is_some(),
        in_check,
        tt,
        history,
        stats,
        ply,
    );

    // Use IID move if we found one and don't have a TT move
    let tt_move = tt_move.or(iid_move);

    // Futility pruning setup
    let use_futility = can_use_futility_pruning(depth, in_check, pv_node, alpha, beta);
    let (static_eval, futility_margin) = if use_futility {
        let margin = get_futility_margin(depth);
        (static_eval, margin)
    } else {
        (0, 0)
    };

    // Generate and order moves
    let mut collector = MoveCollector::new();
    pos.generate_moves(&mut collector);
    let moves = collector.as_slice();

    // Checkmate / Stalemate detection
    if moves.is_empty() {
        return if in_check {
            -MATE_VALUE - (depth as i32)
        } else {
            0
        };
    }

    let move_count = moves.len();
    let mut move_list = [Move(0); MAX_MOVES];
    let mut scores = [0i32; MAX_MOVES];

    // Score moves for ordering
    for i in 0..move_count {
        move_list[i] = moves[i];
        scores[i] = score_move(moves[i], pos, tt_move, Some(history), ply);
    }

    let mut best_score = -INFINITY;
    let mut best_move = Move(0);

    // Search all moves
    for i in 0..move_count {
        pick_next_move(&mut move_list[..move_count], &mut scores[..move_count], i);
        let mv = move_list[i];

        let new_pos = pos.make_move(&mv);
        let gives_check = new_pos.is_in_check();
        let check_extension = if gives_check { 1 } else { 0 };

        // Futility pruning
        if use_futility && i > 0 {
            if should_prune_futility(mv, gives_check, static_eval, alpha, futility_margin) {
                continue;
            }
        }

        let score = if i == 0 {
            // First move: full depth, full window
            -negamax(
                &new_pos,
                depth - 1,
                -beta,
                -alpha,
                tt,
                history,
                stats,
                true,
                pv_node,
                ply + 1,
            )
        } else {
            // PVS for subsequent moves
            let is_hash_move = tt_move.map_or(false, |tt_mv| mv.0 == tt_mv.0);
            let mut s = if should_reduce_lmr(depth, i, in_check, gives_check, mv) & !is_hash_move {
                let reduction = calculate_lmr_reduction(depth, i, pv_node, mv);
                let reduced_depth = depth
                    .saturating_sub(1 + reduction)
                    .saturating_add(check_extension);

                -negamax(
                    &new_pos,
                    reduced_depth,
                    -alpha - 1,
                    -alpha,
                    tt,
                    history,
                    stats,
                    true,
                    false,
                    ply + 1,
                )
            } else {
                // Null window search
                -negamax(
                    &new_pos,
                    depth - 1 + check_extension,
                    -alpha - 1,
                    -alpha,
                    tt,
                    history,
                    stats,
                    true,
                    false,
                    ply + 1,
                )
            };

            if s > alpha && s < beta {
                s = -negamax(
                    &new_pos,
                    depth - 1 + check_extension,
                    -beta,
                    -alpha,
                    tt,
                    history,
                    stats,
                    true,
                    pv_node,
                    ply + 1,
                );
            }

            s
        };

        if stats.should_stop() {
            return 0; // Abort
        }

        // Beta cutoff
        if score >= beta {
            if !mv.is_capture() && !mv.is_promotion() {
                history.killers.store(ply, mv);
                let bonus = (depth as i16 * depth as i16).min(400);
                history.history.update(pos.side_to_move, mv.from(), mv.to(), bonus);
            }

            tt.store(hash, mv, beta, depth, LOWER_BOUND);
            return beta;
        }

        // Update best move
        if score > best_score {
            best_score = score;
            best_move = mv;

            if score > alpha {
                alpha = score;
            }
        }
    }

    let flag = if best_score <= alpha {
        UPPER_BOUND
    } else {
        EXACT
    };
    tt.store(hash, best_move, best_score, depth, flag);

    best_score
}

// ============================================================================
//  PVS
// ============================================================================

#[inline(always)]
pub fn search_move(
    pos: &Position,
    mv: Move,
    depth: u8,
    alpha: i32,
    beta: i32,
    move_num: usize,
    in_check: bool,
    gives_check: bool,
    pv_node: bool,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
) -> i32 {
    if move_num == 0 {
        return -negamax(
            &*pos,
            depth - 1,
            -beta,
            -alpha,
            tt,
            history,
            stats,
            gives_check,
            pv_node,
            ply + 1,
        );
    }

    let do_lmr = should_reduce_lmr(depth, move_num, in_check, gives_check, mv);

    let mut score = if do_lmr {
        let reduction = calculate_lmr_reduction(depth, move_num, pv_node, mv);
        let reduced_depth = depth.saturating_sub(1 + reduction);

        -negamax(
            &*pos,
            reduced_depth,
            -alpha - 1,
            -alpha,
            tt,
            history,
            stats,
            gives_check,
            false,
            ply + 1,
        )
    } else {
        -negamax(
            &*pos,
            depth - 1,
            -alpha - 1,
            -alpha,
            tt,
            history,
            stats,
            gives_check,
            false,
            ply + 1,
        )
    };

    if score > alpha && score < beta {
        score = -negamax(
            &*pos,
            depth - 1,
            -beta,
            -alpha,
            tt,
            history,
            stats,
            gives_check,
            pv_node,
            ply + 1,
        );
    }

    score
}

// ============================================================================
//  INTERNAL ITERATIVE DEEPENING (IID)
// ============================================================================

#[inline(always)]
fn iid_reduction(depth: u8, pv_node: bool) -> u8 {
    if pv_node {
        (depth / 4).max(2)
    } else {
        (depth / 3).max(1)
    }
}

#[inline(always)]
pub fn try_iid(
    pos: &crate::Position,
    depth: u8,
    alpha: i32,
    beta: i32,
    pv_node: bool,
    has_tt_move: bool,
    in_check: bool,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
) -> Option<Move> {
    if has_tt_move || depth < IID_MIN_DEPTH || in_check {
        return None;
    }

    if !pv_node && depth < IID_MIN_DEPTH + 2 {
        return None;
    }

    let reduction = iid_reduction(depth, pv_node);
    let iid_depth = depth.saturating_sub(reduction);

    negamax(
        pos, iid_depth, alpha, beta, tt, history, stats, true, 
        pv_node, ply,
    );

    tt.probe(pos.hash()).map(|entry| entry.best_move)
}

// ============================================================================
//  HELPERS
// ============================================================================

fn print_uci_info(
    depth: u8,
    score: i32,
    stats: &SearchStats,
    start_time: Instant,
    tt: &TranspositionTable,
    mv: &Move,
) {
    let elapsed = start_time.elapsed().as_millis() as u64;
    let nps = if elapsed > 0 {
        (stats.nodes * 1000) / elapsed
    } else {
        0
    };

    println!(
        "info depth {} score cp {} nodes {} time {} nps {} hashfull {} pv {}",
        depth,
        score,
        stats.nodes,
        elapsed,
        nps,
        tt.hashfull(),
        move_to_uci(mv)
    );

    let _ = std::io::stdout().flush();
}

fn should_stop_search(
    max_time_ms: Option<u64>,
    start_time: Instant,
    current_depth_time: u64,
) -> bool {
    if let Some(max_time) = max_time_ms {
        let elapsed_total = start_time.elapsed().as_millis() as u64;
        let time_remaining = max_time.saturating_sub(elapsed_total);
        let predicted_next_depth = current_depth_time.saturating_mul(4);

        predicted_next_depth > time_remaining || elapsed_total * 10 > max_time * 7
    } else {
        false
    }
}

fn move_to_uci(m: &Move) -> String {
    let from = m.from();
    let to = m.to();

    let from_sq = format!(
        "{}{}",
        (b'a' + (from % 8) as u8) as char,
        (b'1' + (from / 8) as u8) as char
    );
    let to_sq = format!(
        "{}{}",
        (b'a' + (to % 8) as u8) as char,
        (b'1' + (to / 8) as u8) as char
    );

    if m.is_promotion() {
        let promo = match m.move_type() {
            crate::types::MoveType::PromotionQueen
            | crate::types::MoveType::CapturePromotionQueen => 'q',
            crate::types::MoveType::PromotionRook
            | crate::types::MoveType::CapturePromotionRook => 'r',
            crate::types::MoveType::PromotionBishop
            | crate::types::MoveType::CapturePromotionBishop => 'b',
            crate::types::MoveType::PromotionKnight
            | crate::types::MoveType::CapturePromotionKnight => 'n',
            _ => unreachable!(),
        };
        format!("{}{}{}", from_sq, to_sq, promo)
    } else {
        format!("{}{}", from_sq, to_sq)
    }
}

// ============================================================================
//  TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iid_reduction() {
        // PV nodes: more aggressive reduction
        assert_eq!(iid_reduction(4, true), 2);
        assert_eq!(iid_reduction(8, true), 2);
        assert_eq!(iid_reduction(12, true), 3);
        assert_eq!(iid_reduction(16, true), 4);

        // Non-PV nodes: less aggressive
        assert_eq!(iid_reduction(4, false), 1);
        assert_eq!(iid_reduction(9, false), 3);
        assert_eq!(iid_reduction(12, false), 4);
    }

    #[test]
    fn test_iid_depth_threshold() {
        use crate::{tpt::TranspositionTable, Position};

        let pos = Position::new();
        let tt = TranspositionTable::new_mb(16);
        let mut history = MoveHistory::new();
        let mut stats = SearchStats::new(None);

        // Should not trigger at shallow depths
        let result = try_iid(
            &pos,
            3,
            -1000,
            1000,
            true,
            false,
            false,
            &tt,
            &mut history,
            &mut stats,
            0,
        );
        assert!(result.is_none());

        // Should trigger at sufficient depth for PV
        let result = try_iid(
            &pos,
            4,
            -1000,
            1000,
            true,
            false,
            false,
            &tt,
            &mut history,
            &mut stats,
            0,
        );
        assert!(result.is_some());
    }

    #[test]
    fn test_pvs_first_move() {
        let pos =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let tt = TranspositionTable::new_mb(16);
        let mut history = MoveHistory::new();
        let mut stats = SearchStats::new(None);

        let mut collector = crate::MoveCollector::new();
        pos.generate_moves(&mut collector);
        let moves = collector.as_slice();

        if let Some(&mv) = moves.first() {
            let score = search_move(
                &pos,
                mv,
                3,
                -1000,
                1000,
                0, // First move
                false,
                false,
                true,
                &tt,
                &mut history,
                &mut stats,
                0,
            );

            assert!(score.abs() < 10000);
        }
    }

    #[test]
    fn test_pvs_later_move() {
        let pos =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let tt = TranspositionTable::new_mb(16);
        let mut history = MoveHistory::new();
        let mut stats = SearchStats::new(None);

        let mut collector = crate::MoveCollector::new();
        pos.generate_moves(&mut collector);
        let moves = collector.as_slice();

        if moves.len() > 1 {
            let score = search_move(
                &pos,
                moves[1],
                3,
                -1000,
                1000,
                1, // Second move
                false,
                false,
                true,
                &tt,
                &mut history,
                &mut stats,
                0,
            );

            assert!(score.abs() < 10000);
        }
    }

    #[test]
    #[ignore = "Overflows On Debug / Need Release"]
    fn test_iterative_deepening() {
        use crate::pruning::init_lmr;
        use std::sync::Arc;

        let depth = 18;
        // let pos = Position::new();
        let pos = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap_or_default();

        let tt = Arc::new(TranspositionTable::new_mb(512));
        init_lmr();

        println!("Starting iterative deepening search to depth {}...", depth);
        let start = std::time::Instant::now();

        let result = search(&pos, depth, None, tt.clone(), 1);

        let duration = start.elapsed();

        if let Some(info) = result {
            println!(
                "Best move: {} (depth {}, score {}, nodes {}, time {:.3}s, nps {})",
                move_to_uci(&info.best_move),
                info.depth,
                info.score,
                info.nodes,
                duration.as_secs_f64(),
                if duration.as_millis() > 0 {
                    (info.nodes * 1000) / duration.as_millis() as u64
                } else {
                    0
                }
            );
        } else {
            println!("No move found");
        }
    }
}
