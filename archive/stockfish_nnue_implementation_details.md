# Stockfish NNUE Analysis - Part 2: Implementation Details

## 1. OrientTable & Perspective Flipping

The `OrientTable` is used to canonicalize the board such that the king is always on the kingside (files E-H). This reduces the dimensionality of the feature space by exploiting horizontal symmetry.

**Logic:**
$$ \text{OrientedSquare} = \text{Square} \oplus \text{OrientTable}[\text{Perspective}][\text{KingSquare}] $$

**Values:**
The table contains constants that either flip the file (XOR with 7) or do nothing (XOR with 0).

**Rust Implementation (Tables):**
```rust
// SQ_A1=0, SQ_H1=7, SQ_A8=56, SQ_H8=63

pub const ORIENT_TABLE: [[u8; 64]; 2] = [
    // White Perspective
    [
        7, 7, 7, 7, 0, 0, 0, 0, // Rank 1
        7, 7, 7, 7, 0, 0, 0, 0, // Rank 2
        7, 7, 7, 7, 0, 0, 0, 0, // Rank 3
        7, 7, 7, 7, 0, 0, 0, 0, // Rank 4
        7, 7, 7, 7, 0, 0, 0, 0, // Rank 5
        7, 7, 7, 7, 0, 0, 0, 0, // Rank 6
        7, 7, 7, 7, 0, 0, 0, 0, // Rank 7
        7, 7, 7, 7, 0, 0, 0, 0  // Rank 8
    ],
    // Black Perspective (Note: Stockfish uses 180-degree flipped values for Black)
    [
        63, 63, 63, 63, 56, 56, 56, 56, // Rank 1 (A1..H1)
        63, 63, 63, 63, 56, 56, 56, 56, // Rank 2
        63, 63, 63, 63, 56, 56, 56, 56, // Rank 3
        63, 63, 63, 63, 56, 56, 56, 56, // Rank 4
        63, 63, 63, 63, 56, 56, 56, 56, // Rank 5
        63, 63, 63, 63, 56, 56, 56, 56, // Rank 6
        63, 63, 63, 63, 56, 56, 56, 56, // Rank 7
        63, 63, 63, 63, 56, 56, 56, 56  // Rank 8
    ]
];
```

## 2. King Bucketing

To reduce the number of features, King positions are grouped into buckets. The `KingBuckets` table provides the base feature index offset for a given king square.

**Values:**
The values are pre-multiplied by `PS_NB` (704).

**Rust Implementation:**
```rust
const PS_NB: u32 = 704;
const fn B(v: u32) -> u32 { v * PS_NB }

pub const KING_BUCKETS: [[u32; 64]; 2] = [
    // White
    [
        B(28), B(29), B(30), B(31), B(31), B(30), B(29), B(28),
        B(24), B(25), B(26), B(27), B(27), B(26), B(25), B(24),
        B(20), B(21), B(22), B(23), B(23), B(22), B(21), B(20),
        B(16), B(17), B(18), B(19), B(19), B(18), B(17), B(16),
        B(12), B(13), B(14), B(15), B(15), B(14), B(13), B(12),
        B( 8), B( 9), B(10), B(11), B(11), B(10), B( 9), B( 8),
        B( 4), B( 5), B( 6), B( 7), B( 7), B( 6), B( 5), B( 4),
        B( 0), B( 1), B( 2), B( 3), B( 3), B( 2), B( 1), B( 0) 
    ],
    // Black
    [
        B( 0), B( 1), B( 2), B( 3), B( 3), B( 2), B( 1), B( 0),
        B( 4), B( 5), B( 6), B( 7), B( 7), B( 6), B( 5), B( 4),
        B( 8), B( 9), B(10), B(11), B(11), B(10), B( 9), B( 8),
        B(12), B(13), B(14), B(15), B(15), B(14), B(13), B(12),
        B(16), B(17), B(18), B(19), B(19), B(18), B(17), B(16),
        B(20), B(21), B(22), B(23), B(23), B(22), B(21), B(20),
        B(24), B(25), B(26), B(27), B(27), B(26), B(25), B(24),
        B(28), B(29), B(30), B(31), B(31), B(30), B(29), B(28) 
    ]
];
```

## 3. PieceSquare Base Offsets

`PieceSquareIndex` maps `(Color, PieceType)` to a base feature index.

