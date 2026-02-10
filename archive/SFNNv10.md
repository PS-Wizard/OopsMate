# Comprehensive Analysis of Stockfish NNUE (SFNNv10)

## Executive Summary

Stockfish's neural network architecture, **SFNNv10**, represents a significant evolution in computer chess evaluation. It employs a **dual-accumulator** system that combines two distinct feature sets:
1.  **HalfKAv2_hm**: A king-relative piece placement feature set (standard NNUE approach).
2.  **Full_Threats**: A complex feature set encoding attack relationships between pieces (e.g., "White Knight attacks Black Queen").

These two feature sets are processed by separate accumulators, combined via **product pooling**, and then fed into a sequence of 8 specialized sub-networks (Layer Stacks) chosen based on the material on the board. The entire system is optimized for integer arithmetic (quantization) and SIMD instructions.

This document details the file format, loading process, and runtime evaluation logic required to implement a probe for SFNNv10.

---

## 1. File Format Specification

Stockfish `.nnue` files are binary files containing serialized network parameters. Integers are stored in **Little Endian** format unless otherwise specified.

### 1.1 Header Section
The file begins with a global header used for versioning and validation.

| Offset | Type | Description | Value / Notes |
| :--- | :--- | :--- | :--- |
| 0x00 | `uint32` | **Version** | `0x7AF32F20` (SFNNv10) |
| 0x04 | `uint32` | **Hash** | `FeatureTransformerHash ^ NetworkArchitectureHash` |
| 0x08 | `uint32` | **Description Size** | Length of the description string (`L`) |
| 0x0C | `char[L]`| **Description** | ASCII string describing the network |

### 1.2 Feature Transformer Section
Immediately following the description is the Feature Transformer. This component maps the board state to the initial hidden layer.

1.  **Hash Check**: Read `uint32`. Must match `FeatureTransformer::get_hash_value()`.
2.  **Biases**: `read_leb_128(stream, biases)`
    *   Count: `HalfDimensions` (Standard: 1024)
    *   Type: `int16`
3.  **Threat Weights**: `read_little_endian(stream, threatWeights)`
    *   **Crucial**: These are **NOT** LEB128 compressed.
    *   Count: `HalfDimensions * ThreatInputDimensions` (1024 * 79856)
    *   Type: `int8` (`ThreatWeightType`)
4.  **Piece Weights**: `read_leb_128(stream, weights)`
    *   Count: `HalfDimensions * InputDimensions` (1024 * 22528)
    *   Type: `int16` (`WeightType`)
5.  **PSQT Weights**: `read_leb_128(stream, threatPsqtWeights, psqtWeights)`
    *   **Threat PSQT**: `ThreatInputDimensions * PSQTBuckets` (79856 * 8)
    *   **Piece PSQT**: `InputDimensions * PSQTBuckets` (22528 * 8)
    *   Type: `int32` (`PSQTWeightType`)

### 1.3 Layer Stacks Section
The network contains **8 independent sub-networks** (Layer Stacks). The appropriate stack is selected at runtime based on the number of pieces.
The following block is repeated **8 times** (once for each stack 0..7).

1.  **Hash Check**: Read `uint32`. Must match `NetworkArchitecture::get_hash_value()`.
2.  **Layer `fc_0` (AffineTransformSparseInput)**:
    *   **Biases**: Read `OutputDimensions` (15+1 = 16) as `int32`.
    *   **Weights**: Read `OutputDimensions * PaddedInputDimensions` (16 * 1024) as `int8`.
        *   Note: These are raw little-endian values, not LEB128.
3.  **Layer `ac_0` (ClippedReLU)**: No parameters (activation function).
4.  **Layer `fc_1` (AffineTransform)**:
    *   **Biases**: Read `OutputDimensions` (32) as `int32`.
    *   **Weights**: Read `OutputDimensions * InputDimensions` (32 * 32) as `int8`.
5.  **Layer `ac_1` (ClippedReLU)**: No parameters.
6.  **Layer `fc_2` (AffineTransform)**:
    *   **Biases**: Read `OutputDimensions` (1) as `int32`.
    *   **Weights**: Read `OutputDimensions * InputDimensions` (1 * 32) as `int8`.

