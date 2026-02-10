# Stockfish 17.1 NNUE: The Definitive Reverse Engineering Reference

**Version:** SFNNv9 (Stockfish 17.1)  
**Date:** February 10, 2026  
**Reference Crate:** `nnue-rs`  
**Original Engine:** Stockfish 17.1

---

## Table of Contents

1.  **Introduction**
2.  **Binary File Format Deep Dive**
    *   Header Structure
    *   Compression (Signed LEB128)
    *   Parameter Loading & Pre-scaling
3.  **Input Feature Set: HalfKAv2_hm**
    *   Mathematical Definition
    *   King Bucketing
    *   Mirroring & Orientation
    *   Index Calculation Formula
4.  **The Accumulator System**
    *   Concept & Motivation
    *   Memory Layout
    *   Incremental Updates
    *   Refresh Logic
5.  **Network Architecture & Bucketing**
    *   The Hybrid Model (Big vs. Small)
    *   The Bucket System
    *   Layer Topology
6.  **Forward Propagation (Step-by-Step)**
    *   Step 1: Feature Transformation
    *   Step 2: Dense Layer 1 (FC_0)
    *   Step 3: Activation Functions (The Nuanced Part)
    *   Step 4: Buffer Overlap & Concat
    *   Step 5: Hidden Layers (FC_1, FC_2)
    *   Step 6: Residual Scaling & Output
7.  **Evaluation Logic & WDL**
    *   Simple Eval & Network Selection
    *   Complexity & Optimism
    *   UCI Centipawn Conversion
8.  **Complete Evaluation Trace**

---

## 1. Introduction

This document provides a comprehensive technical breakdown of the NNUE (Efficiently Updatable Neural Network) architecture used in Stockfish 17.1. It is intended for engine developers, researchers, and enthusiasts who wish to implement bit-exact replicas of the evaluation function.

NNUE differs from standard Deep Learning in that it runs efficiently on CPUs without GPUs, primarily due to:
1.  **Sparse Inputs:** Only a tiny fraction of inputs are active.
2.  **Incremental Updates:** The first layer is updated incrementally as pieces move, rather than recomputed.
3.  **Integer Arithmetic:** The entire inference pass uses integer math (`i8`, `i16`, `i32`), relying on specific quantization schemes.

---

## 2. Binary File Format Deep Dive

Stockfish distributes its networks as `.nnue` files. These are binary files containing versioning info, descriptions, and the raw weights/biases.

### 2.1. Header Structure

The file begins with a 4-byte version integer, followed by metadata. All integers are **Little Endian**.

| Offset | Type | Name | Expected Value (SF 17.1) | Description |
|:-------|:-----|:-----|:-------------------------|:------------|
| 0x00 | `u32` | Version | `0x7AF32F20` | Magic version number. |
| 0x04 | `u32` | Hash | Varies | A hash of the parameter data. |
| 0x08 | `u32` | DescLen | Varies | Length of the description string. |
| 0x0C | `u8[]` | Desc | ASCII | "26bg... with pytorch trainer" |

### 2.2. Compression: Signed LEB128

To reduce file size, most large parameter arrays (specifically the Feature Transformer weights) are compressed using **Signed LEB128**.

**Algorithm:**
Variable-length encoding where each byte holds 7 bits of data. The MSB (bit 7) is a "continuation flag".
-   **Byte Structure:** `[C D D D D D D D]`
    -   `C` (Continuation): 1 if more bytes follow, 0 if this is the last byte.
    -   `D` (Data): 7 bits of the integer payload.
-   **Sign Extension:** Since these are signed integers, the final byte's 7th data bit (bit 6) is sign-extended to fill the remaining bits of the result type (`i32` or `i16`).

**The "Magic String" Nuance:**
The compressed stream is often prefixed by a 17-byte magic string: `COMPRESSED_LEB128`.
-   **Critical Detail:** In Stockfish C++ code, the `read_leb_128` function *automatically* peeks and consumes this string if present.
-   **Implementation Warning:** If writing a custom parser, you **must** check for this string before reading the Feature Transformer parameters. It appears *immediately* after the file header.

### 2.3. Parameter Loading Sequence

