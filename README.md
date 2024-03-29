# Chess TUI

A terminal-based chess interface.

<img src="assets/demo.gif" width="70%" alt="Chess TUI Demo"/>


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

Click on the board to move pieces, or press `:` and use the `:move <mv>` command.
Press `<TAB>` to move between windows, for more info see `Help` window.

## Commands

To enter the command line press `:`, then use any of:

* `!fen`: print current position as a FEN string in the console
* `:fen <fen>`: set a position on the board 
* `:search`: start searching current position
* `:stop`: stop searching current position
* `:move <mv>`: play move on the board. long algebraic notation is used (i.e. e2e4)
* `:passturn`: pass current player turn
* `:flipboard`: flip board vertically
* `:q` or `exit`: exit the program