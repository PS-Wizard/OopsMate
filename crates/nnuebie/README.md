# nnuebie
A high-performance, thread-safe, SIMD-accelerated NNUE (Efficiently Updatable Neural Network) inference library for Chess engines written in Rust.

This crate provides a drop-in evaluation component compatible with Stockfish-trained NNUE networks (SFNNv9 architecture). It is designed for maximum throughput, supporting both single-threaded engines and highly parallel environments (LazySMP) with minimal memory overhead.

## Features
-   **High Performance**: Uses AVX2 SIMD intrinsics for both accumulator updates and the forward pass.
-   **Thread Safety**: Separates read-only network weights (`Arc<NnueNetworks>`) from thread-local state (`NNUEProbe`), allowing thousands of threads to share a single copy of the heavy network data.
-   **Incremental Updates**: Efficiently updates accumulators based on moves (added/removed pieces) rather than refreshing from scratch.
-   **Stockfish Compatibility**: Loads standard `.nnue` files (Big/Small networks) used by modern Stockfish versions. Make sure the Hashes Match. This crate **strictly only supports `HalfKa_hm_v2`**

## Usage

```bash
# Enable AVX2/Native optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### Single-Threaded Example

```rust
use nnuebie::{NNUEProbe, Piece, Color};

fn main() -> std::io::Result<()> {
    let mut probe = NNUEProbe::new("big.nnue", "small.nnue")?;

    // Set up board (simplified example)
    let pieces = vec![(Piece::WhiteKing, 4), (Piece::BlackKing, 60)];
    probe.set_position(&pieces, 0);

    let score = probe.evaluate(Color::White);
    println!("Evaluation: {}", score);
    
    Ok(())
}
```

### Multi-Threaded Example (LazySMP)

For parallel search, load the networks once and share them.

```rust
use nnuebie::{NNUEProbe, NnueNetworks, Piece, Color};
use std::sync::Arc;
use std::thread;

fn main() {
    // 1. Load networks (Heavy I/O)
    let networks = Arc::new(NnueNetworks::new("big.nnue", "small.nnue").unwrap());

    // 2. Spawn threads
    let mut handles = vec![];
    for _ in 0..8 {
        let net_ref = networks.clone();
        handles.push(thread::spawn(move || {
            // 3. Create lightweight thread-local probe
            let mut probe = NNUEProbe::from_networks(net_ref);
            
            // Use probe...
            // probe.set_position(...)
            // probe.apply_delta(...)
            // probe.evaluate(...)
        }));
    }
    
    for h in handles { h.join().unwrap(); }
}
```

## Performance

| Operation | Throughput |
|-----------|------------|
| **Full Refresh** (1 thread) | ~836,000 evals/sec |
| **Incremental** (1 thread) | ~1,392,000 evals/sec |
| **Speedup** | 1.66x |

*Benchmarks run on a single core using `cargo run --release --bin benchmark` with `target-cpu=native`.*

## Current Bottlenecks:

```
Overhead  Symbol
────────  ──────────────────────────────────────────────────────────────────────
 34.70%   <AffineTransform as Layer>::propagate
 17.37%   NNUEProbe::evaluate
 15.18%   finny_tables::update_accumulator_refresh_cache
  7.58%   accumulator::update_accumulators_single_pass_avx2
  5.81%   finny_tables::update_accumulator_refresh_cache  (2)
  3.46%   NNUEProbe::set_position
  1.47%   NNUEProbe::add_piece_internal
  1.22%   libc  0x185e9c
  1.05%   libc  0x185e8e
  0.91%   libc  0x185e95
  0.87%   libc  0x185e87
  0.80%   AccumulatorState::new
  0.76%   benchmark::main
  0.59%   AccumulatorStack::update_incremental
  0.56%   libc  cfree
  0.55%   Accumulator<_>::update_incremental_perspective
  0.50%   Accumulator<_>::update_incremental_perspective  (2)
  0.49%   loader::read_leb128_signed_checked
  0.40%   Accumulator<_>::update_incremental_perspective  (3)
  0.35%   libc  0x185e4f
  0.33%   Accumulator<_>::update_incremental_perspective  (4)
  0.20%   libc  0x0a5c1f
────────  ──────────────────────────────────────────────────────────────────────
```