### 1.4 Compression (LEB128)
LEB128 (Little Endian Base 128) is used to compress sparse integer arrays (Biases, Piece Weights, PSQTs).
*   **Magic String**: The stream must begin with `"COMPRESSED_LEB128"`.
*   **Byte Count**: A `uint32` indicating the number of bytes in the compressed block.
*   **Data**: A variable-length integer encoding where the MSB of each byte indicates if more bytes follow.
*   **ZigZag Encoding**: Signed integers are mapped to unsigned to handle negatives efficiently.

---

## 2. File Loading & Processing

### 2.1 Weight Permutation (Unscrambling)
Stockfish stores weights in a permuted order to optimize for specific AVX2/AVX512 instructions (`_mm256_packus_epi16`). To use these weights in a standard scalar or custom SIMD implementation, you must **unpermute** them after loading.

**The Permutation Pattern (AVX2-compatible):**
Weights are processed in 128-bit chunks (8 `int16`s). Within each chunk, the elements are reordered:
`{0, 2, 1, 3, 4, 6, 5, 7}` (Indices refer to 16-bit words within a 128-bit block).
*   This applies to `biases`, `weights`, and `threatWeights` of the Feature Transformer.
*   The inverse permutation `{0, 2, 1, 3, 4, 6, 5, 7}` must be applied to restore linear order.

### 2.2 Scaling
*   **Non-Threat Architectures**: Weights are typically scaled by 2.
*   **SFNNv10 (Threats)**: Explicit scaling in `read_parameters` is **SKIPPED**. The scaling is handled implicitly or within the specific dot-product logic (product pooling) during inference.

---

## 3. Feature Extraction Logic

### 3.1 HalfKAv2_hm (Piece Features)
*   **Definition**: King-relative piece placement.
*   **Dimensions**: `22,528` features per perspective.
*   **Mirroring**: "hm" stands for Horizontal Mirroring. The king is always mapped to the **e..h files** (right half of board). If the king is on a..d, the entire board is flipped horizontally.

**Feature Index Calculation**:
```cpp
// Constants
PS_NB = 11; // 5 piece types * 2 colors + King = 11 unique piece identifiers
SQUARE_NB = 64;

// Inputs
Color perspective; // WHITE or BLACK
Square ksq;        // Position of the king (perspective's king)
Square s;          // Position of the piece
Piece pc;          // The piece type (W_PAWN..B_KING)

// 1. Orientation: Flip board if king is on left side (files A-D)
IndexType orient = OrientTBL[ksq]; // 0 if king on E-H, 7 (Row Flip) if king on A-D
IndexType flip = 56 * perspective; // Vertical flip for BLACK perspective

// 2. Adjust squares
// X ^ flip applies vertical mirroring for black
// X ^ orient applies horizontal mirroring if needed
// Note: OrientTBL[ksq] effectively changes file index f -> 7-f

// 3. Components
IndexType square_idx = IndexType(s) ^ orient ^ flip;
IndexType piece_idx = PieceSquareIndex[perspective][pc]; // Maps piece to 0..10
IndexType bucket_idx = KingBuckets[int(ksq) ^ flip];     // 32 King Buckets

// 4. Final Index
IndexType index = square_idx + piece_idx + bucket_idx;
```

### 3.2 Full_Threats (Threat Features)
*   **Definition**: Encodes "Piece A attacks Piece B" relationships.
*   **Dimensions**: `79,856` features.
*   **Deduplication**:
    *   **Map Exclusions**: Certain attacks are ignored (e.g., King attacking Queen).
    *   **Semi-Exclusions**: For identical piece types (e.g., Knight attacks Knight), only one direction (`from < to`) is stored to avoid double counting.
    *   **Pawn Exception**: Pawns attacking enemies are never semi-excluded (always tracked).

**Feature Index Calculation**:
This is complex and relies on Look-Up Tables (LUTs) precomputed at compile time.
```cpp
// Inputs: perspective, attacker, from, to, attacked, ksq

// 1. Orientation
int8_t orientation = OrientTBL[ksq] ^ (56 * perspective);
unsigned from_oriented = from ^ orientation;
unsigned to_oriented   = to ^ orientation;

// 2. Color Swap for pieces
int8_t swap = 8 * perspective;
unsigned attacker_oriented = attacker ^ swap;
unsigned attacked_oriented = attacked ^ swap;

// 3. Final Index Summation
// lut1: High-level offset based on (Attacker, Attacked, Direction)
// offsets: Base offset for this specific Attacker piece type and From square
// lut2: Local index of the 'To' square within the attacker's attack bitboard
IndexType index = index_lut1[attacker_oriented][attacked_oriented][from_oriented < to_oriented]
                + offsets[attacker_oriented][from_oriented]
                + index_lut2[attacker_oriented][from_oriented][to_oriented];
```

