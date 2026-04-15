### OopsMate

This is a chess engine written in rust, with 0 external dependencies. It contains:

* `strikes`: crate for attack patterns
* `nnuebie`: Stockfish Half_ka_hm_v2 port
* `nnue-core`: Same stockfish Half_ka_hm_v2 inference but without the optimizations. Good starting ground

**Development Has been shifted to [OopsMate-V2](https://github.com/PS-Wizard/oopsmate-v2/) as a clean rewrite. V2 aims to reach *atleast* top 50** 

---

#### Based off of internal testing, we are ~3.3k 

##### OopsMate vs Minke (3464 CCRL 40/15):

```
Score of OopsMate-abation vs Minke: 10 - 56 - 34  [0.270] 100
...      OopsMate-abation playing White: 7 - 19 - 24  [0.380] 50
...      OopsMate-abation playing Black: 3 - 37 - 10  [0.160] 50
...      White vs Black: 44 - 22 - 34  [0.610] 100
Elo difference: -172.8 +/- 58.7, LOS: 0.0 %, DrawRatio: 34.0 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf
```

##### OopsMate vs Eleanor (3402 CCRL 40/15):

```
Score of OopsMate vs Eleanor: 19 - 35 - 46  [0.420] 100
...      OopsMate playing White: 11 - 10 - 29  [0.510] 50
...      OopsMate playing Black: 8 - 25 - 17  [0.330] 50
...      White vs Black: 36 - 18 - 46  [0.590] 100
Elo difference: -56.1 +/- 50.4, LOS: 1.5 %, DrawRatio: 46.0 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf
```

##### OopsMate Vs Combusken (3370 CCRL 40/15):

```
Finished game 100 (Combusken vs OopsMate): 0-1 {Black wins by adjudication}
Score of OopsMate vs Combusken: 61 - 14 - 25  [0.735] 100
...      OopsMate playing White: 40 - 1 - 9  [0.890] 50
...      OopsMate playing Black: 21 - 13 - 16  [0.580] 50
...      White vs Black: 53 - 22 - 25  [0.655] 100
Elo difference: 177.2 +/- 64.7, LOS: 100.0 %, DrawRatio: 25.0 %
```

##### OopsMate Vs Bitbit ( 3302 CCRL 40/15 ):

```
Score of OopsMate vs Bitbit: 37 - 29 - 34  [0.540] 100
...      OopsMate playing White: 27 - 9 - 14  [0.680] 50
...      OopsMate playing Black: 10 - 20 - 20  [0.400] 50
...      White vs Black: 47 - 19 - 34  [0.640] 100
Elo difference: 27.9 +/- 55.8, LOS: 83.8 %, DrawRatio: 34.0 %
```

##### OopsMate vs Seredina (3204 CCRL):

```
Score of OopsMate vs Seredina: 59 - 8 - 33  [0.755] 100
...      OopsMate playing White: 39 - 2 - 9  [0.870] 50
...      OopsMate playing Black: 20 - 6 - 24  [0.640] 50
...      White vs Black: 45 - 22 - 33  [0.615] 100
Elo difference: 195.5 +/- 59.9, LOS: 100.0 %, DrawRatio: 33.0 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf
```

##### OopsMate vs Meltdown (3064 CCRL 40/15):

```
Finished game 100 (Meltdown vs OopsMate): 1/2-1/2 {Draw by adjudication}
Score of OopsMate vs Meltdown: 74 - 5 - 21  [0.845] 100
...      OopsMate playing White: 43 - 0 - 7  [0.930] 50
...      OopsMate playing Black: 31 - 5 - 14  [0.760] 50
...      White vs Black: 48 - 31 - 21  [0.585] 100
Elo difference: 294.6 +/- 75.5, LOS: 100.0 %, DrawRatio: 21.0 %
SPRT: llr 0 (0.0%), lbound -inf, ubound inf
```

>[!NOTE]
> Command Used To Test:
> ```
> cutechess-cli \
>  -engine cmd=./oops_mate name="OopsMate" \
>  -engine cmd=./<opponent_binary> name="<opponent_name>" \
>  -each proto=uci tc=40/15 \
>  -rounds 100 \
>  -openings file=Modern.pgn format=pgn order=random plies=16 \
>  -pgnout match_40_4.pgn \
>  -concurrency 1 \
>  -draw movenumber=40 movecount=8 score=10 \
>  -resign movecount=5 score=600
>  ```


