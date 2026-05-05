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

### Phase 1: Core Data Structure
* Create a `Board` struct representing the 9×9 sudoku grid
* Implement cell representation using u16 (16-bit integers) to track:
  - Empty cells as sets of possible values (1-9)
  - Filled cells with their definite value
* Build three constraint trackers: `[u16; 9]` each for:
  - Row constraints (what values are possible in each row)
  - Column constraints (what values are possible in each column)  
  - Box constraints (what values are possible in each 3×3 box)

### Phase 2: Board Initialization
* Parse Vincent's sudoku puzzle into the `Board` structure
* Initialize cell possibilities based on given clues
* Propagate immediate constraints (eliminate values that conflict with filled cells)
* Validate the initial board state

### Phase 3: Solver Strategies
Research and implement sudoku solving techniques in order of complexity:
* **Naked Singles**: Cells with only one possible value
* **Hidden Singles**: Values that can only go in one place in a row/column/box
* **Pointing Pairs**: Eliminate candidates using box-row/box-column intersections
* **Box/Line Reduction**: Remove candidates from boxes using row/column constraints
* **X-Wing**: Identify rectangular patterns where a value appears exactly twice in two rows/columns, allowing elimination from other cells in those columns/rows
* **Swordfish**: Generalization of X-Wing - identify patterns where a value appears in exactly three rows (in exactly three columns), allowing elimination from other cells
* Backtracking as a fallback if logical deduction stalls

### Phase 4: Solver Implementation
* Implement a solve loop that:
  - Applies logical deduction strategies iteratively
  - Tracks and displays the next valid move
  - Continues until the sudoku is solved or no progress can be made
* Output each step of the solution process
* Verify the completed sudoku is valid
