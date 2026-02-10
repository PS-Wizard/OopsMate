# Stockfish NNUE Deep Dive Analysis

## 1. Executive Summary

This document provides a comprehensive analysis of the Stockfish 17.1 NNUE (Efficiently Updatable Neural Network) implementation, focusing on the **HalfKAv2_hm** architecture. The goal is to provide sufficient detail to implement a standalone Rust library (`nnue-probe-rs`) capable of loading official Stockfish networks and performing inference.

The architecture consists of a **Feature Transformer** (HalfKAv2_hm) which maps a chess position to a 3072-dimensional embedding, followed by a stack of **8 small Feed-Forward Networks** (Layer Stacks). The specific network used for a given position is determined by the total number of pieces on the board (bucketing).

## 2. File Format Specification

Stockfish NNUE files (`.nnue`) are binary files containing a header followed by serialized network parameters. Integers are stored in **Little-Endian** format. Compression is applied to weights using **LEB128**.

### 2.1. Header

| Field | Type | Description | Value (Stockfish 17.1) |
| :--- | :--- | :--- | :--- |
| `Version` | `uint32` | File format version | `0x7AF32F20` |
| `Hash` | `uint32` | Architecture hash | `0x34C8113E` (Derived from layers) |
| `Size` | `uint32` | Length of description string | Variable |
| `Description` | `char[]` | UTF-8 description string | e.g., "halfKAv2_hm" |

### 2.2. Parameter Serialization (LEB128)

Weights and biases are stored using **Signed LEB128** compression. A magic string "COMPRESSED_LEB128" precedes each bulk read.

**Structure of a bulk LEB128 read:**
1.  **Magic String**: 17 bytes: "COMPRESSED_LEB128".
2.  **Byte Count**: `uint32` - Total number of bytes in the compressed stream.
3.  **Compressed Data**: `uint8[]` - The LEB128 stream.

### 2.3. Data Layout

The file is structured as follows:

1.  **Header**
2.  **Feature Transformer Parameters**:
    *   Biases: `int16` x 3072 (LEB128)
    *   Weights: `int16` x (3072 * 22528) (LEB128)
    *   PSQT Weights: `int32` x (22528 * 8) (LEB128)
3.  **Layer Stacks** (Repeated 8 times, for buckets 0..7):
    *   **Layer 1 (fc_0)**:
        *   Biases: `int32` x 16
        *   Weights: `int8` x (16 * 3072)
    *   **Layer 2 (fc_1)**:
        *   Biases: `int32` x 32
        *   Weights: `int8` x (32 * 32) (Wait: Input is 30, padded to 32 usually)
    *   **Layer 3 (fc_2)**:
        *   Biases: `int32` x 1
        *   Weights: `int8` x (1 * 32)

**Note on Weight Ordering:**
The file stores weights in linear order (Row-Major: `[OutputDim][InputDim]`). However, Stockfish loads them into a scrambled memory layout for SIMD efficiency (`get_weight_index_scrambled`). For a Rust MVP, loading them linearly is recommended unless implementing the exact SIMD kernels.

---

## 3. Feature Engineering: HalfKAv2_hm

The **HalfKAv2_hm** feature set maps the board state to a sparse vector index.

### 3.1. Dimensions
*   **Square Count (`SQUARE_NB`)**: 64
*   **Piece Types (`PS_NB`)**: 11 (No King, see below) * 64 = 704? No.
    *   Stockfish defines `PS_NB = 11 * 64` in `half_ka_v2_hm.h`.
    *   Indices: 
        *   `PS_W_PAWN` = 0
        *   `PS_B_PAWN` = 64
        *   ...
        *   `PS_KING` = 10 * 64 (Used for king bucketing, not feature index?)
*   **Input Dimensions**: `SQUARE_NB * PS_NB / 2` = 64 * 704 / 2 = **22,528**.

### 3.2. Index Calculation
The feature index is calculated relative to the friendly King's position.

$$
\text{Index} = \text{PieceSquare}(\text{Piece}, \text{Square}) + \text{KingBucket}(\text{KingSquare}) + (\text{Square} \oplus \text{Orient}(\text{KingSquare}))
$$

Where:
*   `Orient(KingSq)`: 0 if King is on file A-D, otherwise flips indices to mirror the board horizontally (files E-H).
*   `PieceSquare`: Base offset for the piece type.
*   `KingBucket`: Base offset for the King's position (Kings are bucketed to reduce dimensions).

### 3.3. Feature Transformer (Accumulator)
*   **Accumulator Size**: 3072 (`int16`).
*   **Operation**: The transformer maintains a running sum of weight columns corresponding to active pieces.
*   **Refresh**: On full update, clear accumulator to `Biases`, then add `Weights[Index]` for every piece.
*   **Incremental Update**: Add `Weights[AddedIndex]` / Subtract `Weights[RemovedIndex]`.

**Important Scaling:**
Upon loading, Feature Transformer weights and biases are **multiplied by 2**. This is to support the specific fixed-point arithmetic in the forward pass.

