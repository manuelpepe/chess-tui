# Notes

## Some positions:

* rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq
* r2qk2r/pp3ppp/B1nbpn2/2pp1b2/Q2P1B2/2P1PN2/PP1N1PPP/R3K2R b KQkq


## Calcs

File from index: 7 - (i * 8 + j) / 8
Rank from index: (i * 8 + j) % 8


## TODO:

### Short-term

- [x] Parse FEN turn and castling rights
- [x] Update engine when moves are made (restart search)
- [ ] Choose promoted piece
- [x] Only allow valid moves
    - [x] Pseudo-legal moves
    - [x] Legal moves
    - [x] En passant
    - [ ] Castling
- [x] Highlight grabbed piece
- [x] Highlight legal moves
- [ ] Highlight best move


### Mid-term

- [ ] Move history
- [ ] Settings 
- [ ] Clocks for playing
- [ ] Parse FEN clocks
- [ ] More commands:
    - [ ] Get FEN 
    - [ ] Swap turns
    - [ ] Invert board

### Long-term

- [ ] Maybe some network stuff
- [ ] Chess960