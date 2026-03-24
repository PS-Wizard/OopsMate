#!/usr/bin/env bash
set -euo pipefail

root_dir="$(git rev-parse --show-toplevel)"
target_dir="$root_dir/target/release"
output_root="$root_dir/archive/binaries/feature-matrix"

all_features="pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"

baseline_names=(
  base
  add-pvs
  add-aspiration-windows
  add-check-extensions
  add-null-move
  add-lmr
  add-futility
  add-reverse-futility
  add-razoring
  add-probcut
  add-tt-cutoffs
  add-killer-moves
  add-history-heuristic
  add-see
  add-iid
  add-singular-extensions
  add-tt-move-ordering
)

baseline_features=(
  ""
  "pvs"
  "aspiration-windows"
  "check-extensions"
  "null-move"
  "lmr"
  "futility"
  "reverse-futility"
  "razoring"
  "probcut"
  "tt-cutoffs"
  "killer-moves"
  "history-heuristic"
  "see"
  "tt-cutoffs,iid"
  "tt-cutoffs,singular-extensions"
  "tt-cutoffs,tt-move-ordering"
)

full_names=(
  full
  sub-pvs
  sub-aspiration-windows
  sub-iid
  sub-singular-extensions
  sub-check-extensions
  sub-null-move
  sub-lmr
  sub-futility
  sub-reverse-futility
  sub-razoring
  sub-probcut
  sub-tt-move-ordering
  sub-killer-moves
  sub-history-heuristic
  sub-see
  sub-tt-stack
)

full_features=(
  "$all_features"
  "aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,iid,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,razoring,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,killer-moves,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,history-heuristic,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,see"
  "pvs,aspiration-windows,iid,singular-extensions,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,tt-cutoffs,tt-move-ordering,killer-moves,history-heuristic"
  "pvs,aspiration-windows,check-extensions,null-move,lmr,futility,reverse-futility,razoring,probcut,killer-moves,history-heuristic,see"
)

build_variant() {
  local bin_name="$1"
  local eval_dir="$2"
  local experiment_dir="$3"
  local variant_name="$4"
  local features="$5"
  local output_path="$output_root/$eval_dir/$experiment_dir/oopsmate_${eval_dir}_${variant_name}"

  printf '\n[%s/%s] building %s\n' "$eval_dir" "$experiment_dir" "$variant_name"
  if [[ -n "$features" ]]; then
    cargo build --release --bin "$bin_name" --no-default-features --features "$features" -j 1
  else
    cargo build --release --bin "$bin_name" --no-default-features -j 1
  fi
  install -m 755 "$target_dir/$bin_name" "$output_path"
}

build_eval_set() {
  local bin_name="$1"
  local eval_dir="$2"

  local i
  for ((i = 0; i < ${#baseline_names[@]}; i++)); do
    build_variant "$bin_name" "$eval_dir" "baseline-add" "${baseline_names[$i]}" "${baseline_features[$i]}"
  done

  for ((i = 0; i < ${#full_names[@]}; i++)); do
    build_variant "$bin_name" "$eval_dir" "full-subtract" "${full_names[$i]}" "${full_features[$i]}"
  done
}

build_eval_set "oopsmate-nnue" "nnue"
build_eval_set "oopsmate-pesto" "pesto"

printf '\nDone. Binaries stored under %s\n' "$output_root"
