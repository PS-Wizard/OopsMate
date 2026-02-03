# OopsMate v2 Performance Results

## Summary

| Opponent | Games | Score | Win-Loss-Draw | Elo Diff | LOS | Draw % |
|----------|-------|-------|---------------|----------|-----|--------|
| **SF-1750** | 300 | 55% | 150-121-29 | **+33.7 ± 37.6** | **96.1%** | 9.7% |
| **OopsMate-prev** | 100 | 64% | 43-15-42 | **+100.0 ± 52.8** | **100.0%** | 42.0% |

---

## vs Stockfish 1750

| Metric | Value |
|--------|-------|
| **Total Games** | 300 |
| **Score** | 150-121-29 (55.0%) |
| **Elo Difference** | +33.7 ± 37.6 |
| **Estimated Rating** | ~1784 Elo |
| **LOS (Likelihood of Superiority)** | 96.1% |
| **Draw Rate** | 9.7% |

### Performance by Color
- **As White**: 76-60-14 (55.3%)
- **As Black**: 74-61-15 (54.3%)

### Game Outcomes
- **Wins**: 150 (76 White, 74 Black)
- **Losses**: 121 (59 White, 56 Black)
- **Draws**: 29 (18 3-fold rep, 9 fifty-move, 2 other)
- **Time Losses**: 6 (4 as White, 2 as Black)

---

## vs OopsMate Previous Version

| Metric | Value |
|--------|-------|
| **Total Games** | 100 |
| **Score** | 43-15-42 (64.0%) |
| **Elo Improvement** | +100.0 ± 52.8 |
| **LOS (Likelihood of Superiority)** | 100.0%  |
| **Draw Rate** | 42.0% |

### Performance by Color
- **As White**: 19-7-24 (62.0%)
- **As Black**: 24-8-18 (66.0%)

### Game Outcomes
- **Wins**: 43 (19 White, 24 Black)
- **Losses**: 15 (8 White, 6 Black, 1 time loss)
- **Draws**: 42 (41 3-fold rep, 1 insufficient material)

---

## Key Takeaways

✅ **v2 is ~1784 Elo** (96% confidence vs SF-1750)  
✅ **+100 Elo improvement** over previous version (100% confidence)  
⚠️ **6 time losses** vs SF-1750 - time management could use refinement

---
# RAW
## Against stockfish SF-1750

Score of OopsMate vs SF-1750: 150 - 121 - 29  [0.548] 300
...      OopsMate playing White: 76 - 60 - 14  [0.553] 150
...      OopsMate playing Black: 74 - 61 - 15  [0.543] 150
...      White vs Black: 137 - 134 - 29  [0.505] 300
Elo difference: 33.7 +/- 37.6, **LOS: 96.1 %**, DrawRatio: 9.7 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf

Player: OopsMate
   "Draw by 3-fold repetition": 18
   "Draw by fifty moves rule": 9
   "Draw by insufficient mating material": 1
   "Draw by timeout": 1
   "Loss: Black loses on time": 2
   "Loss: Black mates": 56
   "Loss: White loses on time": 4
   "Loss: White mates": 59
   "Win: Black mates": 74
   "Win: White mates": 76
Player: SF-1750
   "Draw by 3-fold repetition": 18
   "Draw by fifty moves rule": 9
   "Draw by insufficient mating material": 1
   "Draw by timeout": 1
   "Loss: Black mates": 74
   "Loss: White mates": 76
   "Win: Black loses on time": 2
   "Win: Black mates": 56
   "Win: White loses on time": 4
   "Win: White mates": 59
Finished match
[wizard@nixos ~/Projects/OopsMate/archive/results/v2] ->

---

## AGAINST PREVIOUS OOPSMATE

Finished game 100 (OopsMate-prev vs OopsMate-v2): 0-1 {Black mates}
Score of OopsMate-v2 vs OopsMate-prev: 43 - 15 - 42  [0.640] 100
...      OopsMate-v2 playing White: 19 - 7 - 24  [0.620] 50
...      OopsMate-v2 playing Black: 24 - 8 - 18  [0.660] 50
...      White vs Black: 27 - 31 - 42  [0.480] 100
Elo difference: 100.0 +/- 52.8, **LOS: 100.0 %**, DrawRatio: 42.0 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf

Player: OopsMate-v2
   "Draw by 3-fold repetition": 41
   "Draw by insufficient mating material": 1
   "Loss: Black mates": 6
   "Loss: White loses on time": 1
   "Loss: White mates": 8
   "Win: Black mates": 24
   "Win: White mates": 19
Player: OopsMate-prev
   "Draw by 3-fold repetition": 41
   "Draw by insufficient mating material": 1
   "Loss: Black mates": 24
   "Loss: White mates": 19
   "Win: Black mates": 6
   "Win: White loses on time": 1
   "Win: White mates": 8
Finished match
[wizard@nixos ~/Projects/OopsMate/archive/results/v2] ->



