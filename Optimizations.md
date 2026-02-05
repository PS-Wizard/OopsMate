# Chess Engine Optimization Analysis

## Executive Summary
OopsMate is a solid engine with a strong technical foundation (PEXT bitboards, efficient types). However, it lacks several standard chess programming techniques that significantly impact playing strength. The most critical missing components are the **History Heuristic** for move ordering and **Evaluation Terms** (King Safety, Mobility). Implementing these, along with tuning existing pruning parameters, should yield substantial Elo gains.

## High Impact Optimizations

### 1. History Heuristic
**Category**: Algorithm
**Estimated Impact**: High (+50-100 Elo)
**Difficulty**: Medium

**Current State**:
In `move_ordering.rs`, the code explicitly notes `// TODO: History Heuristic` and returns `0` for quiet moves that aren't killers. This means most quiet moves are effectively random, causing the search to waste time on bad branches.

**Proposed Change**:
1.  Create a `HistoryTable` (likely `[[i32; 64]; 64]` for [from][to] or `[from][to][color]`).
2.  Update it in `search.rs` (or `negamax`):
    *   **Bonus**: When a quiet move causes a beta cutoff (fails high), increment its score (e.g., `depth * depth`).
    *   **Malus**: (Optional) Decrement scores for moves that were searched but failed low.
    *   **Aging**: Periodically divide scores by 2 to prefer recent history.
3.  Use this score in `move_ordering.rs` to sort non-killer quiet moves.

**Rationale**:
Good moves tend to remain good across different positions in the same game tree. Sorting them first drastically improves alpha-beta pruning efficiency (better move ordering = fewer nodes searched).

### 2. Comprehensive Evaluation Terms
**Category**: Algorithm
**Estimated Impact**: High (+50-100 Elo)
**Difficulty**: Medium

**Current State**:
`evaluate.rs` only considers Material and Piece-Square Tables (PST).

**Proposed Change**:
Add the following terms:
*   **Mobility**: Count safe squares available to knights, bishops, rooks, and queens.
*   **King Safety**: Penalty for open files near king, bonus for pawn shield, penalty for enemy piece attacks near king ring.
*   **Pawn Structure**: Penalties for doubled, isolated, and backward pawns. Bonus for passed pawns (scaled by rank).

**Rationale**:
The current evaluation is "blind" to positional factors. It will accept bad structures or unsafe king positions if the material is equal.

### 3. Move Ordering Efficiency (Deferred Scoring)
**Category**: Performance
**Estimated Impact**: Medium
**Difficulty**: Easy

**Current State**:
`search_root` and `negamax` call `score_move` for *every* move in the list before picking the first one. `score_move` calls `pos.see()` for every capture.

**Proposed Change**:
Implement "staged" move generation or deferred scoring:
1.  Score and sort only Hash Move, Captures, and Killers.
2.  Pick and search these.
3.  Only if no cutoff occurs, score the remaining Quiet moves (using History Heuristic).

**Rationale**:
In many nodes, a beta cutoff occurs on the first or second move (Hash move or good capture). Scoring the entire list (especially running SEE on all captures) is wasted CPU cycles.

---

## Parameter Tuning Recommendations

The following parameters are hardcoded and should be exposed for tuning (e.g., via SPSA).

| Parameter | Location | Current Value | Suggested Range | Impact |
|-----------|----------|---------------|-----------------|--------|
| **LMR Base** | `pruning.rs` | 0.85 | 0.5 - 1.2 | High |
| **LMR Divisor** | `pruning.rs` | 2.25 | 1.5 - 3.5 | High |
| **Null Move R** | `pruning.rs` | 3 if d>=7 else 2 | `3` if d > 5 | Medium |
| **RFP Margin** | `pruning.rs` | Array `[0, 150, 250...]` | Linear: `50 * depth` | Medium |
| **Futility Margin** | `pruning.rs` | Array `[0, 100, 200...]` | Linear: `100 * depth` | Medium |
| **IID Depth** | `search.rs` | 4 | 3 - 6 | Low |

---

## Unsafe Code Opportunities

The codebase already utilizes `unsafe` effectively (PEXT, unchecked indexing). Further opportunities are limited but exist:

1.  **Move Ordering Swaps**: `pick_next_move` uses slice indexing.
    *   *Safe*: `moves.swap(index, best_idx)` is safe.
    *   *Unsafe*: `ptr::swap` could be used if bounds are proven, but standard library `swap` is usually optimized enough.
2.  **MoveCollector**: The `MoveCollector` uses `MaybeUninit` correctly. No changes needed.

---

## Quick Wins

1.  **Tune LMR Conditions**: The condition `depth < LMR_MIN_DEPTH (3)` is conservative. Modern engines often LMR at depth 2 or even 1 for very bad moves.
2.  **Increase Killers**: `KILLERS_PER_PLY` is 2. Increasing to 3 or 4 might help if move ordering is weak (though History Heuristic is the real fix).
3.  **String Allocations**: `print_uci_info` allocates strings every second. Optimize `move_to_uci` to write to a pre-allocated buffer or use `std::fmt::Write`.

## Future Considerations

1.  **Texel Tuning**: Implement an automated tuner to optimize evaluation weights (Material/PST) against a database of positions.
2.  **Lazy SMP**: Rust makes parallel search relatively easy. Sharing the Transposition Table between threads can add significant nps.
3.  **NNUE**: The ultimate evaluation step. Replacing hand-crafted evaluation with a small neural network (Efficiently Updatable Neural Network) is the standard for modern engines (Stockfish, etc.).

## Strikes Crate Optimizations

The `strikes` crate, which handles magic bitboards/PEXT attack lookups, has a significant structural inefficiency that affects every single slider move generation.

### 1. Flatten Attack Tables (Implemented)
**Category**: Performance
**Estimated Impact**: High (Move Generation Speed)
**Difficulty**: Medium

**Current State**:
The attack tables are defined as `Vec<Vec<u64>>`:
```rust
pub static ROOK_ATTACKS: LazyLock<Vec<Vec<u64>>> = ...
```
This creates a jagged array with double indirection:
1.  Access `ROOK_ATTACKS[sq]` -> Follows pointer to heap-allocated inner `Vec`.
2.  Access `inner_vec[pext_index]` -> Accesses the actual attack bitboard.
This causes two cache misses on cold lookups and scatters the table across memory.

**Proposed Change**:
Flatten the tables into a single contiguous `Vec<u64>` with an offset array.
```rust
pub struct FlatAttackTable {
    pub offsets: [usize; 64],
    pub table: Vec<u64>,
}

// Access becomes:
let offset = table.offsets[sq];
let idx = _pext_u64(blockers, masks[sq]) as usize;
let attack = unsafe { *table.table.get_unchecked(offset + idx) };
```
This removes one level of pointer indirection and bounds checking (if using unsafe), and drastically improves cache locality.

### 2. Remove LazyLock Checks (Optional)
**Current State**:
`LazyLock` performs an atomic check on every access to ensure initialization.

**Proposed Change**:
While `LazyLock` is efficient, heavily hit hot paths like move generation *might* benefit from initializing these tables at the start of `main` and accessing them via a raw static pointer or a `OnceLock` that is assumed initialized in the hot path (using `get()` instead of `get_or_init()` after warmup). However, the compiler often optimizes the check if it can see the initialization is done. Flattening the table is the higher priority.