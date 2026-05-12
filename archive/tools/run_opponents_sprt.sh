#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd -- "$script_dir/../.." && pwd)"
opponents_dir="$root_dir/archive/data/opponents"
results_dir="$root_dir/archive/data/results/sprt"
summary_file="$results_dir/sprt_final_results.txt"

mkdir -p "$results_dir"
cd "$opponents_dir"

printf 'OopsMate 1v1 SPRT Summary\n' > "$summary_file"
printf '=========================\n' >> "$summary_file"

for opp in bitbit boot7 combusken eleanor elixir_linux goldfish Koivisto meltdown minke reckless-linux-avx512 sayuri seredina; do
    echo "================================================="
    echo " Starting SPRT match against: $opp"
    echo "================================================="

    ./fastchess -engine cmd=./oopsmate-nnue name=OopsMate-NNUE \
               -engine cmd=./"$opp" name="$opp" \
               -each tc=10+0.1 -concurrency 5 \
               -rounds 20000 -repeat \
               -sprt elo0=0 elo1=5 alpha=0.05 beta=0.05 model=logistic \
               -openings file=book.epd format=epd order=random \
               -draw movenumber=40 movecount=8 score=10 \
               -recover \
               -pgnout file="$results_dir/sprt_${opp}.pgn" | tee "$results_dir/log_${opp}.txt"

    printf '%s\n' '--------------------------------------------------' >> "$summary_file"
    printf ' Final Result vs %s\n' "$opp" >> "$summary_file"
    tail -n 15 "$results_dir/log_${opp}.txt" | grep -E "Rank|OopsMate|$opp|LLR" >> "$summary_file"
    printf '\n' >> "$summary_file"

    rm "$results_dir/log_${opp}.txt"
done

echo "================================================="
echo " All matches complete! Check $summary_file"
