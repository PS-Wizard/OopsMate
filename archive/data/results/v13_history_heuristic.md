# Against Sayuri
Score of OopsMate vs Sayuri 2018.05.23: 82 - 125 - 93  [0.428] 300
...      OopsMate playing White: 36 - 61 - 53  [0.417] 150
...      OopsMate playing Black: 46 - 64 - 40  [0.440] 150
...      White vs Black: 100 - 107 - 93  [0.488] 300
Elo difference: -50.1 +/- 32.9, LOS: 0.1 %, DrawRatio: 31.0 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf

Player: OopsMate
   "Draw by 3-fold repetition": 89
   "Draw by fifty moves rule": 4
   "Loss: Black loses on time": 3
   "Loss: Black mates": 59
   "Loss: Black's connection stalls": 1
   "Loss: White loses on time": 2
   "Loss: White mates": 60
   "Win: Black mates": 46
   "Win: White mates": 36
Player: Sayuri 2018.05.23
   "Draw by 3-fold repetition": 89
   "Draw by fifty moves rule": 4
   "Loss: Black mates": 46
   "Loss: White mates": 36
   "Win: Black loses on time": 3
   "Win: Black mates": 59
   "Win: Black's connection stalls": 1
   "Win: White loses on time": 2
   "Win: White mates": 60
Finished match
[wizard@nixos ~/Projects/OopsMate/archive/data/opponents] ->

# Against Previous OopsMate
Score of OopsMate-v13 vs OopsMate-v12: 53 - 26 - 221  [0.545] 300
...      OopsMate-v13 playing White: 26 - 15 - 109  [0.537] 150
...      OopsMate-v13 playing Black: 27 - 11 - 112  [0.553] 150
...      White vs Black: 37 - 42 - 221  [0.492] 300
Elo difference: 31.4 +/- 20.0, LOS: 99.9 %, DrawRatio: 73.7 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf

Player: OopsMate-v13
   "Draw by 3-fold repetition": 212
   "Draw by fifty moves rule": 3
   "Draw by insufficient mating material": 6
   "Loss: Black mates": 14
   "Loss: Black's connection stalls": 1
   "Loss: White loses on time": 1
   "Loss: White mates": 10
   "Win: Black mates": 27
   "Win: White mates": 26
Player: OopsMate-v12
   "Draw by 3-fold repetition": 212
   "Draw by fifty moves rule": 3
   "Draw by insufficient mating material": 6
   "Loss: Black mates": 27
   "Loss: White mates": 26
   "Win: Black mates": 14
   "Win: Black's connection stalls": 1
   "Win: White loses on time": 1
   "Win: White mates": 10
Finished match
[wizard@nixos ~/Projects/OopsMate/archive/data/opponents] ->
[wizard@nixos ~/Projects/OopsMate/archive/data/opponents] ->

# Against Stockfish-1950
Finished game 95 (OopsMate vs SF-1950): 1/2-1/2 {Draw by fifty moves rule}
Score of OopsMate vs SF-1950: 57 - 26 - 17  [0.655] 100
...      OopsMate playing White: 28 - 15 - 7  [0.630] 50
...      OopsMate playing Black: 29 - 11 - 10  [0.680] 50
...      White vs Black: 39 - 44 - 17  [0.475] 100
Elo difference: 111.4 +/- 65.4, LOS: 100.0 %, DrawRatio: 17.0 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf

Player: OopsMate
   "Draw by 3-fold repetition": 14
   "Draw by fifty moves rule": 3
   "Loss: Black mates": 14
   "Loss: White loses on time": 1
   "Loss: White mates": 11
   "Win: Black mates": 29
   "Win: White mates": 28
Player: SF-1950
   "Draw by 3-fold repetition": 14
   "Draw by fifty moves rule": 3
   "Loss: Black mates": 29
   "Loss: White mates": 28
   "Win: Black mates": 14
   "Win: White loses on time": 1
   "Win: White mates": 11

# Against Stockfish-2000
Score of OopsMate vs SF-2000: 50 - 40 - 10  [0.550] 100
...      OopsMate playing White: 29 - 17 - 4  [0.620] 50
...      OopsMate playing Black: 21 - 23 - 6  [0.480] 50
...      White vs Black: 52 - 38 - 10  [0.570] 100
Elo difference: 34.9 +/- 65.6, LOS: 85.4 %, DrawRatio: 10.0 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf

Player: OopsMate
   "Draw by 3-fold repetition": 4
   "Draw by fifty moves rule": 6
   "Loss: Black mates": 16
   "Loss: White loses on time": 1
   "Loss: White mates": 23
   "Win: Black mates": 21
   "Win: White mates": 29
Player: SF-2000
   "Draw by 3-fold repetition": 4
   "Draw by fifty moves rule": 6
   "Loss: Black mates": 21
   "Loss: White mates": 29
   "Win: Black mates": 16
   "Win: White loses on time": 1
   "Win: White mates": 23
Finished match
[wizard@nixos ~/Projects/OopsMate/archive/data/opponents] ->

# ORDO

[wizard@nixos ~/Projects/OopsMate/archive/tools] -> ./ordo -a 1807 -A "Sayuri" -p oops_vs_sayuri.pgn

File: oops_vs_sayuri.pgn
Loading data (2000 games x dot):

|

Total games                 308
 - White wins               100
 - Draws                     93
 - Black wins               107
 - Truncated/Discarded        8
Unique head to head        0.65%
Reference rating      1807.0 (set to "Sayuri")

Loose Anchors = none
Relative Anchors = none


players with no games = 2
players with all wins = 0
players w/ all losses = 0
players, total purged = 2


Convergence rating calculation (cycle #1)

phase iteration    deviation    resolution
  0       1    100.027052618      58.90411
  1       1    100.027052618      53.54919
  2       2     88.995416141      46.34177
  3       2     73.873147953      38.65319
  4       2     62.940844848      32.40637
  5       3     51.168439743      26.29765
  6       5     40.372376042      20.74004
  7       6     30.508965653      15.66422
  8       7     21.560419656      11.06147
  9      10     13.483116777       6.91903
 10      15      6.332641846       3.24892
 11      34      0.832552131       0.42691
 12     117      0.000000525       0.00000
 13       4      0.000000078       0.00000
done

White Advantage = 0.0
Draw Rate (eq.) = 50.0 %


   # PLAYER      :  RATING  POINTS  PLAYED   (%)
   1 Sayuri      :  1807.0   171.5     300    57
   2 OopsMate    :  1756.4   128.5     300    43

White advantage = 0.00
Draw rate (equal opponents) = 50.00 %


done!
[wizard@nixos ~/Projects/OopsMate/archive/tools] ->
