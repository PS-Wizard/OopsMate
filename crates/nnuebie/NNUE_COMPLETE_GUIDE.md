# The Complete NNUE Guide: Understanding Efficiently Updatable Neural Networks for Chess

## Table of Contents

1. [Introduction](#introduction)
2. [What Problem Does NNUE Solve?](#what-problem-does-nnue-solve)
3. [High-Level Architecture Overview](#high-level-architecture-overview)
4. [Feature Representation: The Input Layer](#feature-representation-the-input-layer)
5. [The Feature Transformer: Layer 1](#the-feature-transformer-layer-1)
6. [The Accumulator: NNUE's Secret Weapon](#the-accumulator-nnues-secret-weapon)
7. [From Accumulator to Dense Layers](#from-accumulator-to-dense-layers)
8. [The Dense Layers: FC_0, FC_1, FC_2](#the-dense-layers-fc_0-fc_1-fc_2)
9. [Activation Functions Explained](#activation-functions-explained)
10. [Residual Connections](#residual-connections)
11. [PSQT (Piece-Square Tables)](#psqt-piece-square-tables)
12. [King Buckets](#king-buckets)
13. [Layer Stacks](#layer-stacks)
14. [Horizontal Mirroring](#horizontal-mirroring)
15. [Complete Forward Pass Walkthrough](#complete-forward-pass-walkthrough)
16. [Quantization and Integer Math](#quantization-and-integer-math)
17. [File Format and Compression](#file-format-and-compression)
18. [Common Confusions Clarified](#common-confusions-clarified)
19. [Performance Optimizations](#performance-optimizations)
20. [Code Examples](#code-examples)
21. [Glossary](#glossary)

---

## Introduction

**NNUE** stands for **Efficiently Updatable Neural Networks**. It's a revolutionary technique that combines the pattern-recognition power of neural networks with the blazing speed required for chess engines.

### Key Innovation

Traditional neural networks are too slow for chess (need to evaluate millions of positions per second). NNUE solves this by:
1. Using sparse binary inputs (22,528 features, but only ~30 are active)
2. Caching intermediate results in an "accumulator"
3. Only updating what changed when pieces move
4. Using fast integer arithmetic instead of floating-point

**Result:** ~2,000,000x speedup compared to recalculating from scratch!

### Brief History

- **2018**: Yu Nasu develops NNUE for Shogi (Japanese chess)
- **2020**: Stockfish adopts NNUE, gaining ~100 Elo strength
- **Today**: NNUE is the standard for top chess engines

---

## What Problem Does NNUE Solve?

### Traditional Chess Evaluation

Chess engines traditionally use handcrafted evaluation functions:

```c
int evaluate(Position pos) {
    int score = 0;
    
    // Material counting
    score += count_pawns(WHITE) * 100;
    score += count_knights(WHITE) * 320;
    // ... etc
    
    // Positional bonuses
    if (pawn_on_7th_rank) score += 50;
    if (king_castled) score += 30;
    if (bishop_pair) score += 25;
    if (rook_on_open_file) score += 40;
    // ... thousands of such rules
    
    return score;
}
```

**Problems:**
- Hard to tune (thousands of parameters)
- Misses complex patterns
- Requires expert chess knowledge
- Linear combinations can't capture non-linear relationships

### Why Not Just Use Deep Learning?

Standard neural networks CAN learn these patterns, but they're too slow:

```python
# Standard NN approach
def evaluate(board):
    # Convert board to 8x8x12 tensor (64 squares × 12 piece types)
    input = board_to_tensor(board)
    
    # Forward pass through network
    x = layer1(input)   # 768 → 512
    x = layer2(x)       # 512 → 256
    x = layer3(x)       # 256 → 128
    x = layer4(x)       # 128 → 1
    
    return x
```

**Problem:** This takes ~100,000 operations per evaluation. A chess engine evaluates 1-10 million positions per second. That's 100 billion to 1 trillion operations per second - way too slow!

### NNUE's Solution

NNUE uses a clever trick:

1. **Sparse Input**: Most inputs are zero (pieces not on squares)
2. **Incremental Updates**: Cache the result and only update what changed
3. **Integer Math**: No floating-point, everything is quantized

Instead of 100,000 operations, NNUE does ~3,000 operations per evaluation - a **30x speedup** while maintaining neural network quality!

---

## High-Level Architecture Overview

Here's the complete NNUE pipeline:

```
┌─────────────────────────────────────────────────────────────────┐
│  CHESS POSITION                                                 │
│  32 pieces maximum on 64 squares                                │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  FEATURE INDEXING (kings aren't counted 32 piece -> 30 piece)   │
│  Convert board → 22,528 binary features                         │
│  ~30 features are "1" (active), rest are "0" (inactive)         │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  FEATURE TRANSFORMER (Layer 1)                                   │
│  Each active feature contributes:                               │
│  - 1,536 weights → ACCUMULATOR                                  │
│  - 8 PSQT values → PSQT_ACCUMULATOR (separate track!)          │
└─────────────────────────────────────────────────────────────────┘
          ↓                                    ↓
   [ACCUMULATOR]                        [PSQT_ACCUMULATOR]
   3,072 values                         16 values (8×2)
   (2 × 1,536)                          (2 perspectives)
          ↓                                    |
   ┌──────────────────────┐                   |
   │ TRANSFORM STEP       │                   |
   │ 3,072 → 1,536        │                   |
   │ (multiply pairs)     │                   |
   └──────────────────────┘                   |
          ↓                                    |
   ┌──────────────────────┐                   |
   │ FC_0 (Layer 2)       │                   |
   │ 1,536 → 16           │                   |
   └──────────────────────┘                   |
          ↓                                    |
   ┌──────────────────────┐                   |
   │ DUAL ACTIVATION      │                   |
   │ ClippedReLU: 15 vals │                   |
   │ SqrClippedReLU: 15   │                   |
   │ Concatenate: 30      │                   |
   │ Residual: 1 (saved)  │                   |
   └──────────────────────┘                   |
          ↓                                    |
   ┌──────────────────────┐                   |
   │ FC_1 (Layer 3)       │                   |
   │ 30 → 32              │                   |
   └──────────────────────┘                   |
          ↓                                    |
   ┌──────────────────────┐                   |
   │ ClippedReLU          │                   |
   └──────────────────────┘                   |
          ↓                                    |
   ┌──────────────────────┐                   |
   │ FC_2 (Layer 4)       │                   |
   │ 32 → 1               │                   |
   └──────────────────────┘                   |
          ↓                                    |
   [Positional Score]                         |
          ↓                                    |
   ┌──────────────────────┐                   |
   │ Add Residual         │                   |
   │ (saved from FC_0)    │                   |
   └──────────────────────┘                   |
          |                                    |
          +------------------------------------+
                         ↓
                  ┌──────────────┐
                  │  Add PSQT    │
                  │  (choose     │
                  │  bucket 0-7) │
                  └──────────────┘
                         ↓
                  ┌──────────────┐
                  │ FINAL SCORE  │
                  │ (centipawns) │
                  └──────────────┘
```

### Two Parallel Tracks

Notice there are **two separate tracks** that merge at the end:

1. **Neural Network Track**: Accumulator → Transform → FC_0 → FC_1 → FC_2 → Positional Score
2. **PSQT Track**: PSQT_Accumulator → Choose Bucket → PSQT Score

Both are updated incrementally and combined at the final step.

---

## Feature Representation: The Input Layer

### What is a Feature?

A **feature** represents a specific piece on a specific square relative to the king's position.

Each feature is binary: **1** if the piece is there, **0** if not.

### Feature Components

A feature uniquely identifies:
1. **Piece type and color**: White pawn, black knight, etc. (11 types total)
2. **Square**: Where the piece is (64 squares)
3. **King bucket**: Which group the king position falls into (32 buckets per side)

### Total Features

```
22,528 features = 11 piece types × 64 squares × 32 king buckets
```

### Example Features

```
Feature 0:    "White pawn on a2, white king in bucket 0"
Feature 1:    "White pawn on a3, white king in bucket 0"
Feature 2:    "White pawn on a4, white king in bucket 0"
...
Feature 528:  "Black knight on f6, white king in bucket 5"
...
Feature 22527: "Black king on h8, black king in bucket 31"
```

### Sparsity: The Key Insight

In any chess position:
- **Maximum 32 pieces** on the board
- **Typically 20-30 pieces** in the middlegame
- Out of **22,528 features**, only ~**30 are active** (value = 1)
- The remaining **22,498 are inactive** (value = 0)

This sparsity is what makes NNUE efficient!

### Feature Calculation Example

```
Position: Starting position
White king: e1 (bucket 5)
Black king: e8 (bucket 27)

Active features (white's perspective):
- White pawn on a2, king bucket 5 → Feature 64
- White pawn on b2, king bucket 5 → Feature 65
- White pawn on c2, king bucket 5 → Feature 66
... (all 16 white pieces)
- Black pawn on a7, king bucket 5 → Feature 10234
- Black pawn on b7, king bucket 5 → Feature 10235
... (all 16 black pieces)

Total active features: 32
Inactive features: 22,496
```

---

## The Feature Transformer: Layer 1

### What Does It Do?

The Feature Transformer is the first and largest layer. It converts the sparse 22,528 binary features into a dense 3,072-value vector.

### Weight Structure

Each of the 22,528 features has:
- **1,536 weights** for the neural network (accumulator)
- **8 PSQT values** for traditional piece-square evaluation

```
Total weights in Feature Transformer:
- Neural Network: 22,528 × 1,536 = 34,603,008 weights (~69 MB as i16)
- PSQT: 22,528 × 8 = 180,224 values (~721 KB as i32)
```

### How It Works: Element-Wise Addition

The Feature Transformer doesn't do matrix multiplication in the traditional sense. Instead, it does **element-wise addition** of weight vectors.

Think of it like this:
- Start with a bias vector (1,536 values)
- For each active feature, add its 1,536 weights to the accumulator
- That's it!

### Mathematical Formula

```
For perspective P (white or black):

accumulator[P] = bias + Σ(weights[feature_i]) for all active features i

Where:
- accumulator[P] is a vector of 1,536 values
- bias is a vector of 1,536 values (the network's default state)
- weights[feature_i] is a vector of 1,536 values for feature i
```

### Concrete Example (Simplified)

Let's use small numbers to illustrate:
- 5 possible features (instead of 22,528)
- 3 output values per perspective (instead of 1,536)

**Weights:**
```
Feature 0 weights: [10, -5, 20]
Feature 1 weights: [ 5, 10, -15]
Feature 2 weights: [-5, 15, 10]
Feature 3 weights: [ 8, -3, 5]
Feature 4 weights: [20, -5, 10]
```

**Biases:**
```
[100, 100, 100]
```

**Current position:** Features 1 and 3 are active (pieces exist)

**Calculation:**
```
Start with bias:        [100, 100, 100]

Feature 1 is active:
  Add its weights:      + [  5,  10, -15]
  Running total:        = [105, 110,  85]

Feature 3 is active:
  Add its weights:      + [  8,  -3,   5]
  Final accumulator:    = [113, 107,  90]
```

That's the accumulator for one perspective!

### Two Perspectives

NNUE evaluates from **both** white's and black's point of view:

```
White's perspective:
  accumulator[WHITE] = bias + Σ(weights[white_features])
  = [v₀, v₁, v₂, ..., v₁₅₃₅]

Black's perspective:
  accumulator[BLACK] = bias + Σ(weights[black_features])
  = [v₀, v₁, v₂, ..., v₁₅₃₅]

Total output: 3,072 values (1,536 × 2)
```

### Why Two Perspectives?

Chess isn't symmetric - what's good for white might be bad for black! By evaluating from both perspectives, the network can learn:
- "This pawn structure is great for white"
- "But black has compensation with piece activity"

The network combines both views in later layers.

---

## The Accumulator: NNUE's Secret Weapon

### What Is an Accumulator?

The **accumulator** is a data structure that **caches** the output of the Feature Transformer so we don't have to recalculate it from scratch every time.

### Structure

```rust
struct Accumulator {
    // Main neural network values: [perspective][value]
    accumulation: [[i16; 1536]; 2],
    
    // PSQT values: [perspective][bucket]
    psqt_accumulation: [[i32; 8]; 2],
    
    // Is this perspective up to date?
    computed: [bool; 2],
}
```

### Why Is It Fast?

**Without accumulator (recalculate each time):**
```
For each position:
  For each of 22,528 features:
    If feature is active (piece on square):
      Add its 1,536 weights to accumulator

Operations: 22,528 features × 1,536 weights = 34,603,008 operations
```

Even with sparsity (only ~30 active features):
```
Operations: 30 active features × 1,536 weights = 46,080 operations
```

**With accumulator (incremental update):**
```
When piece moves from square A to square B:
  Subtract feature_A weights (1,536 operations)
  Add feature_B weights (1,536 operations)
  
Total: 3,072 operations
```

**Speedup: 46,080 / 3,072 = ~15x faster!**

And if we had to recalculate from scratch every time: **34,603,008 / 3,072 = ~11,263x faster!**

### Incremental Update Algorithm

Here's how the accumulator updates when a piece moves:

```rust
// Piece moves from square A to square B

// 1. Calculate old and new feature indices
let old_feature = calculate_feature_index(piece, square_A, king_bucket);
let new_feature = calculate_feature_index(piece, square_B, king_bucket);

// 2. Subtract old feature's weights
for i in 0..1536 {
    accumulator[perspective][i] -= weights[old_feature][i];
}

// 3. Add new feature's weights
for i in 0..1536 {
    accumulator[perspective][i] += weights[new_feature][i];
}

// 4. Do the same for PSQT (separate track)
for bucket in 0..8 {
    psqt_accumulation[perspective][bucket] -= psqt_weights[old_feature][bucket];
    psqt_accumulation[perspective][bucket] += psqt_weights[new_feature][bucket];
}
```

### CRITICAL: Bias Is Not Re-Added

This is a common confusion! The bias is added **once** during initialization or full refresh, and then stays in the accumulator.

**Correct:**
```rust
// Initial refresh
accumulator = biases.clone();
for each piece on board {
    accumulator += weights[piece_feature];
}

// Incremental update (NO BIAS!)
accumulator -= weights[old_feature];
accumulator += weights[new_feature];
```

**Incorrect:**
```rust
// WRONG - don't add bias on incremental update!
accumulator -= weights[old_feature];
accumulator += weights[new_feature];
accumulator += biases;  // ❌ NO!
```

Why? Because the bias is already in there from the initial calculation!

### Full Refresh vs Incremental Update

**Full Refresh** (from scratch):
```rust
fn refresh_accumulator(perspective: Perspective, pieces: &[Piece]) {
    // Start with biases
    accumulator[perspective] = biases.clone();
    
    // Add weights for each piece
    for piece in pieces {
        let feature = calculate_feature_index(piece, king_bucket);
        for i in 0..1536 {
            accumulator[perspective][i] += weights[feature][i];
        }
    }
}
```

**Incremental Update** (piece moved):
```rust
fn update_accumulator(perspective: Perspective, removed: &[Feature], added: &[Feature]) {
    // Subtract removed features
    for feature in removed {
        for i in 0..1536 {
            accumulator[perspective][i] -= weights[feature][i];
        }
    }
    
    // Add new features
    for feature in added {
        for i in 0..1536 {
            accumulator[perspective][i] += weights[feature][i];
        }
    }
    
    // NO BIAS - it's already in there!
}
```

### When Do We Need Full Refresh?

**King moves!**

When the king moves, ALL feature indices change (because features depend on king position via king buckets). In this case, incremental update won't work - we need a full refresh.

```rust
if king_moved {
    refresh_accumulator(perspective);  // Full recalculation
} else {
    update_accumulator(perspective, removed_features, added_features);  // Incremental
}
```

This is still fast because:
1. Kings don't move every turn
2. Full refresh with ~30 pieces = 30 × 1,536 = 46,080 ops (still way less than 34 million!)

### The `computed` Flag

The `computed` field tracks whether each perspective is up-to-date:

```rust
struct Accumulator {
    accumulation: [[i16; 1536]; 2],
    psqt_accumulation: [[i32; 8]; 2],
    computed: [bool; 2],  // ← This!
}
```

**Purpose:** Sometimes we only need one perspective (e.g., in certain search optimizations). The `computed` flag lets us skip updating the other perspective.

```rust
// Only update white's perspective
if !accumulator.computed[WHITE] {
    refresh_accumulator(WHITE);
    accumulator.computed[WHITE] = true;
}

// Black's perspective can wait
```

### SIMD Optimization

The accumulator updates use **SIMD** (Single Instruction, Multiple Data) to process multiple values at once:

```rust
// Process 16 values at once using AVX2
unsafe {
    for i in (0..1536).step_by(16) {
        // Load 16 accumulator values
        let acc_vec = _mm256_loadu_si256(accumulator.as_ptr().add(i) as *const __m256i);
        
        // Load 16 weight values
        let weight_vec = _mm256_loadu_si256(weights.as_ptr().add(i) as *const __m256i);
        
        // Add them (16 additions in one instruction!)
        let result = _mm256_add_epi16(acc_vec, weight_vec);
        
        // Store back
        _mm256_storeu_si256(accumulator.as_mut_ptr().add(i) as *mut __m256i, result);
    }
}
```

This gives another **16x speedup** on top of the incremental update speedup!

**Total speedup:** 15x (incremental) × 16x (SIMD) = **240x faster** than naive calculation!

---

## From Accumulator to Dense Layers

After the Feature Transformer, we have 3,072 values in the accumulator (1,536 per perspective). But the next layer (FC_0) takes 1,536 inputs, not 3,072!

What happens in between?

### The Transform Step

This is a crucial step that's often glossed over in documentation. It converts 3,072 values → 1,536 values.

**Algorithm:**
```
1. Split each perspective's 1,536 values into two halves (768 + 768)
2. Multiply corresponding values from each half together
3. Divide by 512 (scaling factor)
4. Apply ClippedReLU (clamp to 0-127)
5. Combine results from both perspectives
```

**Pseudocode:**
```rust
let white = accumulator[WHITE];  // 1,536 values
let black = accumulator[BLACK];  // 1,536 values

// Split each into halves
let white_half1 = white[0..768];
let white_half2 = white[768..1536];
let black_half1 = black[0..768];
let black_half2 = black[768..1536];

// Multiply pairs and scale
let mut output = [0i8; 1536];
for i in 0..768 {
    // White contribution
    let val = (white_half1[i] as i32 * white_half2[i] as i32) / 512;
    output[i] = clipped_relu(val);
    
    // Black contribution
    let val = (black_half1[i] as i32 * black_half2[i] as i32) / 512;
    output[i + 768] = clipped_relu(val);
}

// output now has 1,536 values ready for FC_0
```

### Why Multiply Pairs?

This multiplication creates **non-linear interactions** between features. It allows the network to detect:
- "Feature X AND Feature Y together mean something special"
- "This combination of pieces creates a tactical opportunity"

Without this, the network could only learn linear combinations.

### ClippedReLU During Transform

```rust
fn clipped_relu(x: i32) -> i8 {
    if x < 0 {
        0
    } else if x > 127 {
        127
    } else {
        x as i8
    }
}
```

This keeps values in the range [0, 127], which:
1. Prevents overflow in later calculations
2. Introduces non-linearity (negative values become zero)
3. Uses 8-bit integers (faster and more cache-friendly)

---

## The Dense Layers: FC_0, FC_1, FC_2

After the transform step, we have 1,536 values ready to go through the "traditional" neural network layers.

### Layer Dimensions

| Layer | Input Size | Output Size | Purpose |
|-------|------------|-------------|---------|
| **FC_0** (Layer 2) | 1,536 | 16 | First dense layer; outputs 15 active + 1 residual |
| **FC_1** (Layer 3) | 30 | 32 | Second dense layer; takes concatenated activations |
| **FC_2** (Layer 4) | 32 | 1 | Final layer; produces positional evaluation |

### FC_0: The First Dense Layer

**Input:** 1,536 values (from transform step)
**Output:** 16 values

This is a standard fully-connected layer:

```rust
fn fc_0_forward(input: &[i8; 1536], weights: &[[i8; 1536]; 16], biases: &[i32; 16]) -> [i32; 16] {
    let mut output = [0i32; 16];
    
    for j in 0..16 {
        let mut sum = biases[j];
        for i in 0..1536 {
            sum += (input[i] as i32) * (weights[j][i] as i32);
        }
        output[j] = sum;
    }
    
    output
}
```

**Weight matrix size:** 1,536 × 16 = 24,576 weights (as i8 values)

### Why 16 Outputs?

The 16 outputs serve different purposes:
- **Outputs 0-14** (15 values): Active outputs that go through dual activation
- **Output 15** (1 value): Residual value (saved for later, added at the very end)

### Dual Activation: ClippedReLU and SqrClippedReLU

After FC_0 outputs 16 values, we run **both** activation functions on them:

```rust
let fc0_output = fc_0_forward(input, weights, biases);  // [i32; 16]

// Apply ClippedReLU to all 16 values
let mut relu_output = [0i8; 15];
for i in 0..15 {
    relu_output[i] = clipped_relu(fc0_output[i]);
}

// Apply SqrClippedReLU to all 16 values
let mut sqr_output = [0i8; 15];
for i in 0..15 {
    sqr_output[i] = sqr_clipped_relu(fc0_output[i]);
}

// Save the residual (16th value)
let residual = fc0_output[15];

// Concatenate: 15 + 15 = 30 values
let fc1_input = concat(sqr_output, relu_output);  // [i8; 30]
```

### Activation Functions

**ClippedReLU:**
```rust
fn clipped_relu(x: i32) -> i8 {
    let scaled = x >> 6;  // Divide by 64 (right shift by 6)
    if scaled < 0 {
        0
    } else if scaled > 127 {
        127
    } else {
        scaled as i8
    }
}
```

**SqrClippedReLU:**
```rust
fn sqr_clipped_relu(x: i32) -> i8 {
    let clamped = x.max(0).min(127 << 6);  // Clamp input
    let squared = (clamped as i64 * clamped as i64) >> 19;  // Square and scale
    squared.min(127) as i8
}
```

### Why Two Activations?

- **ClippedReLU**: Standard activation, good for linear relationships
- **SqrClippedReLU**: Emphasizes stronger signals, good for detecting piece coordination

By using both, the network gets **two different views** of the same data, which helps it learn complex patterns.

### Where Does 30 Come From?

This is a super common confusion! The 30 comes from **concatenating** the dual activations:

```
FC_0 outputs: 16 values

ClippedReLU on first 15:     [r₀, r₁, r₂, ..., r₁₄]  → 15 values
SqrClippedReLU on first 15:  [s₀, s₁, s₂, ..., s₁₄]  → 15 values

Concatenate: [s₀, s₁, ..., s₁₄, r₀, r₁, ..., r₁₄]  → 30 values

The 16th value is saved as the residual!
```

### FC_1: The Second Dense Layer

**Input:** 30 values (from concatenated activations)
**Output:** 32 values

```rust
fn fc_1_forward(input: &[i8; 30], weights: &[[i8; 30]; 32], biases: &[i32; 32]) -> [i32; 32] {
    let mut output = [0i32; 32];
    
    for j in 0..32 {
        let mut sum = biases[j];
        for i in 0..30 {
            sum += (input[i] as i32) * (weights[j][i] as i32);
        }
        output[j] = sum;
    }
    
    output
}
```

**Weight matrix size:** 30 × 32 = 960 weights (as i8 values)

After FC_1, we apply ClippedReLU again:
```rust
let fc1_output = fc_1_forward(input, weights, biases);
let fc1_activated = apply_clipped_relu(&fc1_output);  // [i8; 32]
```

### FC_2: The Final Layer

**Input:** 32 values (from FC_1 after activation)
**Output:** 1 value (the positional evaluation!)

```rust
fn fc_2_forward(input: &[i8; 32], weights: &[i8; 32], biases: &[i32; 1]) -> i32 {
    let mut sum = biases[0];
    for i in 0..32 {
        sum += (input[i] as i32) * (weights[i] as i32);
    }
    sum
}
```

**Weight matrix size:** 32 × 1 = 32 weights (as i8 values)

This single value represents the **positional evaluation** before adding the residual and PSQT.

---

## Activation Functions Explained

Activation functions introduce **non-linearity** into neural networks. Without them, multiple layers would collapse into a single linear transformation.

### ClippedReLU (Rectified Linear Unit)

**Formula:**
```
f(x) = max(0, min(127, x >> 6))
```

**What it does:**
1. Divides by 64 (right shift by 6 bits)
2. Clamps negative values to 0
3. Clamps values above 127 to 127

**Purpose:**
- Zero out negative values (dead neurons)
- Keep values in i8 range (0-127)
- Simple and fast

**Graph:**
```
  127 ─────────────────
      │               ╱
      │             ╱
      │           ╱
      │         ╱
      │       ╱
    0 ─────╱──────────
      │   0         x
```

### SqrClippedReLU (Squared Clipped ReLU)

**Formula:**
```
f(x) = min(127, ((max(0, min(127 << 6, x)))² >> 19))
```

**What it does:**
1. Clamps input to reasonable range
2. Squares the value (emphasizes larger values)
3. Scales down by dividing by 2^19
4. Clamps to 127

**Purpose:**
- Emphasizes strong signals (squaring amplifies larger values)
- Helps detect piece coordination (when multiple features are active)
- Non-linear transformation

**Why both?**

Using both ClippedReLU and SqrClippedReLU gives the network:
- **Linear features** (from ClippedReLU)
- **Quadratic features** (from SqrClippedReLU)

This is more expressive than using just one activation!

### Quantization: Why >> 6?

The `>> 6` (divide by 64) is part of **quantization** - converting floating-point to fixed-point integers.

During training:
- Values are floating-point: 0.0 to 1.0
- Network learns with gradients and backpropagation

For inference (evaluation):
- Values are integers with implicit scale factor
- `>> 6` means "scale factor is 64" (2^6)
- A stored value of 640 represents 640/64 = 10.0 in floating-point

This allows fast integer math while maintaining precision!

---

## Residual Connections

A **residual connection** (or skip connection) adds the output of an earlier layer directly to a later layer's output.

### Why Residuals?

1. **Prevents vanishing gradients** during training
2. **Preserves information** that might be lost in deep networks
3. **Allows network to learn "corrections"** to a base value

### How It Works in NNUE

Remember FC_0 outputs 16 values? The 16th value (index 15) is the **residual**:

```rust
let fc0_output = fc_0_forward(input, weights, biases);  // [i32; 16]

// First 15 go through normal path
let activations = dual_activation(&fc0_output[0..15]);  // → FC_1 → FC_2

// 16th is saved
let residual = fc0_output[15];
```

At the very end, after FC_2, we add the residual back:

```rust
let fc2_output = fc_2_forward(fc1_output, weights, biases);  // Single value

// Scale the residual
let scaled_residual = (residual * 600 * OUTPUT_SCALE) / (127 * (1 << WEIGHT_SCALE_BITS));
// = (residual * 9600) / 8128
// ≈ residual * 1.181

// Add it to FC_2 output
let final_positional = fc2_output + scaled_residual;
```

### Why Scale the Residual?

The residual comes from FC_0 (earlier in the network), so it's in a different scale than FC_2's output. The scaling factor (~1.181) normalizes it.

### Intuition

Think of it like:
- **Main path** (FC_0 → FC_1 → FC_2): "Here's my detailed analysis of the position"
- **Residual**: "But don't forget this basic fact I noticed early on"

The final evaluation is: detailed analysis + basic fact.

Example:
- Main path evaluates: "+50 centipawns (detailed tactical evaluation)"
- Residual says: "+20 centipawns (simple material imbalance)"
- Final: 50 + 20 = +70 centipawns

---

## PSQT (Piece-Square Tables)

**PSQT** stands for **Piece-Square Table**. It's a traditional chess evaluation technique that assigns values based on piece type and square.

### Traditional PSQT

Classic chess engines use tables like this:

```c
// Value of white knight on each square
int knight_psqt[64] = {
    -50, -40, -30, -30, -30, -30, -40, -50,  // Rank 1 (bad)
    -40, -20,   0,   5,   5,   0, -20, -40,  // Rank 2
    -30,   5,  10,  15,  15,  10,   5, -30,  // Rank 3
    -30,   0,  15,  20,  20,  15,   0, -30,  // Rank 4
    -30,   5,  15,  20,  20,  15,   5, -30,  // Rank 5
    -30,   0,  10,  15,  15,  10,   0, -30,  // Rank 6
    -40, -20,   0,   0,   0,   0, -20, -40,  // Rank 7
    -50, -40, -30, -30, -30, -30, -40, -50,  // Rank 8 (bad)
};
```

Knights are worth more on central squares (d4, e4, d5, e5) and less on the edges.

### NNUE's PSQT: 8 Buckets

NNUE doesn't use a single PSQT. It learns **8 different PSQTs** for different game phases!

**Why 8?**

The value of squares changes throughout the game:
- **Opening** (29-32 pieces): Knights want to be on f3/c3, close to king
- **Middlegame** (17-24 pieces): Knights want central squares
- **Endgame** (8-12 pieces): Knights less important, king wants to be active
- **Late endgame** (1-4 pieces): Passed pawns are everything

### PSQT Buckets

```
Bucket 0: 1-4 pieces   (very late endgame)
Bucket 1: 5-8 pieces   (late endgame)
Bucket 2: 9-12 pieces  (endgame)
Bucket 3: 13-16 pieces (transitional)
Bucket 4: 17-20 pieces (early middlegame)
Bucket 5: 21-24 pieces (middlegame)
Bucket 6: 25-28 pieces (late opening)
Bucket 7: 29-32 pieces (opening)
```

### How PSQT is Stored

Each feature has **8 PSQT values** (one per bucket):

```rust
struct FeatureWeights {
    // For neural network
    nn_weights: [i16; 1536],
    
    // For PSQT (8 buckets)
    psqt_weights: [i32; 8],
}
```

### PSQT Accumulation (Parallel to Neural Network)

When the accumulator is updated, PSQT values are updated **at the same time**:

```rust
// When a feature becomes active
fn add_feature(feature_idx: usize, perspective: Perspective) {
    // Update neural network accumulator
    for i in 0..1536 {
        accumulator[perspective][i] += nn_weights[feature_idx][i];
    }
    
    // Update PSQT accumulator (separate!)
    for bucket in 0..8 {
        psqt_accumulation[perspective][bucket] += psqt_weights[feature_idx][bucket];
    }
}
```

### Two Perspectives for PSQT

Just like the neural network accumulator, PSQT has two perspectives:

```rust
psqt_accumulation: [[i32; 8]; 2]
                    ^^^      ^^^
                    8        2 perspectives
                    buckets  (WHITE, BLACK)
```

Each side accumulates its own PSQT values because the same piece position looks different relative to different kings!

### PSQT Evaluation

At evaluation time, we choose the appropriate bucket and calculate the PSQT score:

```rust
fn compute_psqt(side_to_move: Color, piece_count: usize) -> i32 {
    // Choose bucket based on piece count
    let bucket = min((piece_count - 1) / 4, 7);
    
    // Get accumulated PSQT values for this bucket
    let our_psqt = psqt_accumulation[side_to_move][bucket];
    let their_psqt = psqt_accumulation[opposite(side_to_move)][bucket];
    
    // Return the difference (our advantage)
    (our_psqt - their_psqt) / 2
}
```

**Example:**
```
Position has 20 pieces
Bucket = (20 - 1) / 4 = 4 (capped at 7)

psqt_accumulation[WHITE][4] = 350
psqt_accumulation[BLACK][4] = 280

If white to move:
  PSQT score = (350 - 280) / 2 = 35 centipawns
  
If black to move:
  PSQT score = (280 - 350) / 2 = -35 centipawns
```

### PSQT Never Goes Through Dense Layers!

This is critical to understand: PSQT is a **separate track** that runs parallel to the neural network and only combines at the very end.

```
Feature Transformer
    ↓           ↓
Accumulator   PSQT_Accumulator
    ↓           ↓
Transform       |
    ↓           |
FC_0 → FC_1 → FC_2
    ↓           |
Positional      |
    ↓           ↓
    +───────────+ (Add here!)
         ↓
    Final Score
```

---

## King Buckets

**King buckets** are a way to reduce the input space by grouping similar king positions together.

### The Problem Without King Buckets

If we tracked every (piece, piece_square, king_square) combination:
```
64 king squares × 64 piece squares × 11 piece types = 45,056 features
```

For two perspectives (white and black):
```
45,056 × 2 = 90,112 features!
```

This is too many features - the weight matrix would be huge and training would be slow.

### The Solution: Group Similar King Positions

Instead of 64 unique king positions, group them into **32 buckets**:

```
Bucket 0:  King on a1, b1  (queenside castled)
Bucket 1:  King on c1, d1  (queenside, not castled)
Bucket 2:  King on e1      (center, starting square)
Bucket 3:  King on f1      (center-right)
Bucket 4:  King on g1, h1  (kingside castled)
...
Bucket 31: King on g8, h8  (black kingside castled)
```

**New feature count:**
```
32 king buckets × 64 piece squares × 11 piece types = 22,528 features
```

This is half the size! And because of horizontal mirroring (see next section), it's effectively even smaller.

### Why Group King Positions?

Kings in similar positions have similar strategic implications:
- King on f1 vs g1: Both are castled kingside, evaluation logic is mostly the same
- King on e4 vs f4: Both are centralized, danger patterns are similar

We don't need separate weights for every possible king square!

### King Bucket Mapping

The exact king bucket mapping varies by implementation, but a typical scheme:

```
White's perspective:
  a1,b1 → bucket 0
  c1,d1 → bucket 1
  e1    → bucket 2
  f1    → bucket 3
  g1,h1 → bucket 4
  a2,b2 → bucket 5
  ...
  
Black's perspective (mirrored):
  a8,b8 → bucket 0
  c8,d8 → bucket 1
  ...
```

### Feature Index Calculation

```rust
fn make_index(
    perspective: Color,
    piece_square: Square,
    piece: Piece,
    king_square: Square
) -> usize {
    let king_bucket = KING_BUCKETS[perspective][king_square];
    let piece_offset = piece as usize * 64;  // 11 piece types
    let square_offset = piece_square as usize;
    
    king_bucket * 704 + piece_offset + square_offset
    //            ^^^
    //            64 squares × 11 piece types = 704
}
```

### King Buckets vs Layer Stacks

**Common confusion!** These are completely different:

| Concept | Number | Purpose | Where Used |
|---------|--------|---------|------------|
| **King Buckets** | 32 | Group similar king positions | Feature Transformer (Layer 1) |
| **PSQT Buckets** | 8 | Different game phases (piece count) | PSQT evaluation |
| **Layer Stacks** | 8 | Different networks for game phases | FC_0, FC_1, FC_2 (Layers 2-4) |

They all serve different purposes!

---

## Layer Stacks

**Layer stacks** are multiple copies of the dense layers (FC_0, FC_1, FC_2) with different weights for different game phases.

### What Are Layer Stacks?

NNUE has **8 complete sets** of FC_0, FC_1, and FC_2 weights:

```
Layer Stack 0: FC_0 weights + FC_1 weights + FC_2 weights (for 1-4 pieces)
Layer Stack 1: FC_0 weights + FC_1 weights + FC_2 weights (for 5-8 pieces)
Layer Stack 2: FC_0 weights + FC_1 weights + FC_2 weights (for 9-12 pieces)
...
Layer Stack 7: FC_0 weights + FC_1 weights + FC_2 weights (for 29-32 pieces)
```

### Why Multiple Layer Stacks?

Different game phases require different evaluation logic:

**Opening (29-32 pieces):**
- Piece development is critical
- King safety is paramount
- Center control matters

**Middlegame (17-24 pieces):**
- Tactics and combinations
- Pawn structure becomes important
- Weak squares and outposts

**Endgame (5-12 pieces):**
- King activity is crucial
- Passed pawns are everything
- Opposition and zugzwang matter

A single network can't learn all these phase-specific patterns well. By having separate networks for each phase, NNUE can specialize!

### How Layer Stacks Are Chosen

Based on piece count:

```rust
fn choose_layer_stack(piece_count: usize) -> usize {
    let bucket = (piece_count - 1) / 4;
    min(bucket, 7)  // Cap at 7
}
```

**Examples:**
```
3 pieces  → bucket = (3-1)/4 = 0 → Layer Stack 0
8 pieces  → bucket = (8-1)/4 = 1 → Layer Stack 1
15 pieces → bucket = (15-1)/4 = 3 → Layer Stack 3
32 pieces → bucket = (32-1)/4 = 7 → Layer Stack 7
```

### Layer Stack Weights

Each stack has the same architecture but different weights:

```rust
struct LayerStack {
    fc_0_weights: [[i8; 1536]; 16],  // 1536 × 16
    fc_0_biases: [i32; 16],
    
    fc_1_weights: [[i8; 30]; 32],    // 30 × 32
    fc_1_biases: [i32; 32],
    
    fc_2_weights: [i8; 32],          // 32 × 1
    fc_2_biases: [i32; 1],
}

// 8 complete sets
let layer_stacks: [LayerStack; 8];
```

### Same Feature Transformer!

The Feature Transformer (Layer 1) is **shared** across all layer stacks:

```
                    Feature Transformer
                    (shared, always used)
                            ↓
                      Accumulator
                            ↓
                        Transform
                            ↓
                   Choose Layer Stack
                    (based on piece count)
                            ↓
         ┌──────────┬───────┴────────┬──────────┐
         ↓          ↓                ↓          ↓
    Stack 0    Stack 1   ...    Stack 6    Stack 7
      FC_0       FC_0             FC_0       FC_0
      FC_1       FC_1             FC_1       FC_1
      FC_2       FC_2             FC_2       FC_2
```

### Layer Stacks ≠ PSQT Buckets

Both use piece count, and both have 8 options, but they're different:

| Concept | What It Is | Where It's Used |
|---------|------------|-----------------|
| **PSQT Buckets** | Different piece-square table values | PSQT accumulator (parallel to NN) |
| **Layer Stacks** | Different neural network weights | FC_0, FC_1, FC_2 (dense layers) |

They both group game phases the same way (by piece count), but they're separate systems!

---

## Horizontal Mirroring

Chess is **left-right symmetric**. A position with white king on a1 and pieces on the queenside is the mirror image of white king on h1 with pieces on the kingside.

### The Insight

Instead of learning weights for both left and right sides of the board, we can:
1. Learn weights for one side (say, the right side)
2. Mirror positions that are on the left side
3. Use the same weights!

This **halves** the amount of data needed for training!

### How It Works

When calculating feature indices, we use an **orientation table**:

```rust
// ORIENT_TBL maps squares to their horizontally mirrored version
const ORIENT_TBL: [[usize; 64]; 2] = [
    // White's perspective
    [
        // a-file → h-file, b → g, c → f, d → e, etc.
        7, 6, 5, 4, 3, 2, 1, 0,   // Rank 1
        7, 6, 5, 4, 3, 2, 1, 0,   // Rank 2
        ...
    ],
    // Black's perspective (different mirroring)
    [...]
];

fn make_index(perspective: Color, square: Square, piece: Piece, king_square: Square) -> usize {
    // Mirror the square if it's on the left side
    let oriented_square = ORIENT_TBL[perspective][square];
    let oriented_king = ORIENT_TBL[perspective][king_square];
    
    // Calculate feature index using mirrored positions
    let king_bucket = KING_BUCKETS[perspective][oriented_king];
    king_bucket * 704 + piece * 64 + oriented_square
}
```

### Example

```
Original position:
  King on a1, Knight on c3

After mirroring:
  King on h1, Knight on f3

Feature index is calculated using (h1, f3) instead of (a1, c3)
```

Since the network only learned weights for kings on the right side, we mirror left-side positions to fit that learned pattern!

### Combined with King Buckets

Horizontal mirroring works together with king buckets:

```
Instead of:
  64 king squares × 64 piece squares = 4,096 combinations

We have:
  32 king buckets × 64 piece squares = 2,048 combinations
  
With mirroring:
  Effectively ~1,024 unique patterns!
```

This massive reduction in parameters makes training faster and prevents overfitting!

---

## Complete Forward Pass Walkthrough

Let's walk through the entire NNUE evaluation from start to finish with a concrete example.

### Starting Position

```
rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1

32 pieces total
White king on e1 (bucket 5)
Black king on e8 (bucket 27)
```

### Step 1: Feature Indexing

For each piece, calculate its feature index:

```
White pawn on a2, white king bucket 5:
  feature_idx = 5 * 704 + (WHITE_PAWN) * 64 + a2
              = 3520 + 0 * 64 + 8
              = 3528

White knight on b1, white king bucket 5:
  feature_idx = 5 * 704 + (WHITE_KNIGHT) * 64 + b1
              = 3520 + 64 + 1
              = 3585

... (for all 32 pieces, from both white and black perspectives)
```

Total active features: 32 (per perspective)

### Step 2: Feature Transformer (Accumulator)

**White's perspective:**
```rust
accumulator[WHITE] = biases.clone();  // Start with [b₀, b₁, ..., b₁₅₃₅]

// Add weights for each of white's 32 active features
for feature in white_features {
    for i in 0..1536 {
        accumulator[WHITE][i] += weights[feature][i];
    }
}

// Result: [a₀, a₁, a₂, ..., a₁₅₃₅]
```

**Black's perspective:** (same process with black's features)
```rust
accumulator[BLACK] = biases.clone();

for feature in black_features {
    for i in 0..1536 {
        accumulator[BLACK][i] += weights[feature][i];
    }
}

// Result: [a₀, a₁, a₂, ..., a₁₅₃₅]
```

**PSQT (parallel):**
```rust
// For each feature, also update PSQT
for feature in all_features {
    for bucket in 0..8 {
        psqt_accumulation[perspective][bucket] += psqt_weights[feature][bucket];
    }
}
```

### Step 3: Transform (3,072 → 1,536)

```rust
let white_vals = accumulator[WHITE];  // 1,536 values
let black_vals = accumulator[BLACK];  // 1,536 values

let mut transformed = [0i8; 1536];

for i in 0..768 {
    // White contribution
    let w1 = white_vals[i] as i32;
    let w2 = white_vals[i + 768] as i32;
    transformed[i] = clipped_relu((w1 * w2) / 512);
    
    // Black contribution
    let b1 = black_vals[i] as i32;
    let b2 = black_vals[i + 768] as i32;
    transformed[i + 768] = clipped_relu((b1 * b2) / 512);
}

// transformed now has 1,536 values
```

### Step 4: Choose Layer Stack

```rust
let piece_count = 32;  // Starting position
let stack_idx = min((piece_count - 1) / 4, 7);
// stack_idx = min(31 / 4, 7) = min(7, 7) = 7

// Use Layer Stack 7 (opening weights)
let stack = &layer_stacks[7];
```

### Step 5: FC_0 (1,536 → 16)

```rust
let mut fc0_output = [0i32; 16];

for j in 0..16 {
    let mut sum = stack.fc_0_biases[j];
    for i in 0..1536 {
        sum += (transformed[i] as i32) * (stack.fc_0_weights[j][i] as i32);
    }
    fc0_output[j] = sum;
}

// fc0_output: [v₀, v₁, ..., v₁₄, v₁₅]
//                                ^^^
//                                residual (saved!)
```

### Step 6: Dual Activation (16 → 30)

```rust
let residual = fc0_output[15];  // Save for later

let mut relu_out = [0i8; 15];
let mut sqr_out = [0i8; 15];

for i in 0..15 {
    relu_out[i] = clipped_relu(fc0_output[i]);
    sqr_out[i] = sqr_clipped_relu(fc0_output[i]);
}

// Concatenate
let fc1_input = [sqr_out, relu_out].concat();  // 30 values
```

### Step 7: FC_1 (30 → 32)

```rust
let mut fc1_output = [0i32; 32];

for j in 0..32 {
    let mut sum = stack.fc_1_biases[j];
    for i in 0..30 {
        sum += (fc1_input[i] as i32) * (stack.fc_1_weights[j][i] as i32);
    }
    fc1_output[j] = sum;
}

// Apply ClippedReLU
let fc1_activated = apply_clipped_relu(&fc1_output);  // [i8; 32]
```

### Step 8: FC_2 (32 → 1)

```rust
let mut fc2_output = stack.fc_2_biases[0];

for i in 0..32 {
    fc2_output += (fc1_activated[i] as i32) * (stack.fc_2_weights[i] as i32);
}

// fc2_output is a single i32 value!
```

### Step 9: Add Residual

```rust
let scaled_residual = (residual * 600 * OUTPUT_SCALE) / (127 * 64);
// = (residual * 9600) / 8128
// ≈ residual * 1.181

let positional = fc2_output + scaled_residual;
```

### Step 10: Add PSQT

```rust
let bucket = min((piece_count - 1) / 4, 7);  // 7 for opening

let psqt = (psqt_accumulation[WHITE][bucket] 
          - psqt_accumulation[BLACK][bucket]) / 2;

let final_score = positional + psqt;
```

### Step 11: Final Adjustments

```rust
// Complexity reduction
let complexity = (psqt - positional).abs();
final_score -= final_score * complexity / 18000;

// Material scaling
let material = count_material(position);
final_score = final_score * (77777 + material) / 77777;

// Clamp to valid range
final_score = final_score.clamp(-31753, 31753);
```

### Result

```
Final evaluation: +25 centipawns
(Slight advantage for white in the starting position)
```

---

## Quantization and Integer Math

NNUE uses **integer arithmetic** instead of floating-point for speed. This is called **quantization**.

### What is Quantization?

Converting continuous floating-point values to discrete integer values:

```
Floating-point: 0.523
Quantized (scale 64): 0.523 × 64 = 33.472 ≈ 33
```

When we need the floating-point value back:
```
33 / 64 = 0.515625 ≈ 0.523 (small error)
```

### Scale Factors in NNUE

Different layers use different scale factors:

| Layer | Weight Type | Scale Factor | Shift |
|-------|-------------|--------------|-------|
| Feature Transformer weights | i16 | 64 | >> 6 |
| Feature Transformer biases | i16 | 64 | >> 6 |
| FC_0, FC_1, FC_2 weights | i8 | 64 | >> 6 |
| FC_0, FC_1, FC_2 biases | i32 | 4096 | >> 12 |
| PSQT values | i32 | 256 | >> 8 |

### Why Different Data Types?

**i16 for Feature Transformer:**
- Large accumulation (adding ~30 features × 1,536 weights)
- Needs more range to prevent overflow
- 2 bytes per weight

**i8 for Dense Layers:**
- Smaller range needed (already activated/clamped)
- 1 byte per weight (saves memory and cache)
- Faster to load and multiply

**i32 for Biases:**
- Result of many multiplications
- Needs large range for intermediate sums
- 4 bytes each

### Integer Multiplication Example

```rust
// FC_0 calculation with quantization
let mut sum: i32 = bias;  // i32

for i in 0..1536 {
    let input_val = input[i] as i32;    // i8 → i32
    let weight_val = weight[i] as i32;  // i8 → i32
    sum += input_val * weight_val;
}

// sum is now in scale 64 × 64 = 4096
// To get back to scale 64, shift right by 6:
let output = sum >> 6;
```

### Activation with Quantization

```rust
fn clipped_relu(x: i32) -> i8 {
    // x is in scale 4096 (from bias scale)
    // We want output in scale 64
    // So divide by 64: 4096 / 64 = 64 = 2^6
    let scaled = x >> 6;
    
    // Clamp to i8 range
    if scaled < 0 {
        0
    } else if scaled > 127 {
        127
    } else {
        scaled as i8
    }
}
```

### Why Integer Math is Faster

**Floating-point (slow):**
```rust
let result = (input * 0.523) + 0.231;  // FP multiply + FP add
```

**Integer (fast):**
```rust
let result = (input * 33) >> 6;  // Integer multiply + bit shift
```

Integer operations are:
- **2-4x faster** on modern CPUs
- **More cache-friendly** (smaller data types)
- **Deterministic** (no rounding errors across platforms)

---

## File Format and Compression

NNUE networks are stored in binary files with a specific format.

### File Structure

```
┌─────────────────────────────────────────────────────────────┐
│ HEADER                                                      │
│ - Version: 4 bytes (u32, little-endian)                    │
│   Expected: 0x7AF32F20 (SFNNv9 format)                     │
│ - Hash: 4 bytes (integrity check)                          │
│ - Description Length: 4 bytes (u32)                        │
│ - Description: N bytes (ASCII string)                      │
├─────────────────────────────────────────────────────────────┤
│ FEATURE TRANSFORMER                                         │
│ - Hash: 4 bytes                                            │
│ - Magic: "COMPRESSED_LEB128\0" (17 bytes)                 │
│ - Biases: LEB128-encoded i16 × 1,536                      │
│ - Weights: LEB128-encoded i16 × (22,528 × 1,536)          │
│ - PSQT Weights: LEB128-encoded i32 × (22,528 × 8)         │
├─────────────────────────────────────────────────────────────┤
│ LAYER STACK 0                                               │
│ - Hash: 4 bytes                                            │
│ - FC_0 weights: i8 × (1,536 × 16) = 24,576 bytes          │
│ - FC_0 biases: i32 × 16 = 64 bytes                        │
│ - FC_1 weights: i8 × (30 × 32) = 960 bytes                │
│ - FC_1 biases: i32 × 32 = 128 bytes                       │
│ - FC_2 weights: i8 × 32 = 32 bytes                        │
│ - FC_2 biases: i32 × 1 = 4 bytes                          │
├─────────────────────────────────────────────────────────────┤
│ LAYER STACK 1 (same structure as Stack 0)                  │
├─────────────────────────────────────────────────────────────┤
│ ... (Layer Stacks 2-7)                                      │
└─────────────────────────────────────────────────────────────┘
```

### LEB128 Compression

**LEB128** stands for **Little Endian Base 128**. It's a variable-length encoding for integers.

**Why use it?**

Feature Transformer weights are often small values (-50 to +50). Instead of storing each as 2 bytes (i16), LEB128 stores:
- Small values in 1 byte
- Medium values in 2 bytes
- Large values in 3+ bytes

This compresses the file by ~40-50%!

**How it works:**
```
Value: 127 or less → 1 byte
Value: 128-16383   → 2 bytes
Value: 16384+      → 3+ bytes
```

Each byte has 7 bits of data + 1 continuation bit:
```
Byte 1: [continue bit][7 data bits]
Byte 2: [continue bit][7 data bits]
...
```

**Example:**
```
Value: 42
Binary: 00101010
LEB128: 00101010 (1 byte, continue bit = 0)

Value: 300
Binary: 00000001 00101100
LEB128: 10101100 00000010 (2 bytes)
        ^^^^^^^^  ^^^^^^^^
        continue  last byte
        bit=1     continue=0
```

### File Size

**Uncompressed:**
```
Feature Transformer: 22,528 × 1,536 × 2 bytes = ~69 MB
PSQT: 22,528 × 8 × 4 bytes = ~721 KB
Layer Stacks (×8): ~26 KB × 8 = ~208 KB

Total: ~70 MB
```

**With LEB128 compression:**
```
Feature Transformer: ~40-45 MB (compressed)
PSQT: ~400 KB (compressed)
Layer Stacks: 208 KB (not compressed, already small)

Total: ~40-45 MB
```

### Loading a Network

```rust
fn load_network(filename: &str) -> Network {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    
    // Read header
    let version = read_u32(&mut reader)?;
    assert_eq!(version, 0x7AF32F20, "Invalid version");
    
    let hash = read_u32(&mut reader)?;
    let desc_len = read_u32(&mut reader)?;
    let description = read_string(&mut reader, desc_len)?;
    
    // Read Feature Transformer
    let ft_hash = read_u32(&mut reader)?;
    let magic = read_bytes(&mut reader, 17)?;
    assert_eq!(magic, b"COMPRESSED_LEB128\0");
    
    let biases = read_leb128_array(&mut reader, 1536)?;
    let weights = read_leb128_array(&mut reader, 22528 * 1536)?;
    let psqt_weights = read_leb128_array(&mut reader, 22528 * 8)?;
    
    // Read 8 Layer Stacks
    let mut stacks = Vec::new();
    for _ in 0..8 {
        let stack_hash = read_u32(&mut reader)?;
        let fc0_weights = read_i8_array(&mut reader, 1536 * 16)?;
        let fc0_biases = read_i32_array(&mut reader, 16)?;
        let fc1_weights = read_i8_array(&mut reader, 30 * 32)?;
        let fc1_biases = read_i32_array(&mut reader, 32)?;
        let fc2_weights = read_i8_array(&mut reader, 32)?;
        let fc2_biases = read_i32_array(&mut reader, 1)?;
        
        stacks.push(LayerStack { ... });
    }
    
    Network { feature_transformer, stacks }
}
```

---

## Common Confusions Clarified

### Q: Where does 3,072 come from?

**A:** Two perspectives × 1,536 values per perspective = 3,072

The Feature Transformer outputs 1,536 values for white's perspective and 1,536 for black's perspective.

### Q: Where does 30 come from?

**A:** FC_0 outputs 16 values. We run two activations on the first 15, giving 15 + 15 = 30.

```
FC_0: 16 outputs
  ↓
ClippedReLU on first 15 → 15 values
SqrClippedReLU on first 15 → 15 values
  ↓
Concatenate → 30 values

(The 16th is the residual, saved separately)
```

### Q: Why 1,536? Why not 1,000 or 2,048?

**A:** 1,536 = 512 × 3, which is SIMD-friendly (divisible by 16 for AVX2). It's large enough to capture complex patterns but small enough to be fast.

### Q: Are king buckets the same as layer stacks?

**A:** NO! Completely different:
- **King buckets (32)**: Group similar king positions to reduce features
- **Layer stacks (8)**: Different neural network weights for different game phases
- **PSQT buckets (8)**: Different piece-square tables for different game phases

### Q: Does the number of active features change the output size?

**A:** NO! The accumulator always outputs 3,072 values regardless of whether there are 10 pieces or 32 pieces.

More pieces = more weight additions, but the output vector size stays constant.

### Q: Is the accumulator the same as the Feature Transformer?

**A:** Related but not the same:
- **Feature Transformer**: The layer with weights and biases
- **Accumulator**: The data structure that caches the Feature Transformer's output

### Q: Why is the PSQT separate from the neural network?

**A:** PSQT provides traditional chess knowledge (piece values, square bonuses) while the neural network learns corrections and complex patterns.

They're maintained in parallel and combined only at the end.

### Q: Do we use both big and small networks?

**A:** Yes! NNUE has two networks:
- **Big network**: 1,536 dimensions, used for normal positions
- **Small network**: 128 dimensions, used when |eval| > 962 (winning/losing)

The small network is faster but less accurate, which is fine when the position is already decided.

### Q: Why 8 layer stacks and not 12 or 16?

**A:** Trade-off between:
- More stacks = better specialization per game phase
- Fewer stacks = less memory, faster to load

8 was found to be optimal through experimentation.

---

## Performance Optimizations

### SIMD (Single Instruction, Multiple Data)

Modern CPUs can process multiple values in one instruction using SIMD:

```rust
// Scalar (1 operation at a time)
for i in 0..1536 {
    accumulator[i] += weights[i];
}
// 1,536 operations

// SIMD (16 operations at a time with AVX2)
for i in (0..1536).step_by(16) {
    let acc = _mm256_loadu_si256(&accumulator[i]);
    let wgt = _mm256_loadu_si256(&weights[i]);
    let result = _mm256_add_epi16(acc, wgt);
    _mm256_storeu_si256(&mut accumulator[i], result);
}
// 1,536 / 16 = 96 operations
```

**Speedup: 16x!**

### Cache Optimization

NNUE is designed to be cache-friendly:

**Feature Transformer weights (69 MB):**
- Too large for CPU cache
- But we only access ~30 feature weights per update
- ~30 × 1,536 × 2 bytes = ~92 KB (fits in L2 cache!)

**Dense layer weights:**
- FC_0: 24 KB
- FC_1: 960 bytes
- FC_2: 32 bytes
- Total: ~25 KB (fits in L1 cache!)

This means most of the time, we're working with cached data!

### Integer Math

As discussed in Quantization section:
- Integer multiply: 1 cycle
- Floating-point multiply: 3-5 cycles
- Integer operations are 3-5x faster

### Lazy Evaluation

Only compute what's needed:

```rust
// Don't update black's perspective if we only need white's
if side_to_move == WHITE && accumulator.computed[WHITE] {
    // White's accumulator is up-to-date, skip black
    return evaluate_from_white_perspective();
}
```

### Batch Evaluation

In some search algorithms, we can batch multiple positions:

```rust
// Evaluate 4 positions at once using SIMD
fn evaluate_batch(positions: &[Position; 4]) -> [i32; 4] {
    // Use 256-bit SIMD to process 4 positions in parallel
    // Each position's values are interleaved
}
```

---

## Code Examples

### Complete Evaluation Function

```rust
pub fn evaluate(position: &Position, network: &Network) -> i32 {
    // Step 1: Update accumulator if needed
    if !accumulator.computed[WHITE] {
        refresh_accumulator(&position, WHITE, network);
    }
    if !accumulator.computed[BLACK] {
        refresh_accumulator(&position, BLACK, network);
    }
    
    // Step 2: Transform (3,072 → 1,536)
    let transformed = transform_accumulator(&accumulator);
    
    // Step 3: Choose layer stack based on piece count
    let piece_count = position.piece_count();
    let stack_idx = min((piece_count - 1) / 4, 7);
    let stack = &network.stacks[stack_idx];
    
    // Step 4: FC_0 forward pass
    let fc0_out = fc_0_forward(&transformed, stack);
    let residual = fc0_out[15];
    
    // Step 5: Dual activation
    let fc1_input = dual_activation(&fc0_out[0..15]);
    
    // Step 6: FC_1 forward pass
    let fc1_out = fc_1_forward(&fc1_input, stack);
    let fc1_activated = apply_clipped_relu(&fc1_out);
    
    // Step 7: FC_2 forward pass
    let fc2_out = fc_2_forward(&fc1_activated, stack);
    
    // Step 8: Add residual
    let scaled_residual = (residual * 600 * OUTPUT_SCALE) / (127 * 64);
    let positional = fc2_out + scaled_residual;
    
    // Step 9: Calculate PSQT
    let bucket = min((piece_count - 1) / 4, 7);
    let psqt = compute_psqt(&accumulator, bucket, position.side_to_move());
    
    // Step 10: Combine
    let mut score = positional + psqt;
    
    // Step 11: Complexity adjustment
    let complexity = (psqt - positional).abs();
    score -= score * complexity / 18000;
    
    // Step 12: Material scaling
    let material = count_material(position);
    score = score * (77777 + material) / 77777;
    
    // Step 13: Clamp
    score.clamp(-31753, 31753)
}
```

### Incremental Update

```rust
pub fn make_move(&mut self, m: Move, network: &Network) {
    let from = m.from();
    let to = m.to();
    let piece = self.piece_at(from);
    
    // Calculate feature changes
    let old_feature_white = calculate_feature(piece, from, self.white_king_sq(), WHITE);
    let new_feature_white = calculate_feature(piece, to, self.white_king_sq(), WHITE);
    
    let old_feature_black = calculate_feature(piece, from, self.black_king_sq(), BLACK);
    let new_feature_black = calculate_feature(piece, to, self.black_king_sq(), BLACK);
    
    // Check if king moved
    if piece.is_king() {
        // Full refresh needed
        self.accumulator.computed[piece.color()] = false;
    } else {
        // Incremental update
        update_accumulator_incremental(
            &mut self.accumulator,
            WHITE,
            &[old_feature_white],
            &[new_feature_white],
            network
        );
        
        update_accumulator_incremental(
            &mut self.accumulator,
            BLACK,
            &[old_feature_black],
            &[new_feature_black],
            network
        );
    }
    
    // Update board state
    self.pieces[from] = None;
    self.pieces[to] = Some(piece);
}

fn update_accumulator_incremental(
    acc: &mut Accumulator,
    perspective: Color,
    removed: &[usize],
    added: &[usize],
    network: &Network
) {
    // Subtract removed features
    for &feature_idx in removed {
        for i in 0..1536 {
            acc.accumulation[perspective][i] -= network.ft_weights[feature_idx][i];
        }
        for bucket in 0..8 {
            acc.psqt_accumulation[perspective][bucket] -= network.psqt_weights[feature_idx][bucket];
        }
    }
    
    // Add new features
    for &feature_idx in added {
        for i in 0..1536 {
            acc.accumulation[perspective][i] += network.ft_weights[feature_idx][i];
        }
        for bucket in 0..8 {
            acc.psqt_accumulation[perspective][bucket] += network.psqt_weights[feature_idx][bucket];
        }
    }
}
```

### Transform Step

```rust
fn transform_accumulator(acc: &Accumulator) -> [i8; 1536] {
    let mut output = [0i8; 1536];
    
    // Process white's perspective
    for i in 0..768 {
        let v1 = acc.accumulation[WHITE][i] as i32;
        let v2 = acc.accumulation[WHITE][i + 768] as i32;
        let product = (v1 * v2) / 512;
        output[i] = clipped_relu(product);
    }
    
    // Process black's perspective
    for i in 0..768 {
        let v1 = acc.accumulation[BLACK][i] as i32;
        let v2 = acc.accumulation[BLACK][i + 768] as i32;
        let product = (v1 * v2) / 512;
        output[i + 768] = clipped_relu(product);
    }
    
    output
}
```

---

## Glossary

| Term | Definition |
|------|------------|
| **Accumulator** | Cache that stores Feature Transformer output, enabling incremental updates |
| **Activation Function** | Non-linear function applied after layer (ClippedReLU, SqrClippedReLU) |
| **Centipawn** | 1/100 of a pawn; standard unit for chess evaluation (100 cp = 1 pawn advantage) |
| **ClippedReLU** | Activation: max(0, min(127, x >> 6)) |
| **Dense Layer** | Fully-connected neural network layer (FC_0, FC_1, FC_2) |
| **Feature** | Binary input representing "piece X on square Y relative to king position Z" |
| **Feature Transformer** | First layer converting 22,528 sparse features → 3,072 dense values |
| **Forward Pass** | Running input through all layers to get output |
| **Horizontal Mirroring** | Using left-right symmetry to halve training data |
| **Incremental Update** | Only updating changed features instead of full recalculation |
| **King Bucket** | Group of similar king positions (32 buckets per side) |
| **Layer Stack** | Complete set of FC_0/FC_1/FC_2 weights for a game phase (8 total) |
| **LEB128** | Variable-length integer compression (Little Endian Base 128) |
| **NNUE** | Efficiently Updatable Neural Network |
| **Perspective** | Viewpoint (white's or black's) for evaluation |
| **PSQT** | Piece-Square Table; traditional positional evaluation |
| **PSQT Bucket** | One of 8 game-phase-specific PSQTs based on piece count |
| **Quantization** | Converting floating-point to fixed-point integers |
| **Residual Connection** | Skip connection adding earlier layer output to final output |
| **SIMD** | Single Instruction Multiple Data; parallel processing |
| **Sparsity** | Most features are 0 (only ~30 of 22,528 are active) |
| **SqrClippedReLU** | Squared activation: max(0, min(127, (x²) >> 19)) |
| **Transform Step** | Converting 3,072 accumulator values → 1,536 via pair multiplication |

---

## Summary

NNUE revolutionized computer chess by combining:

1. **Sparse Input Representation**: 22,528 features, but only ~30 active
2. **Incremental Updates**: Accumulator caches results, only update what changed
3. **Quantization**: Fast integer math instead of slow floating-point
4. **King Buckets**: Group similar king positions (64 → 32)
5. **Horizontal Mirroring**: Use left-right symmetry to reduce parameters
6. **Layer Stacks**: 8 specialized networks for different game phases
7. **PSQT Integration**: Combine traditional chess knowledge with neural network
8. **Residual Connections**: Preserve information through deep network
9. **SIMD Optimization**: Process 16 values at once

**Result:** A neural network that evaluates **1-2 million positions per second** while playing at **superhuman strength**.

The key insight is that chess positions are **sparse** - only a few pieces out of many possible positions. By caching the expensive calculations and only updating what changed, NNUE achieves a **2,000,000x speedup** over naive neural networks while maintaining full accuracy!

---