**Mapping (PS_NB = 704):**
| Piece | Value | Base Index |
| :--- | :--- | :--- |
| W_Pawn | 1 | 0 |
| W_Knight | 2 | 128 |
| W_Bishop | 3 | 256 |
| W_Rook | 4 | 384 |
| W_Queen | 5 | 512 |
| W_King | 6 | 640 |
| B_Pawn | 9 | 64 |
| B_Knight | 10 | 192 |
| ... | ... | ... |

**Formula:**
`Offset = PieceType_Index * 64`.
*   White Pieces: `[0, 2, 4, 6, 8, 10] * 64`.
*   Black Pieces: `[1, 3, 5, 7, 9, 10] * 64`.
*   (Note: The table in `half_ka_v2_hm.h` interleaves them differently. Use the exact array values).

## 4. Accumulator Refresh (Pseudocode)

```rust
fn refresh_accumulator(
    position: &Position, 
    transformer: &FeatureTransformer
) -> Accumulator {
    let mut acc = Accumulator::new();
    
    // 1. Initialize with Biases
    acc.white = transformer.biases.clone(); // 3072 values
    acc.black = transformer.biases.clone();
    
    // 2. Add Active Features
    for (color, perspective) in [(White, 0), (Black, 1)] {
        let ksq = position.king_square(color);
        
        // Iterate ALL pieces on the board
        for square in 0..64 {
            let piece = position.piece_at(square);
            if piece == None { continue; }
            
            // Calculate Feature Index
            let index = make_index(perspective, square, piece, ksq);
            
            // Add Weights (Accumulate column)
            // Weight column size = 3072
            let weight_offset = index * 3072; 
            for i in 0..3072 {
                acc[perspective][i] += transformer.weights[weight_offset + i];
            }
            
            // Add PSQT Weights
            // PSQT column size = 8
            let psqt_offset = index * 8;
            for i in 0..8 {
                acc_psqt[perspective][i] += transformer.psqt_weights[psqt_offset + i];
            }
        }
    }
    return acc;
}
```

## 5. Transform Function (Exact)

The `transform` function converts the 3072 `int16` accumulator values into the 3072 `int8` input for the first layer. It fundamentally performs a **pairwise multiplication** of the two halves of the accumulator.

**Algorithm:**

```rust
fn transform(
    acc: &Accumulator, 
    us: Color, 
    bucket: usize
) -> ([i8; 3072], i32) {
    let mut output = [0i8; 3072];
    
    // Perspectives: Us, Them
    let perspectives = [us, !us];
    
    for (p_idx, &perspective) in perspectives.iter().enumerate() {
        let offset = 1536 * p_idx; // 0 for Us, 1536 for Them
        let acc_slice = &acc.accumulation[perspective];
        
        for j in 0..1536 {
            // Take value from first half and second half
            let sum0 = acc_slice[j];
            let sum1 = acc_slice[j + 1536];
            
            // Clamp to [0, 254] (Since weights were x2, 127*2 = 254)
            let c0 = sum0.clamp(0, 254) as i32;
            let c1 = sum1.clamp(0, 254) as i32;
            
            // Multiply and divide by 512
            let product = (c0 * c1) / 512;
            
            output[offset + j] = product as i8;
        }
    }
    
    // Calculate PSQT Score
    let psqt_us = acc.psqt_accumulation[us][bucket];
    let psqt_them = acc.psqt_accumulation[!us][bucket];
    let psqt_score = (psqt_us - psqt_them) / 2;
    
    (output, psqt_score)
}
```

## 6. Layer Activation Details

*   **WeightScaleBits**: 6
*   **OutputScale**: 16

### SqrClippedReLU
Used for the first 15 outputs of Layer 1.
$$ y = \min(127, \lfloor (x^2) / 2^{19} \rfloor) $$
Implementation: `min(127, (x * x) >> 19)`

### ClippedReLU
Used for the rest.
$$ y = \min(127, \lfloor x / 2^6 \rfloor) $$
Implementation: `min(127, x >> 6)`

