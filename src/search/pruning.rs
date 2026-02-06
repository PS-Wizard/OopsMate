use crate::qsearch::qsearch;
use crate::tpt::TranspositionTable;
use crate::Move;
use crate::{Piece, Position};
use super::alphabeta::negamax;
use super::ordering::MoveHistory;
use super::SearchStats;
use std::sync::OnceLock;

// ============================================================================
//  LATE MOVE REDUCTION (LMR)
// ============================================================================

const LMR_MIN_DEPTH: u8 = 3;
const LMR_FULL_DEPTH_MOVES: usize = 2;

const MAX_DEPTH: usize = 64;
const MAX_MOVES: usize = 256;

// aggressive base formula constants
const QUIET_BASE: f32 = 0.85;
const QUIET_DIVISOR: f32 = 2.25;
const CAPTURE_BASE: f32 = 0.10;
const CAPTURE_DIVISOR: f32 = 2.85;

// PV reduction offset
const PV_REDUCTION: u8 = 1;

static LMR_TABLE: OnceLock<[[u8; MAX_MOVES]; MAX_DEPTH]> = OnceLock::new();

/// Initialize the LMR lookup table with aggressive reduction formula
fn init_lmr_table() -> [[u8; MAX_MOVES]; MAX_DEPTH] {
    let mut table = [[0u8; MAX_MOVES]; MAX_DEPTH];

    for depth in 1..MAX_DEPTH {
        for move_num in 1..MAX_MOVES {
            if depth >= LMR_MIN_DEPTH as usize && move_num >= LMR_FULL_DEPTH_MOVES {
                // More aggressive formula: lower base, smaller divisor
                let d = (depth as f32).ln();
                let m = (move_num as f32).ln();
                let reduction = QUIET_BASE + (d * m) / QUIET_DIVISOR;

                // Allow deeper reductions (depth - 1 instead of depth - 2)
                let max_reduction = (depth as i32 - 1).max(0) as f32;
                table[depth][move_num] = reduction.min(max_reduction).max(0.0) as u8;
            }
        }
    }

    table
}

/// Initialize LMR tables at startup
pub fn init_lmr() {
    LMR_TABLE.get_or_init(init_lmr_table);
}

#[inline(always)]
fn get_lmr_table() -> &'static [[u8; MAX_MOVES]; MAX_DEPTH] {
    LMR_TABLE
        .get()
        .expect("LMR table not initialized - call pruning::init_lmr() at startup")
}

/// Determine if a move should be reduced - more aggressive conditions
#[inline(always)]
pub fn should_reduce_lmr(
    depth: u8,
    move_num: usize,
    in_check: bool,
    gives_check: bool,
    mv: Move,
    thread_id: usize,
) -> bool {
    // Reduced minimum depth from 3 to allow earlier reductions if needed
    if depth < LMR_MIN_DEPTH {
        return false;
    }

    // Start reducing after move 3 instead of 4
    if move_num < LMR_FULL_DEPTH_MOVES {
        return false;
    }

    // Don't reduce check moves
    if in_check || gives_check {
        return false;
    }

    // Don't reduce promotions
    if mv.is_promotion() {
        return false;
    }
    
    // Helper threads can be more aggressive with LMR
    if thread_id > 0 && move_num > 4 {
        return true;
    }

    true
}

/// Calculate aggressive reduction amount
#[inline(always)]
pub fn calculate_lmr_reduction(depth: u8, move_num: usize, pv_node: bool, mv: Move, thread_id: usize) -> u8 {
    if depth < LMR_MIN_DEPTH || move_num < LMR_FULL_DEPTH_MOVES {
        return 0;
    }

    let table = get_lmr_table();
    let depth_idx = (depth as usize).min(MAX_DEPTH - 1);
    let move_idx = move_num.min(MAX_MOVES - 1);

    // Get base reduction from table
    let mut reduction = unsafe { *table.get_unchecked(depth_idx).get_unchecked(move_idx) };

    // More aggressive capture reductions
    if mv.is_capture() {
        let d = (depth as f32).ln();
        let m = (move_num as f32).ln();
        let capture_reduction = CAPTURE_BASE + (d * m) / CAPTURE_DIVISOR;
        let max_reduction = (depth as i32 - 1).max(0) as f32; // Allow deeper reductions
        reduction = capture_reduction.min(max_reduction).max(0.0) as u8;
    }

    // Reduce less in PV nodes (but not as much less)
    if pv_node && reduction > PV_REDUCTION {
        reduction = reduction.saturating_sub(PV_REDUCTION);
    }
    
    // DIVERSIFICATION: Vary LMR aggressiveness per thread
    if thread_id > 0 {
        let thread_mod = (thread_id % 3) as u8;
        reduction = reduction.saturating_add(thread_mod);
        
        // Even more aggressive for late moves
        if move_num > 6 {
             reduction = reduction.saturating_add(1);
        }
    }

    // More aggressive: allow reductions up to depth - 1
    reduction.min(depth.saturating_sub(1))
}

