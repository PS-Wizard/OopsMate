#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 5 || $# -gt 6 ]]; then
  echo "usage: $0 <engine_a> <name_a> <engine_b> <name_b> <log_file> [concurrency]" >&2
  exit 1
fi

engine_a="$1"
name_a="$2"
engine_b="$3"
name_b="$4"
log_file="$5"
concurrency="${6:-2}"
nice_level="${NICE_LEVEL:-10}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd -- "$script_dir/../.." && pwd)"
openings_dir="$root_dir/archive/data/opponents"
openings_file="$openings_dir/Modern.pgn"

mkdir -p "$(dirname -- "$log_file")"

{
  printf 'started_at=%s\n' "$(date -Is)"
  printf 'engine_a=%s\n' "$engine_a"
  printf 'name_a=%s\n' "$name_a"
  printf 'engine_b=%s\n' "$engine_b"
  printf 'name_b=%s\n' "$name_b"
  printf 'tc=%s\n' '8+0.08'
  printf 'rounds=%s\n' '50'
  printf 'games_per_encounter=%s\n' '2'
  printf 'total_games=%s\n' '100'
  printf 'repeat=%s\n' 'true'
  printf 'concurrency=%s\n' "$concurrency"
  printf 'hash=%s\n' '64'
  printf 'threads=%s\n' '1'
  printf 'wait_ms=%s\n' '1000'
  printf 'timemargin_ms=%s\n' '200'
  printf 'nice_level=%s\n' "$nice_level"
  printf 'openings=%s\n' "$openings_file"
  printf '%s\n' '---'
} > "$log_file"

cd "$openings_dir"

runner_cmd=(nice -n "$nice_level")
runner_cmd+=(cutechess-cli)

"${runner_cmd[@]}" \
  -engine cmd="$engine_a" name="$name_a" option."Hash"=64 option."Threads"=1 \
  -engine cmd="$engine_b" name="$name_b" option."Hash"=64 option."Threads"=1 \
  -each proto=uci tc=8+0.08 timemargin=200 \
  -games 2 \
  -rounds 50 \
  -repeat \
  -recover \
  -wait 1000 \
  -openings file=Modern.pgn format=pgn order=random plies=16 \
  -concurrency "$concurrency" \
  -draw movenumber=40 movecount=8 score=10 \
  -resign movecount=5 score=600 \
  2>&1 | tee -a "$log_file"