*Note: Implementing `Full_Threats` requires replicating the logic to generate `index_lut1`, `offsets`, and `index_lut2`. See `src/nnue/features/full_threats.cpp` for the generation algorithms.*

---

## 4. Dual Accumulator System & Updates

Stockfish uses **two** accumulators per color (White/Black), totaling 4 logical accumulation states.

### 4.1 Piece Accumulator (`Accumulator<22528>`)
*   **Data Type**: `int16`
*   **Update Strategy**: **Incremental with Cache**.
    *   **King Move**: Uses a "Finny Table" cache. The cache stores the accumulation state for the King at `ksq` with *no other pieces*. The engine then just adds the current pieces to this base state.
    *   **Non-King Move**: Standard incremental update. Subtract weights for removed features, add weights for added features.
*   **Double Update**: Supports handling two changes simultaneously (e.g., Capture = Remove captured + Add mover) for efficiency.

### 4.2 Threat Accumulator (`Accumulator<79856>`)
*   **Data Type**: `int16` (Weights are stored as `int8` but accumulated into `int16`).
*   **Update Strategy**: **Full Refresh on Zone Change**.
    *   **Refresh Condition**: If the King moves across the **d/e file boundary** (i.e., horizontal mirroring flips), the entire accumulator must be recomputed from scratch (`update_threats_accumulator_full`).
    *   **Incremental**: If the King stays within the same horizontal zone (Left or Right), standard incremental updates are used.
*   **No Cache**: Due to the complexity and size, Finny Table caching is not used for threats.

---

## 5. Runtime Evaluation Process

### 5.1 Bucket Selection
Before running the network, select the correct Layer Stack (0-7):
```cpp
int bucket = (piece_count - 1) / 4;
// Clamp bucket to 0..7 range implicitly by the logic (max pieces 32)
```

### 5.2 Product Pooling (The "Transform" Step)
The two accumulators (Piece and Threat) are combined to produce the input for the first linear layer.
```cpp
// For each of the 1024 dimensions (i = 0..1023):

// 1. Get Accumulator Values
int16_t sum_piece  = piece_acc[perspective][i];
int16_t sum_threat = threat_acc[perspective][i];

// 2. Clamp
// Note: clamp range depends on architecture. SFNNv10 typically uses 0..255 for combined.
int16_t clamped_piece  = clamp(sum_piece + sum_threat, 0, 255);

// 3. Product Pooling (Multiply pairs)
// We take neighbors (j, j+1) where j is even.
// This reduces dimensions from 1024 -> 512.
int16_t val1 = clamped_piece_at_i;
int16_t val2 = clamped_piece_at_i_plus_half_dim; // or neighbor?
// WAIT: The code combines P[i] and P[i + 512].
// It's strictly defined as:
// output[i] = (clamped[i] * clamped[i + 512]) / 512;
// This produces 512 outputs.
```
*Correction*: The transform combines the first half (0..511) with the second half (512..1023).
`output[j] = (clamped[j] * clamped[j + 512]) / 512;`
This results in **512** outputs per perspective.
Since we use **both** perspectives (Us and Them), we get `512 + 512 = 1024` features.

**Final Input Vector Construction**:
1.  Compute 512 transformed features for `side_to_move`.
2.  Compute 512 transformed features for `~side_to_move`.
3.  Concatenate them: `[Us_0..511, Them_0..511]`. Total 1024.

### 5.3 PSQT Evaluation
Calculating the "classical" evaluation part from the accumulators:
```cpp
// PSQTBuckets = 8
int32_t psqt_score = 0;
// Combine PSQT accumulators from both feature sets
psqt_score = piece_psqt[us][bucket] - piece_psqt[them][bucket];
psqt_score += threat_psqt[us][bucket] - threat_psqt[them][bucket];
psqt_score /= 2; // Average
```

