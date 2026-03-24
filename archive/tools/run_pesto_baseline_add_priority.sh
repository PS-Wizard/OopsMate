#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd -- "$script_dir/../.." && pwd)"
runner="$script_dir/run_feature_match.sh"
base_dir="$root_dir/archive/binaries/feature-matrix/pesto/baseline-add"
results_dir="$root_dir/archive/data/results/feature-matrix/pesto/baseline-add"
base_engine="$base_dir/oopsmate_pesto_base"

variants=(
  add-tt-cutoffs
  add-pvs
  add-null-move
  add-lmr
  add-history-heuristic
  add-tt-move-ordering
  add-see
  add-futility
  add-aspiration-windows
  add-killer-moves
  add-check-extensions
  add-iid
  add-singular-extensions
  add-reverse-futility
  add-razoring
  add-probcut
)

for variant in "${variants[@]}"; do
  "$runner" \
    "$base_engine" "pesto-base" \
    "$base_dir/oopsmate_pesto_${variant}" "pesto-${variant}" \
    "$results_dir/base_vs_${variant}.log" \
    8
done