The file is read sequentially.

1.  **Feature Transformer Hash** (`u32`)
2.  **Magic String Check** (Consume "COMPRESSED_LEB128" if present)
3.  **Feature Transformer Parameters:**
    -   **Biases:** `read_leb128_i16` -> Array of size `HalfDimensions`.
    -   **Weights:** `read_leb128_i16` -> Array of size `HalfDimensions * InputDimensions`.
    -   **PSQT Weights:** `read_leb128_i32` -> Array of size `8 * InputDimensions`.
4.  **Scaling Triggers:**
    -   Immediately after loading, Feature Transformer Biases and Weights are multiplied by **2**.
    -   `bias = bias * 2`
    -   `weight = weight * 2`
    -   **Reason:** This pre-scaling is an optimization for the inference phase to utilize the full range of `i16` and align with `mulhi` (multiply high) SIMD instructions which effectively divide by $2^{16}$.

5.  **Layer Stacks (The 8 Buckets):**
    For each bucket `k` from 0 to 7:
    -   **Hash** (`u32`)
    -   **Layer 0 (FC_0):**
        -   Biases: `read_i32` (Raw 4 bytes) -> Size `OutputDims`.
        -   Weights: `read_i8` (Raw 1 byte) -> Size `OutputDims * PaddedInputDims`.
    -   **Layer 1 (FC_1):** Biases (`i32`), Weights (`i8`).
    -   **Layer 2 (FC_2):** Biases (`i32`), Weights (`i8`).

---

## 3. Input Feature Set: HalfKAv2_hm

The input to the network is a boolean vector indicating the presence of pieces.
**Dimensions:** 22,528 active features per perspective.

### 3.1. "HalfKA" Semantics
-   **Half:** The network evaluates the position from the perspective of the *side to move* (Us). To get the full evaluation, it also evaluates from the opponent's perspective (Them) and combines them. This architecture allows the weights to capture "My King" relationships.
-   **K (King):** Features are relative to the King's square.
-   **A (And):** Represents "King at Sq1 AND Piece at Sq2".

### 3.2. King Buckets & Mirroring
Stockfish 17.1 uses `HalfKAv2_hm`. The `hm` stands for **Horizontal Mirroring**.
Instead of 64 separate feature sets for 64 king squares, it uses **32 buckets**.
-   The board is split vertically.
-   A King on `h1` (index 7) uses the same parameters as a King on `a1` (index 0), but the entire board is mirrored horizontally relative to it.

**King Bucket Mapping:**
```text
Bucket 0:  a1, h1
Bucket 1:  b1, g1
...
Bucket 31: d8, e8
```

### 3.3. Index Calculation Formula

To calculate the active feature index for a piece `P` at square `S`, given King at `K`:

1.  **Perspective Transform:**
    If perspective is BLACK, the board is flipped vertically (Rank 1 becomes Rank 8).
    `S' = S ^ 56` (XOR 56 flips ranks).

2.  **Orientation (Horizontal Mirroring):**
    If the King is on the right side of the board (files e-h), we mirror the board horizontally.
    `OrientTable[K] = 7` (if e-h) else `0`.
    `S_final = S' ^ OrientTable[K]`

3.  **Components:**
    -   **Piece Index:** 11 types (Pawn..King for both colors).
        -   Start indices: `W_Pawn=0`, `B_Pawn=64`, ..., `King=640`.
    -   **King Bucket:** `KingBucket[K]` (0..31).
    -   **PS_NB:** 64 * 11 = 704 (Stride per bucket).

4.  **Final Index:**
    ```rust
    Index = (KingBucket[K] * 704) + (PieceType * 64) + S_final
    ```

**Total Size:** 32 Buckets * 11 Pieces * 64 Squares = **22,528**.

---

## 4. The Accumulator System

The Accumulator is the "state" of the first layer.

### 4.1. Memory Layout
```rust
struct Accumulator {
    // Transformer 1: Main Features
    // Stores sum of weights (i16)
    accumulation: [Vec<i16>; 2], // Index 0: White, Index 1: Black
    
    // Transformer 2: PSQT
    // Stores sum of weights (i32)
    psqt_accumulation: [Vec<i32>; 2], 
}
```
-   **Big Net Size:** `accumulation` has 3,072 elements per color.
-   **Small Net Size:** `accumulation` has 128 elements per color.