// ============================================================================
//  PROBCUT
// ============================================================================

const PROBCUT_MARGIN: i32 = 150;
const PROBCUT_MIN_DEPTH: u8 = 5;

pub fn try_probcut(
    pos: &Position,
    depth: u8,
    beta: i32,
    pv_node: bool,
    in_check: bool,
    allow_null: bool,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
    thread_id: usize,
) -> Option<i32> {
    // Check basic requirements
    if depth < PROBCUT_MIN_DEPTH || in_check || pv_node || !allow_null {
        return None;
    }

    // Don't use near mate scores
    const MATE_BOUND: i32 = 40_000;
    if beta.abs() > MATE_BOUND {
        return None;
    }

    let probcut_beta = beta + PROBCUT_MARGIN;
    let probcut_depth = depth - 5;

    let mut collector = crate::MoveCollector::new();
    pos.generate_captures(&mut collector);
    let moves = collector.as_slice();

    for &mv in moves {
        let new_pos = pos.make_move(&mv);
        
        // Search with narrow window around raised beta
        let score = -negamax(
            &new_pos,
            probcut_depth,
            -probcut_beta,
            -probcut_beta + 1,
            tt,
            history,
            stats,
            true,
            false,
            ply + 1,
            thread_id,
        );

        if score >= probcut_beta {
            return Some(beta);
        }
    }

    None
}

// ============================================================================
//  REVERSE FUTILITY PRUNING
// ============================================================================

/// Maximum depth at which reverse futility pruning is applied
const MAX_RFP_DEPTH: u8 = 7;

/// Reverse futility margins by depth
/// These are more aggressive than forward futility since we're betting our position is winning
#[rustfmt::skip]
const RFP_MARGINS: [i32; 8] = [
    0,   // depth 0 (not used)
    100, // depth 1
    200, // depth 2
    300, // depth 3
    400,
    450,
    550,
    650,
];

/// Check if we can apply reverse futility pruning to this position
#[inline(always)]
pub fn can_use_reverse_futility(depth: u8, in_check: bool, pv_node: bool, beta: i32) -> bool {
    // Don't prune in check (tactical)
    if in_check {
        return false;
    }

    // Don't prune in PV nodes (need exact scores)
    if pv_node {
        return false;
    }

    // Only apply at low depths
    if depth == 0 || depth > MAX_RFP_DEPTH {
        return false;
    }

    // Don't prune near mate scores
    const MATE_BOUND: i32 = 40_000;
    if beta.abs() > MATE_BOUND {
        return false;
    }

    true
}

/// Get the reverse futility margin for a given depth
#[inline(always)]
pub fn get_rfp_margin(depth: u8) -> i32 {
    if depth as usize >= RFP_MARGINS.len() {
        return RFP_MARGINS[RFP_MARGINS.len() - 1];
    }
    RFP_MARGINS[depth as usize]
}

/// Check if we should prune this node with reverse futility
/// Returns true if static_eval - margin >= beta (our position is too good)
#[inline(always)]
pub fn should_rfp_prune(static_eval: i32, beta: i32, margin: i32) -> bool {
    static_eval - margin >= beta
}

// ============================================================================
//  NULL MOVE PRUNING
// ============================================================================

/// Attempts null move pruning - returns Some(score) if pruning succeeds, None otherwise
#[inline(always)]
pub fn try_null_move_pruning(
    pos: &Position,
    depth: u8,
    beta: i32,
    allow_null: bool,
    in_check: bool,
    tt: &TranspositionTable,
    history: &mut MoveHistory,
    stats: &mut SearchStats,
    ply: usize,
    thread_id: usize,
) -> Option<i32> {
    // Don't do null move if:
    // - Not allowed (to prevent double null moves)
    // - In check (illegal to pass when in check)
    // - Not deep enough
    if !allow_null || in_check || depth < 3 {
        return None;
    }

    // Don't do null move in endgame positions without pieces
    // (zugzwang risk is too high)
    let has_pieces = (pos.our(Piece::Knight).0
        | pos.our(Piece::Bishop).0
        | pos.our(Piece::Rook).0
        | pos.our(Piece::Queen).0)
        != 0;

    if !has_pieces {
        return None;
    }

    // Create null move position
    let null_pos = make_null_move(pos);

    // Calculate reduction depth
    // DIVERSIFICATION: Alternate reduction depth
    let base = if depth >= 7 { 4 } else { 3 };
    let thread_adj = if thread_id > 0 { (thread_id & 1) as i32 } else { 0 };
    let reduction = base + thread_adj;
    
    let null_depth = depth.saturating_sub(1 + reduction as u8);

    // Search with null window
    let null_score = -negamax(
        &null_pos,
        null_depth,
        -beta,
        -beta + 1,
        tt,
        history,
        stats,
        false,
        false,
        ply + 1,
        thread_id,
    );

    // If null move fails high, we can prune this node
    if null_score >= beta {
        Some(beta)
    } else {
        None
    }
}