## 7. Layer Concatenation
Layer 1 has 32 outputs (actually 16 outputs x 2 groups? No).
*   **Layer 1 Config**: `L2 = 15`. `FC_0_OUTPUTS = 15`.
*   Layer definition: `AffineTransformSparseInput<3072, 16>` (Output size 16).
*   Output 0..14 (15 values): Passed through `SqrClippedReLU`.
*   Output 15 (1 value): Passed through `ClippedReLU`.
*   Total 16 values.
*   But `Layer 2` expects 32 inputs?
    *   Check `nnue_architecture.h`: `fc_0_out` buffer size.
    *   Code says: `std::memcpy(buffer.ac_sqr_0_out + FC_0_OUTPUTS, buffer.ac_0_out, ...)`
    *   It effectively duplicates or processes the output twice?
    *   Re-reading `nnue_architecture.h`:
        *   `fc_0`: Output 16.
        *   `ac_sqr_0`: Processes `fc_0` output. `SqrClippedReLU<16>`.
        *   `ac_0`: Processes `fc_0` output. `ClippedReLU<16>`.
        *   Wait, the template is `<FC_0_OUTPUTS + 1>`. `FC_0_OUTPUTS` is 15. So 16.
        *   It processes the **same** 16 outputs through **two different** activation functions.
        *   The input to Layer 2 is the concatenation of `SqrClippedReLU(out[0..15])` AND `ClippedReLU(out[0..15])`?
        *   **NO**.
        *   Look at `propagate`:
            ```cpp
            fc_0.propagate(..., buffer.fc_0_out); // Writes 16 values
            ac_sqr_0.propagate(buffer.fc_0_out, buffer.ac_sqr_0_out); // Writes 16 bytes (first 15 valid SQR?)
            ac_0.propagate(buffer.fc_0_out, buffer.ac_0_out); // Writes 16 bytes
            memcpy(buffer.ac_sqr_0_out + 15, buffer.ac_0_out, 15 bytes??);
            ```
        *   Actually `FC_0_OUTPUTS` is 15. The layer size is 16.
        *   `ac_sqr_0` is size 16.
        *   `ac_0` is size 16.
        *   `ac_sqr_0` computes SqrClippedReLU for all 16?
        *   The memory copy suggests: `dest = ac_sqr_0_out + 15`. `src = ac_0_out`. Size `15`.
        *   So the buffer becomes:
            *   Indices 0..14: `SqrClippedReLU(fc_0[0..14])`
            *   Indices 15..29: `ClippedReLU(fc_0[0..14])`??
            *   Total 30 values.
        *   What about index 15 of `fc_0`?
            *   It is used in the **Final Scaling** formula as a residual! `fwdOut = buffer.fc_0_out[15] ...`
            *   It is NOT passed to Layer 2.

**Conclusion:**
Layer 2 Input (32 bytes):
*   0..14: `SqrClippedReLU(Layer1[0..14])`
*   15..29: `ClippedReLU(Layer1[0..14])`
*   30..31: Padding (Zero? Or uninitialized? AffineTransform usually assumes 0 for padding).

## 8. Weight Scrambling
For the MVP, we can ignore scrambling if we implement `AffineTransform::propagate` using a naive loop `output[i] += weight[i][j] * input[j]`.
The scrambling is only relevant if we load the weights as a bulk blob and then try to access them linearly in a SIMD kernel that expects scrambling.
**Recommendation**: Read weights linearly. Implement naive matrix multiplication. If performance is needed later, implement scrambling + SIMD.

## 9. Big vs Small Net
*   **Small Net**: `nn-37f18f62d772.nnue` (128 dims, 15/32 hidden).
*   **Big Net**: `nn-1c0000000000.nnue` (3072 dims, 15/32 hidden).
*   **Usage**: Calculate `simple_eval`. If high imbalance (> 9.6 pawns), use Small. Else use Big.
*   **Hybrid**: If Small net returns near-draw (< 2.36 pawns), re-run Big net.

## 10. Compile-Time Constants
*   `SHIFT = 6` (WeightScaleBits)
*   `OutputScale = 16`
*   `L1_SIZE = 3072` (Big) / `128` (Small)
*   `L2_SIZE = 15`
*   `L3_SIZE = 32`

## 11. Incremental Updates (DirtyPiece)
`DirtyPiece` tracks changes to avoid full refresh.
*   If `DirtyPiece.dirty_num == 0` (Null move?): Copy accumulator.
*   If `dirty_num > 0`:
    *   Subtract weights for `dirty.from` / `dirty.piece`.
    *   Add weights for `dirty.to` / `dirty.piece`.
    *   Handle promotions (Remove Pawn, Add Queen).
    *   **Refresh Condition**: If `King` moves, `make_index` changes for ALL pieces. Must do Full Refresh.

## 12. Network File structure example
Not needed for MVP if we strictly follow the LEB128 + Header spec. The header verification (Magic + Version) is sufficient.

```