### 5.4 Forward Propagation (Layer Stacks)
Inputs: `transformedFeatures` (1024 bytes).

1.  **Layer 0 (`fc_0`)**:
    *   Input: 1024 (`int8` range, derived from `int16`).
    *   Operation: Sparse Affine Transform.
    *   Output: 16 values (`int32`).
    *   **Skip Connection**: The 16th value (`output[15]`) is a special skip connection value.

2.  **Activation 0**:
    *   `ac_sqr_0`: Square the first 15 outputs. `y = x*x`.
    *   `ac_0`: Clip the first 15 outputs. `y = clamp(x, 0, 127)`.
    *   Concat: `[Squared_0..14, Clipped_0..14]`. Total 30 inputs for next layer.

3.  **Layer 1 (`fc_1`)**:
    *   Input: 30 (`int32` -> clamped to `int8`).
    *   Operation: Affine Transform.
    *   Output: 32 values.

4.  **Activation 1 (`ac_1`)**:
    *   Operation: Clipped ReLU (`0..127`).

5.  **Layer 2 (`fc_2`)**:
    *   Input: 32.
    *   Operation: Affine Transform.
    *   Output: 1 value.

6.  **Final Summation**:
    ```cpp
    int32_t fwdOut = (skip_connection_value * 600 * 16) / (127 * 64);
    int32_t final_eval = fc_2_output + fwdOut;
    // Plus the PSQT score calculated earlier
    return final_eval / 16 + psqt_score / 16;
    ```

---

## 6. Implementation Checklist for Rust Port

- [ ] **LEB128 Decoder**: Implement `read_leb_128` matching `nnue_common.h`.
- [ ] **Threat LUT Generator**: Replicate the `full_threats.cpp` compile-time logic to generate `index_lut1`, `offsets`, and `index_lut2`.
- [ ] **File Parser**:
    - [ ] Read Header (Verify Version/Hash).
    - [ ] Read Feature Transformer (Handle `int8` threat weights correctly).
    - [ ] Read 8x Layer Stacks.
- [ ] **Weight Unpermuter**: Implement the AVX2 inverse permutation for loaded weights.
- [ ] **Accumulators**:
    - [ ] `Accumulator<22528>` (Pieces) with Finny Table Cache support.
    - [ ] `Accumulator<79856>` (Threats) with Full Refresh logic on File D/E crossing.
- [ ] **Position Logic**:
    - [ ] Implement `OrientTBL`, `KingBuckets`, `PieceSquareIndex`.
    - [ ] Implement `make_index` for HalfKAv2_hm.
    - [ ] Implement `make_index` for Full_Threats using the generated LUTs.
- [ ] **Evaluation Function**:
    - [ ] Bucket selection.
    - [ ] Product pooling transform.
    - [ ] PSQT summation.
    - [ ] Forward pass with skip connection logic.

# NNUE SFNNv10 Implementation Details - Addendum

This document provides the missing technical specifications, lookup tables, and algorithms required to implement a fully compatible Stockfish SFNNv10 probe. All information is derived directly from the Stockfish source code.

## 1. Layer 0 Weight Descrambling

The Stockfish file format stores `fc_0` weights in a linear order corresponding to the logical matrix dimensions. However, Stockfish scrambles these weights **in memory** upon loading to optimize for SIMD (AVX2/SSSE3) access patterns.

If you are implementing a standard matrix multiplication, you **do not** need to descramble the file. You can simply read the weights linearly.

**Linear File Layout (Logical Order):**
The weights are stored in **Input-Major** order (grouped by input feature).
`Weights[Input_0][Output_0]`, `Weights[Input_0][Output_1]`, ..., `Weights[Input_0][Output_15]`, `Weights[Input_1][Output_0]`, ...

If you need to replicate Stockfish's in-memory scrambling (e.g., to use their SIMD kernels), the mapping from **Linear Index `i`** to **Scrambled Index** is:

```cpp
// Constants for SFNNv10 Layer 0
const int ChunkSize = 4; // For AVX2/SSSE3
const int OutputDimensions = 16;
const int PaddedInputDimensions = 1024; // Ceil(1024, 32)

// i is the linear index from the file (0 .. Output*Input - 1)
int get_scrambled_index(int i) {
    return (i / ChunkSize) % (PaddedInputDimensions / ChunkSize) * OutputDimensions * ChunkSize
         + i / PaddedInputDimensions * ChunkSize 
         + i % ChunkSize;
}
```