/// Creates a position after making a null move (passing the turn)
#[inline(always)]
fn make_null_move(pos: &Position) -> Position {
    let mut null_pos = *pos;
    null_pos.side_to_move = null_pos.side_to_move.flip();
    null_pos.hash ^= crate::zobrist::SIDE_KEY;
    null_pos.en_passant = None;
    null_pos
}

// ============================================================================
//  RAZORING
// ============================================================================

/// Razoring margins by depth
#[rustfmt::skip]
const RAZOR_MARGINS: [i32; 4] = [
    0,   // depth 0 (not used)
    300, // depth 1
    400, // depth 2
    500, // depth 3
];

/// Try razoring - return Some(score) if we can prune, None otherwise
#[inline(always)]
pub fn try_razoring(
    pos: &Position,
    depth: u8,
    alpha: i32,
    in_check: bool,
    pv_node: bool,
    static_eval: i32,
    stats: &mut SearchStats,
) -> Option<i32> {
    // Only razor at very low depths
    if depth == 0 || depth > 3 {
        return None;
    }

    // Don't razor in check or PV nodes
    if in_check || pv_node {
        return None;
    }

    // Don't razor near mate scores
    const MATE_BOUND: i32 = 40_000;
    if alpha.abs() > MATE_BOUND {
        return None;
    }

    let margin = RAZOR_MARGINS[depth as usize];

    // If static eval is way below alpha, even with margin
    if static_eval + margin < alpha {
        // Verify with qsearch
        let razor_score = qsearch(pos, alpha - margin, alpha - margin + 1, stats, 0);

        if razor_score < alpha - margin {
            return Some(razor_score);
        }
    }

    None
}

// ============================================================================
//  FUTILITY PRUNING
// ============================================================================

/// Maximum depth at which futility pruning is applied
const MAX_FUTILITY_DEPTH: u8 = 7;

#[rustfmt::skip]
const FUTILITY_MARGINS: [i32; 8] = [
    0,   // depth 0 (not used, handled by qsearch)
    90,  // depth 1
    180, // depth 2
    270,
    360,
    450,
    540,
    630,
];

/// Check if we can apply futility pruning to this position/search state
#[inline(always)]
pub fn can_use_futility_pruning(
    depth: u8,
    in_check: bool,
    pv_node: bool,
    alpha: i32,
    beta: i32,
) -> bool {
    // Don't prune in check (tactical)
    if in_check {
        return false;
    }

    // Don't prune in PV nodes (need exact scores)
    if pv_node {
        return false;
    }

    // Only apply at low depths
    if depth == 0 || depth > MAX_FUTILITY_DEPTH {
        return false;
    }

    // Don't prune near mate scores (margins don't apply)
    const MATE_BOUND: i32 = 40_000;
    if alpha.abs() > MATE_BOUND || beta.abs() > MATE_BOUND {
        return false;
    }

    true
}

/// Get the futility margin for a given depth
#[inline(always)]
pub fn get_futility_margin(depth: u8) -> i32 {
    if depth as usize >= FUTILITY_MARGINS.len() {
        return FUTILITY_MARGINS[FUTILITY_MARGINS.len() - 1];
    }
    FUTILITY_MARGINS[depth as usize]
}

/// Check if a move should be pruned based on futility
/// Returns true if the move can be safely skipped
#[inline(always)]
pub fn should_prune_futility(
    mv: Move,
    gives_check: bool,
    static_eval: i32,
    alpha: i32,
    margin: i32,
) -> bool {
    // Never prune tactical moves
    if mv.is_capture() {
        return false;
    }

    // This can change the val like ~900 centipawns from a queen promo
    if mv.is_promotion() {
        return false;
    }

    if gives_check {
        return false;
    }

    // Prune if static_eval + margin <= alpha
    static_eval + margin <= alpha
}