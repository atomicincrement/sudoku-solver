# A sudoku solver to solve Vincent Hindriksen's challenge on Linkedin.

Given the sudoku

```
73___4___
2___7__3_
9__3__4__
571643289
8237__164
469821_7_
__5__7___
_9__1___8
___4____6
```

Can we find the next move?

## Plan

* Build a board model with three kinds of 9 bit sets, squares, rows and columns. ie. [9; u16]
* Initialise the board from Vincent's sudoku.
* Research methods of calculating the next move in Sudoku.
* Implement enough methods to complete this sudoku.
