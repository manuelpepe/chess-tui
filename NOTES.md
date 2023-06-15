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
    - [x] Castling
- [x] Highlight grabbed piece
- [x] Highlight legal moves
- [ ] Highlight best move
- [ ] Add scrolling:
    - [ ] Help window
    - [ ] Console
- [ ] Improve `:move` parsing
    - [x] Support castling
    - [ ] Support en-pasant
    - [ ] Support non-queen promotion
- [ ] Improve `Legal Moves` pane:
    - [ ] Make move by clicking enter
    - [ ] Toggle grouping by piece
- [ ] Migrate from `tui-rs` to `ratatui`

### Mid-term

- [ ] Track material imbalance
- [ ] Move history
- [ ] Settings 
- [ ] Clocks for playing
- [ ] Parse FEN clocks
- [ ] Command work:
    - [ ] `!fen`: get FEN of current position  (remove `:set-position`)
    - [ ] `!pgn`: get PGN of current move history
    - [ ] `:fen <fen>`: set a position on the board
    - [ ] `:pgn <pgn>`: load pgn to move history
    - [x] `:passturn`: pass turn
    - [x] `:flipboard`: flip board vertically


### Long-term

- [ ] Maybe some network stuff
    - [ ] Self-hosted game ?
    - [ ] Liches Bot API ?
    - [ ] Stream games from lichess / chess.com ?
- [ ] Chess960