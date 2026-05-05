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
* ✓ **Naked Singles**: Cells with only one possible value
* ✓ **Hidden Singles**: Values that can only go in one place in a row/column/box
* ✓ **Pointing Pairs**: Eliminate candidates using box-row/box-column intersections
* ✓ **Box/Line Reduction**: Remove candidates from boxes using row/column constraints
* ✓ **X-Wing**: Identify rectangular patterns where a value appears exactly twice in two rows/columns, allowing elimination from other cells in those columns/rows
* ✓ **Swordfish**: Generalization of X-Wing - identify patterns where a value appears in exactly 2-3 candidates across exactly 3 rows/columns, allowing elimination from other cells
* **Backtracking**: Fallback strategy for puzzles that logic alone cannot solve

### Phase 4: Solver Implementation
* Implement a solve loop that:
  - Applies logical deduction strategies iteratively
  - Tracks and displays the next valid move
  - Continues until the sudoku is solved or no progress can be made
* Output each step of the solution process
* Verify the completed sudoku is valid

## Current Status

### Implementation Complete
- ✓ Core data structures with u16 bitmasks for candidates
- ✓ Unified constraint matrix [CellMask; 27] for efficient access
- ✓ All major solver strategies: Naked Singles, Hidden Singles, Pointing Pairs, Box/Line Reduction, X-Wing, and Swordfish
- ✓ Deduplication with u128 cell bitmasks
- ✓ Compilation and test suite working

### Current Limitations
Vincent's Challenge puzzle currently solves to only **1 cell** with the implemented logical strategies, indicating it requires:
- More advanced techniques (e.g., advanced fish patterns like Jellyfish)
- Backtracking algorithm for forced sequences
- Deeper constraint propagation

### Next Steps
1. **Backtracking Implementation**: Add recursive backtracking with constraint propagation to handle unsolvable-by-logic puzzles
2. **Testing on Various Puzzles**: Test solver on multiple puzzle difficulties to validate strategies
3. **Performance Optimization**: Profile and optimize hot paths in solver if needed

## Implementation Notes
- Candidate representation: u16 bitmask (bit 0 unused, bits 1-9 for digits 1-9, ALL_POSSIBLE = 0x3FE)
- Deduplication: u128 bitmask with one bit per cell (index = row*9+col) for O(1) lookup
- Constraint matrix: [CellMask; 27] with indices 0-8 (rows), 9-17 (columns), 18-26 (boxes)