---

## 4. Network Architecture & Forward Pass

The network is a "Mixture of Experts" model.

### 4.1. Bucketing (Model Selection)
The active sub-network is selected based on the number of pieces:
```cpp
bucket = (count_all_pieces() - 1) / 4; // Clamped 0..7
```
Each bucket has its own set of weights for Layers 1, 2, and 3.

### 4.2. Activation: Feature Transform -> Output
The accumulator output (3072 `int16`s) is transformed into the input for the first layer (3072 `int8`s).
*   **Split**: The 3072 values are treated as two halves of 1536.
*   **Operation**:
    ```rust
    for i in 0..1536 {
        let a = clamp(acc[i], 0, 254);
        let b = clamp(acc[i + 1536], 0, 254);
        output[i] = (a * b) / 512; // Result is 0..127 (fits in int8)
    }
    // Repeat for the other perspective (White/Black)
    ```
*   **Result**: 3072 `int8` values (1536 for White, 1536 for Black).

### 4.3. Layer 1: Affine -> SqrClippedReLU + ClippedReLU
*   **Input**: 3072 `int8`.
*   **Weights**: 16 x 3072 `int8`.
*   **Biases**: 16 `int32`.
*   **Output Calculation**: Matrix multiplication. `out = (W * input) + bias`.
*   **Activation**:
    *   Outputs 0..14 (First 15): Apply **SqrClippedReLU**.
        $$ y = \min(127, (x^2) / 2^{2 \times 6 + 7}) $$
        (Effectively $x^2 / 4096 / 128$? No, `(x*x) >> 19`).
    *   Outputs 0..14 (First 15): Apply **ClippedReLU** (Standard clamp).
        $$ y = \min(127, x / 2^6) $$
    *   **Concatenation**: The input to Layer 2 is the concatenation of these two results (30 values).

### 4.4. Layer 2: Affine -> ClippedReLU
*   **Input**: 30 `int8` (padded to 32 for SIMD).
*   **Weights**: 32 x 30 `int8`.
*   **Biases**: 32 `int32`.
*   **Activation**: ClippedReLU (clamp 0..127, shift by 6).

### 4.5. Layer 3: Affine -> Output
*   **Input**: 32 `int8`.
*   **Weights**: 1 x 32 `int8`.
*   **Biases**: 1 `int32`.
*   **Output**: Single scalar.

### 4.6. Final Scaling
The network produces two values: the PSQT score and the Neural Network score.
1.  **PSQT Score**: Derived from the `psqtAccumulation` in the Feature Transformer.
    $$ \text{PSQT} = (\text{PSQT\_Acc}[Us] - \text{PSQT\_Acc}[Them]) / 2 $$
2.  **Network Score**:
    $$ \text{NN} = \text{Layer3\_Out} + \text{Layer1\_Out}[15] \times \frac{600 \times 16}{127 \times 64} $$

**Total Evaluation**:
$$ \text{Total} = \frac{\text{PSQT}}{16} + \frac{\text{NN}}{16} $$
(Integer division, effectively `(PSQT + NN) / 16`).

---

## 5. Data Structure Reference

### Accumulator
```rust
struct Accumulator {
    accumulation: [[i16; 3072]; 2], // [Color][FeatureDim]
    psqt_accumulation: [[i32; 8]; 2], // [Color][Bucket]
    computed: [bool; 2],
}
```

### FeatureTransformer
```rust
struct FeatureTransformer {
    biases: Vec<i16>,     // 3072
    weights: Vec<i16>,    // 3072 * 22528
    psqt_weights: Vec<i32> // 22528 * 8
}
```

### Layer
```rust
struct Layer<const IN: usize, const OUT: usize> {
    biases: [i32; OUT],
    weights: Vec<i8>, // OUT * IN (Linear)
}
```

---

## 6. Implementation Roadmap (Rust)

### Phase 1: File Parsing
1.  Implement `LEB128` reader.
2.  Implement `read_header` and validate magic/version.
3.  Implement `read_parameters` to load Biases/Weights into vectors.
    *   **Crucial**: Multiply FT parameters by 2 immediately.

### Phase 2: Position & Features
1.  Port `Square`, `Piece`, `Color` enums.
2.  Implement `HalfKAv2_hm::make_index`.
3.  Implement `refresh_accumulator`:
    *   Iterate all pieces.
    *   Calculate indices.
    *   Sum weight columns.

### Phase 3: Forward Pass (Scalar)
1.  Implement `transform` (Accumulator -> Input Layer).
2.  Implement `AffineTransform::propagate` (Matrix Mul).
3.  Implement `ClippedReLU` and `SqrClippedReLU`.
4.  Implement the Bucketing logic to select the correct sub-network.
5.  Combine layers to produce final score.

### Phase 4: Optimization
1.  Implement Incremental Updates (DirtyPiece tracking).
2.  Add SIMD (AVX2) for the Accumulator update (this is the bottleneck).
