# Back To Square 1

### Current Problem: Legal Move Gen

##### 1st attempt: 30 nano seconds in perft(1) failed at perft(3) -> Make / Unmake based legality check + 3 buckets for types (Capture, Quiet, Promo) 
##### 2nd attempt: Failed at perft(6), Failed at Kiwipete(3) -> Constraint based legality check.
##### 3rd attempt: ONGOING

- The Plan:
    - Make / Unmake Based Legality Check 
    - [Board;12] -> [Board;6] & [Color;2]
    - Mailbox for lookups [Option(Piece,Color); 64]
    - IMA GET PERFT THIS TIME
