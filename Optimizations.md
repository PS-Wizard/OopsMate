# Optimization Report

## 1. Critical Performance Bottlenecks

### A. The "Piece Lookup" Catastrophe (`piece_at`)
**Severity:** CRITICAL
**Location:** `src/position.rs` -> `piece_at`
**Impact:** Move Generation, Make Move, SEE, Evaluation.

Currently, `Position` is a "Pure Bitboard" implementation. To find out what piece is on a square (e.g., during `make_move` capture handling or `SEE`), the engine loops through 6 piece types and 2 colors.
```rust
// Current O(12) Implementation
pub fn piece_at(&self, sq: usize) -> Option<(Piece, Color)> {
    // Loops over 2 colors * 6 pieces = 12 branches/memory accesses
}
```
This function is called:
1.  **Every `make_move`**: To identify captured pieces.
2.  **Every `SEE` call**: To identify the victim and the attacker.
3.  **Every QSearch node**: SEE is called for every capture.

**Solution: Mailbox Array**
Add a 64-element array to `Position` to store the piece at each square.
```rust
pub struct Position {
    // ... bitboards ...
    pub board: [Option<(Piece, Color)>; 64], // or use a specific u8 encoding
}
```
This turns `piece_at` into an **O(1)** array index. This single change could speed up `SEE` by 5-10x and `make_move` significantly.

### B. Static Exchange Evaluation (SEE) Overhead
**Severity:** HIGH
**Location:** `src/see.rs`

SEE is theoretically correct but implementation details make it slow:
1.  **Initialization:** Calls `piece_at(from)` (Slow, see above).
2.  **Attack Generation:** Inside the SEE loop, `attackers_to_board` is not fully optimized. It re-calculates PEXT lookups for both Rooks and Bishops every iteration, even though we only really need to scan for X-rays behind the removed piece.

**Optimization:**
1.  Fix `piece_at` (See A).
2.  Pass the moving piece `Piece` to `see()` directly from the MoveGen/Move struct if available, avoiding the initial lookup.

### C. `make_move` Efficiency
**Severity:** MEDIUM
**Location:** `src/position.rs` -> `apply_move`

*   `apply_move` calls `piece_at(to)` to handle captures.
*   `apply_move` calls `piece_at(from)` (indirectly or explicitly) to clear bits.
*   It updates the Hash (Zobrist) by XORing random numbers. This is standard, but efficient lookup is key.

## 2. Correctness & Safety

### A. `MoveCollector` Initialization
**Severity:** LOW (Potential UB)
**Location:** `src/types.rs`

```rust
pub fn new() -> Self {
    MoveCollector {
        moves: unsafe { MaybeUninit::uninit().assume_init() }, // Technically risky
        count: 0,
    }
}
```
While likely safe for an array of `MaybeUninit`, `assume_init()` on uninitialized memory is technically Undefined Behavior if the type has validity invariants. `[MaybeUninit<T>; N]` technically doesn't, but it's better to use `MaybeUninit::uninit_array()` (if unstable) or simpler `mem::zeroed()` (if performance allows, though we want to avoid that).
*Recommendation:* Since `MaybeUninit` is a union, it's fine. But ensure this doesn't get optimized out or cause weirdness.

## 3. Micro-Optimizations

### A. Branchless SEE LVA
In `src/see.rs`, `get_lva` checks piece types in order (Pawn -> Knight -> ...).
This is good, but with `tzcnt` (trailing zeros), we can do this faster if we have a bitmask of "all attackers of current side".
1.  `let side_attackers = attackers & self.colors[side].0;`
2.  `let pawns = side_attackers & self.pieces[Pawn];`
3.  If `pawns != 0`, return `lsb(pawns)`.
This is what you have. It is good.

### B. PEXT Redundancy
In `src/position.rs`, `is_square_attacked` performs PEXT lookups.
If you check for check (King Safety) and then Castling safety, you might probe the same lines/squares.

## 4. Proposed Plan

1.  **Refactor `Position`**: Add `mailbox: [u8; 64]` (0=Empty, 1-6=White Pieces, 9-14=Black Pieces).
2.  **Update `piece_at`**: Use Mailbox.
3.  **Update `make_move`**: Maintain Mailbox.
4.  **Profile**: Measure `SEE` impact again. It should be negligible now.