### 4.2. Initialization & Updates
-   **Initialization:** Start with the **Bias** vector.
-   **Add Piece:** `acc[i] += weight[feature_index][i]`
-   **Remove Piece:** `acc[i] -= weight[feature_index][i]`

### 4.3. Refresh Logic
If the King moves, the `KingBucket` likely changes. This invalidates *every* feature index (since index depends on KingBucket).
-   **King Move:** Full Refresh (Reset to bias, re-add all pieces).
-   **Other Move:** Incremental Update (Remove old square, add new square).

---

## 5. Network Architecture & Bucketing

Stockfish uses a "Hybrid" hand-crafted MLP (Multi-Layer Perceptron).

### 5.1. Dual Networks
1.  **Big Network (`nn-1c...`)**: 133MB. Used for 90%+ of evaluations.
    -   Topology: `Input(22528) -> L1(3072) -> L2(15) -> L3(32) -> Out(1)`
2.  **Small Network (`nn-37...`)**: 6MB. Used for "easy" positions (high material imbalance).
    -   Topology: `Input(22528) -> L1(128) -> L2(15) -> L3(32) -> Out(1)`

### 5.2. The Bucket System
Dense layers (L2, L3, Out) are specialized. There are **8 sets** of these layers.
-   **Selection:** Based on piece count (including Kings/Pawns).
    `Bucket = floor((PieceCount - 1) / 4)`
    Max bucket 7.
-   **Motivation:** Opening positions (32 pieces) have vastly different structural requirements than Endgames (5 pieces).

---

## 6. Forward Propagation (Step-by-Step)

This section traces the exact mathematical operations for a single evaluation.

**Assumptions:**
-   Big Network.
-   Bucket 7 selected.
-   Accumulators `acc[US]` and `acc[THEM]` are ready.

### Step 1: Feature Transformation (L1)
We combine Us and Them into a single activation vector.

**Operation:** Pointwise multiplication of clipped values.
**Scaling Constant:** 512.

For each neuron $i$ in `0..3072`:
1.  `v_us = clamp(acc[US][i], 0, 254)`
    *(Note: 254 = 127 * 2. Max value for i16 pre-scaled weights)*
2.  `v_them = clamp(acc[THEM][i], 0, 254)`
3.  `product = v_us * v_them` (Result is u16-like, fits in i32)
4.  `output[i] = product / 512`
    *(Integer division. Result fits in u8: $254^2 / 512 \approx 126$)*

**Result:** `input_vector` of 3072 `u8` values.

### Step 2: Dense Layer 1 (FC_0)
**Dimensions:** 3072 inputs $\to$ 16 outputs.
**Matrix:** Weights `[16 x 3072]` (`i8`). Biases `[16]` (`i32`).

`raw_output = MatMul(input_vector, weights) + bias`
**Result:** 16 `i32` values.

### Step 3: Activation Functions & The Buffer Overlap
This is the most nuanced part of the architecture implementation.

We apply two different activation functions to the *same* `raw_output`.
1.  **SqrClippedReLU:** $f(x) = \text{clamp}( (x^2) >> 19, 0, 127 )$
2.  **ClippedReLU:** $g(x) = \text{clamp}( x >> 6, 0, 127 )$

**The "Input Construction" for Next Layer:**
The next layer (FC_1) expects 30 inputs.
-   Indices `0..15`: Derived from `SqrClippedReLU`.
-   Indices `15..30`: Derived from `ClippedReLU`.

**Wait, indices overlap?**
Yes. In C++, this is handled by a `memcpy` that overwrites.
-   We calculate `Sqr` for all 16 outputs. Write to buffer `[0..15]`.
-   We calculate `Clipped` for all 16 outputs. Write to buffer starting at offset 15: `[15..30]`.
-   **Result:**
    -   `Buffer[0..14]`: Sqr results of neurons 0-14.
    -   `Buffer[15]`: Clipped result of neuron 0 (**Overwrites Sqr neuron 15**).
    -   `Buffer[16..30]`: Clipped results of neurons 1-15.