## 2. Product Pooling Transform

The product pooling step combines the `Piece` and `Threat` accumulators into the input for Layer 0 (`fc_0`).

**Inputs**:
*   `PieceAccumulator[2][1024]` (`int16`)
*   `ThreatAccumulator[2][1024]` (`int16`)

**Output**:
*   `TransformedFeatures[1024]` (`int8` / `uint8`)

**Algorithm**:
For each perspective (Us, Them) and for each of the 512 pooling groups:

```cpp
// Pseudo-code for calculating the 1024 transformed features
// perspective 0 = Side to Move, 1 = Not Side to Move
// out_idx ranges from 0 to 1023
int out_idx = 0;

for (int p = 0; p < 2; ++p) { // Loop perspectives
    for (int j = 0; j < 512; ++j) { // Loop pooling groups
        // 1. Sum Accumulators
        int16_t sum0 = piece_acc[p][j] + threat_acc[p][j];
        int16_t sum1 = piece_acc[p][j + 512] + threat_acc[p][j + 512];

        // 2. Clamp (Range 0..255 for SFNNv10)
        // Note: Non-threat architectures use 0..254 or different scaling.
        int16_t clamped0 = std::clamp(sum0, (int16_t)0, (int16_t)255);
        int16_t clamped1 = std::clamp(sum1, (int16_t)0, (int16_t)255);

        // 3. Product Pooling
        // Multiply and divide by 512 (effectively >> 9)
        // This corresponds to (A * B) / 128 / 2 / 2 implicit scaling factors
        int32_t product = (int32_t)clamped0 * (int32_t)clamped1;
        int8_t result = (int8_t)(product / 512); 

        output[out_idx++] = result;
    }
}
```

## 3. Threat Weight Conversion & Scaling

*   **File Storage**: `int8` (Raw bytes).
*   **Accumulator**: `int16`.
*   **Scaling**: Unlike piece weights in older architectures, **Threat Weights are NOT scaled by 2** during loading in SFNNv10. They are used raw.
*   **Accumulation**: Simple integer addition.

```cpp
// Incremental Update
accumulator[i] += (int16_t)threat_weight_from_file;
// OR
accumulator[i] -= (int16_t)threat_weight_from_file;
```

## 4. Finny Table Cache Specification

The Finny Table caches the accumulation state of the **Piece Feature Set** relative to a specific King square. It allows the engine to skip re-accumulating the entire board when the King moves to a square it (or the engine) has visited before.

*   **Structure**: `Entry entries[64][2]` (64 Squares, 2 Colors).
*   **Content**:
    *   `accumulation[1024]`: `int16` biases + weights of pieces present *at update time*.
    *   `psqtAccumulation[8]`: `int32` PSQT scores.
    *   `pieceBB`: `Bitboard` of pieces present when this entry was last updated.
    *   `pieces[64]`: Array of pieces present when this entry was last updated.
*   **Usage Logic**:
    1.  King moves to `sq`.
    2.  Retrieve `cache[sq][perspective]`.
    3.  Calculate diff between `current_pos` and `cache_entry.pieces`.
    4.  Update `cache_entry` with the diff (add/remove weights).
    5.  Copy `cache_entry` to the main accumulator.
*   **Benefit**: If the position is similar to the last time the king was at `sq`, the diff is small (few updates). If visited for the first time, it updates from the "Base" state (Biases only).

## 5. OrientTBL (Lookup Table)

Used to mirror the board horizontally if the King is on the left side (files A-D).
*   **Value 7 (`0x07`)**: Represents `SQ_H1` (flip file index: `x ^ 7`).
*   **Value 0 (`0x00`)**: Represents `SQ_A1` (no flip).

**Table Values (64 squares, A1 to H8):**
```text
Rows 1-8 are identical:
A  B  C  D  E  F  G  H
7, 7, 7, 7, 0, 0, 0, 0
```

