# Chessbot5

This is a WIP project to create a chess program with an AI capable of beating most humans.
For now the entire application is entirely terminal-based, we hope to change that at some point in the future.

## How to install

First install Rust from this guide: https://www.rust-lang.org/learn/get-started

In a terminal, navigate to the directory you want to install in, then type
```
git clone https://github.com/SteinarSi/Chessbot5.git
cd Chessbot5
cargo build --release
```

To start the program:
```
cargo run --release
```

## How to play

To move a piece, type a move into the terminal and press enter.

The game uses a coordinate-based notation for moves, rather than the more common algebraic notation.
Simply type in the coordinates the piece is moving from, and the coordinates you're moving to in one word.

To promote a piece, add an extra letter at the end to signify what to promote to.
Q for Queen, N for Knight, B for Bishop and R for Rook.

Some examples:
```
g1f3      //Moves something from g1 to f3
e1e2      //Moves something from e1 to e2
f7g8Q     //Moves a pawn from f7 to g8, and promotes it to a queen
```