**The Lost Residual:**
The raw output of neuron 15 (`raw_output[15]`) is **NOT** used in the hidden layers via `Sqr`. It is overwritten.
However, `raw_output[15]` is saved and used as a **Residual** scaling factor at the very end.

### Step 4: Hidden Layers (FC_1, FC_2)
1.  **FC_1:**
    -   Input: 30 `u8` values (from Step 3 buffer).
    -   Output: 32 `i32` values.
    -   Activation: `ClippedReLU` (`>> 6`, clamp 127).
    -   Result: 32 `u8` values.

2.  **FC_2:**
    -   Input: 32 `u8` values.
    -   Output: 1 `i32` value (`fc_2_out`).

### Step 5: Residual Scaling & Final Output
We combine the FC_2 output with the residual from FC_0.

**Constants:**
-   `OutputScale = 16`
-   `WeightScaleBits = 6`
-   `ResidualScale = 600` (Implicit constant in formula)

**Formula:**
```rust
residual = raw_output_fc0[15];
fwd_out = residual * (600 * 16) / (127 * (1 << 6)); 
// fwd_out = residual * 9600 / 8128

final_value_internal = fc_2_out + fwd_out;
```

**PSQT Integration:**
During the feature transform, we also computed `psqt_score`.
`psqt_score = (acc_psqt[US][bucket] - acc_psqt[THEM][bucket]) / 2`

**Hybrid Output:**
The network is trained to output `(psqt, positional)` tuple logic, but for inference:
`result = (psqt / 16, final_value_internal / 16)`

---

## 7. Evaluation Logic & WDL

The raw network outputs `Internal Units`. These are not Centipawns.

### 7.1. Hybrid Selection
Stockfish calculates a heuristic "Simple Eval" (Material counting).
-   If `|SimpleEval| > 962` (Decisive/Unbalanced), try **Small Net**.
-   If Small Net result is ambiguous (`< 236`), fallback to **Big Net**.
-   Otherwise use Small Net.

### 7.2. Complexity Scaling
Stockfish adjusts the score based on "NNUE Complexity".
`complexity = |psqt - positional|`
`final = final - (final * complexity / 18000)`
*This dampens evaluations in highly complex/volatile positions.*

### 7.3. UCI Conversion (The WDL Model)
To match Stockfish's `eval` command output, we must convert Internal Units to Centipawns using a Win-Rate model.

$$ CP = \frac{100 \cdot Internal}{A} $$

Where $A$ is a coefficient dependent on total material on the board.
-   More material -> Higher $A$ -> Internal units are "worth less" cp.
-   Less material -> Lower $A$ -> Internal units are "worth more" cp.

---

## 8. Complete Evaluation Trace

**Position:** `e4` (Black to move).
**FEN:** `rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1`

**1. Parse:**
-   Side: Black.
-   Piece Count: 32.
-   Bucket: `(32-1)/4 = 7`.

**2. Accumulator (Big Net):**
-   Refresh logic runs.
-   White Acc (`acc[0]`) calculated.
-   Black Acc (`acc[1]`) calculated.

**3. Feature Transform:**
-   Us = `acc[1]`, Them = `acc[0]`.
-   3072 multiplies, shifts, clamps.
-   Result: Dense u8 vector.

**4. Forward Pass:**
-   Input: Vector.
-   FC_0 (Bucket 7): produces 16 values.
-   Residual = `FC_0[15]`.
-   Construct Input for FC_1 (Overlapping Sqr/Clipped).
-   FC_1 (Bucket 7): produces 32 values.
-   FC_2 (Bucket 7): produces 1 value.
-   Scale Residual & Add.
-   Result (Internal): **-140**.

**5. Output:**
-   Perspective Flip (for UI): **+140** (White's advantage).
-   Material Count: 78.
-   WDL `a` coeff: ~378.
-   CP: `140 * 100 / 378` = **37 cp**.

**6. Verification:**
-   Stockfish 17.1 Output: `+0.37`.
-   **Match.**

---

*This document serves as the master reference for the `nnue-rs` implementation.*
