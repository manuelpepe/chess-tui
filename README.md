# Chess TUI

A terminal-based chess interface.


## Features

* Playable board
* Interactive commands
* UCI engine integration


## Usage

To use the engine features, download [Stockfish](https://stockfishchess.org/download/) and pass the path to the executable with the `-P` parameter. 

```
cargo run -- -h
cargo run -- -P ./path/to/sf
```

Press `<TAB>` to move between windows, for more info see `Help` window.

## Commands

To enter the command line press ':', then use any of:

* `!<fen>` or `:set-position <fen>`: set a position on the board 
* `:search` start searching current position
* `:stop` stop searching current position
* `exit`: exit the program