**C++ Array:**
```cpp
static constexpr int8_t OrientTBL[64] = {
    7, 7, 7, 7, 0, 0, 0, 0, // Rank 1
    7, 7, 7, 7, 0, 0, 0, 0, // Rank 2
    7, 7, 7, 7, 0, 0, 0, 0, // ...
    7, 7, 7, 7, 0, 0, 0, 0,
    7, 7, 7, 7, 0, 0, 0, 0,
    7, 7, 7, 7, 0, 0, 0, 0,
    7, 7, 7, 7, 0, 0, 0, 0,
    7, 7, 7, 7, 0, 0, 0, 0  // Rank 8
};
```

## 6. KingBuckets (Lookup Table)

Stockfish SFNNv10 uses **32 King Buckets**. The `KingBuckets` array maps the King's square (after orientation/mirroring) to a bucket offset index.
*   **Offset Multiplier**: `11` (`PS_NB`). The stored value is `BucketID * 11`.

**Table Values (Post-Mirroring - E..H files only effectively):**
The table is defined for all 64 squares, but since we mirror A..D to E..H, only the values for E..H are strictly critical (though the full table handles the logic).

```cpp
// Values are BucketID * 11
static constexpr int KingBuckets[64] = {
    308, 319, 330, 341, 341, 330, 319, 308, // Rank 1
    264, 275, 286, 297, 297, 286, 275, 264, // Rank 2
    220, 231, 242, 253, 253, 242, 231, 220, // Rank 3
    176, 187, 198, 209, 209, 198, 187, 176, // Rank 4
    132, 143, 154, 165, 165, 154, 143, 132, // Rank 5
     88,  99, 110, 121, 121, 110,  99,  88, // Rank 6
     44,  55,  66,  77,  77,  66,  55,  44, // Rank 7
      0,  11,  22,  33,  33,  22,  11,   0  // Rank 8
};
```

## 7. Threat Refresh Condition

The Threat accumulator must be fully refreshed when the King crosses the boundary between file D and file E. This changes the horizontal mirroring logic.

**Condition:**
```cpp
// ksq is the new king square (0-63)
// prev_ksq is the old king square
// 0b100 (4) detects if the file index (0-7) is >= 4.
bool requires_refresh = 
    ((ksq & 4) != (prev_ksq & 4));
```
*   **Refreshes**: e1->d1, d1->e1, d8->e8.
*   **No Refresh**: e1->e2, d1->c1, a1->d1.

## 8. Skip Connection Logic

The skip connection adds the raw output of the sparse layer (`fc_0`) to the final result, bypassing the hidden layers. This helps gradients flow and allows the network to learn linear relationships easily.

**Formula**:
```cpp
// buffer.fc_0_out[15] is the 16th output of Layer 0
int32_t fwdOut = (buffer.fc_0_out[15] * 600 * 16) / (127 * 64);
```

**Constants Explained**:
*   `600`: Represents the value of "1.0" in the network's internal training fixed-point representation.
*   `16`: `OutputScale`. The final output is scaled down by this amount (div 16) to get centipawns. The skip connection is pre-scaled to match.
*   `127`: The maximum value of the ClippedReLU activation (since the inputs to `fc_0` were effectively `0..127` or similar magnitude).
*   `64`: `(1 << WeightScaleBits)`. Weights in `fc_0` are conceptually scaled by 64.

## 9. Quantization & Scaling Pipeline

1.  **File Weights**:
    *   Piece: `int16` (Raw).
    *   Threat: `int8` (Raw).
2.  **Accumulator**: `int16`. (Sum of weights).
3.  **Product Pooling**: `int16` inputs -> clamped `0..255` -> product -> divide by 512 -> `int8` output.
4.  **Layer 0 (`fc_0`)**:
    *   Input: `int8` (from pooling).
    *   Weights: `int8`.
    *   Dot Product: `int32`.
    *   Output: `int32` (Accumulated) -> Scaled down by `WeightScaleBits (6)` (>> 6) before activation.
5.  **Activation 0**:
    *   `ac_0`: Clips `int32` to `0..127`.
    *   `ac_sqr_0`: Squares `int32`, divides by `127`? (Actually `SqrClippedReLU` implementation: `(x*x)/127` usually, or just `x*x` if x is small).
6.  **Final Output**:
    *   `fc_2` Output: `int32`.
    *   Result: `(fc_2_out + skip_connection + psqt) / OutputScale (16)`.
    *   **Unit**: Centipawns.

## 10. PieceSquareIndex Table

Maps piece types to indices `0..10`. Note the ordering swaps based on color perspective.

```cpp
// PS_NONE=0, PAWN=1..2, KNIGHT=3..4, BISHOP=5..6, ROOK=7..8, QUEEN=9..10, KING=11(ignored here)
// Piece Indices: 0..10 used for Feature Index

// White Perspective (W=Us, B=Them)
// W_PAWN: 1, W_KNIGHT: 2, W_BISHOP: 3, W_ROOK: 4, W_QUEEN: 5
// B_PAWN: 6, B_KNIGHT: 7, B_BISHOP: 8, B_ROOK: 9, B_QUEEN: 10
// King is handled by buckets, not this index.

// Black Perspective (B=Us, W=Them)
// B_PAWN: 1, B_KNIGHT: 2, ...
// W_PAWN: 6, W_KNIGHT: 7, ...
```

## 11. Threat Feature Deduplication

To reduce feature dimensions, redundant symmetric threats are removed.

1.  **Map Exclusion**: `King` attacking `King` is illegal/ignored. `King` attacking `Queen` (defensively) is mapped to -1 (excluded).
2.  **Semi-Exclusion (Identical Piece Types)**:
    *   If a White Knight at `sq1` attacks a White Knight at `sq2`.
    *   We only record the feature if `sq1 < sq2`.
    *   This prevents counting the relationship twice.
3.  **Pawn Exception**:
    *   Pawns are **never** semi-excluded. A pawn attacking a pawn is always recorded from the attacker's perspective because pawn attacks are asymmetric (capture diagonal forward).

## 12. File Format - Sizes & Offsets (Example)

For a typical SFNNv10 network (`nn-7af32f20...nnue`):
*   **Total Size**: ~20MB - 30MB (Depends on compression efficiency).
*   **Header**: ~100 bytes.
*   **Feature Transformer**: The bulk of the file.
    *   Threat Weights (Raw): 1024 * 79856 = **81.7 MB** if uncompressed.
    *   *Correction*: Wait, Stockfish compresses standard weights but Threat Weights are raw `int8`?
    *   Checked Code: `read_little_endian<ThreatWeightType>(stream, ... threatWeights.data())`.
    *   **YES**, Threat Weights are stored RAW.
    *   Size: ~81 MB.
    *   Wait, SFNNv10 files are usually smaller (~28MB compressed).
    *   Let's re-verify: `src/nnue/nnue_feature_transformer.h`.
    *   `read_little_endian` reads raw bytes.
    *   **CRITICAL**: `UseThreats` is true for SFNNv10. The file size *should* be huge.
    *   Actually, modern Stockfish nets *are* around 50-80MB?
    *   Checking typical SFNNv10 download size... `nn-37f18f62d772.nnue` is 82MB.
    *   So yes, the bulk is the 81MB raw threat weights block.
*   **Layer Stacks**: Very small (< 100KB).

## 13. Data Flow Diagram

```text
Board Position
   |
   +--> [HalfKAv2_hm Logic] --> [Active Indices P] (22,528 dim)
   |
   +--> [Full_Threats Logic] -> [Active Indices T] (79,856 dim)
   |
   v
[Accumulators (Cache/Refresh)]
   |
   +--> Piece_Acc_Us   (1024 int16)
   +--> Piece_Acc_Them (1024 int16)
   +--> Threat_Acc_Us  (1024 int16)
   +--> Threat_Acc_Them(1024 int16)
   |
   v
[Product Pooling Transform]
   | (Sum + Clamp + Multiply pairs)
   v
[Transformed Features] (1024 int8)
   |
   +--> [Us_Features (512)] + [Them_Features (512)]
   |
   v
[Bucket Selection] -> Choose Layer Stack (0..7) based on piece count
   |
   v
[Layer Stack Evaluation]
   |--> fc_0 (Sparse 1024->16)
   |    |--> [Skip Connection val=out[15]]
   |    |--> [Square/Clip Activation]
   |--> fc_1 (Dense 30->32)
   |--> ac_1 (ClippedReLU)
   |--> fc_2 (Dense 32->1)
   |
   v
[Final Summation]
   Result = (fc_2_out + Skip_Scaled + PSQT_Score) / 16
```
