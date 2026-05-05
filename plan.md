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
- ✓ Constraint propagation validation (detects candidate violations)

### Bug Fixed: Constraint Propagation for Naked Singles
**Problem:** When a cell became a naked single through constraint elimination (not via set_cell), the value wasn't propagated to eliminate it from box/row/column peers. This left "ghost" candidates in peer cells.

**Example Case:** Cell (8,1) with candidates [1,8] - when another constraint eliminated candidate 1, the cell became filled with 8. But cells (6,1) and (8,2) in box 6 retained 8 as a candidate, violating sudoku constraints.

**Solution:** Added guard in solve_one_iteration to ensure set_cell is called only once per cell, triggering proper constraint propagation for all newly-filled cells.

### Current Limitations
Vincent's Challenge puzzle currently solves to only **1 cell** with the implemented logical strategies, indicating it requires:
- More advanced techniques (e.g., advanced fish patterns like Jellyfish)
- Backtracking algorithm for forced sequences

### Next Steps
1. **Backtracking Implementation**: Add recursive backtracking with constraint propagation to handle unsolvable-by-logic puzzles
2. **Testing on Various Puzzles**: Test solver on multiple puzzle difficulties to validate strategies
3. **Performance Optimization**: Profile and optimize hot paths in solver if needed

## Implementation Notes
- Candidate representation: u16 bitmask (bit 0 unused, bits 1-9 for digits 1-9, ALL_POSSIBLE = 0x3FE)
- Deduplication: u128 bitmask with one bit per cell (index = row*9+col) for O(1) lookup
- Constraint matrix: [CellMask; 27] with indices 0-8 (rows), 9-17 (columns), 18-26 (boxes)


## Case examples for debugging:

### 1

In this case in the bottom left square we have already solved 8 and 9 but there are still
some 8's in the possible sets.

Cells with 3 or fewer candidates:
```
  7   3  68 |   .   .   4 |   .   . 125
  2   . 468 | 159   7   . |   .   3  15
  9 158  68 |   3 568   . |   4 125   7
---------+---------+---------
  5   7   1 |   6   4   3 |   2   8   9
  8   2   3 |   7  59  59 |   1   6   4
  4   6   9 |   8   2   1 |  35   7  35
---------+---------+---------
136  48   5 |  29   .   7 |  39   . 123
 36   9 247 |  25   1 256 | 357 245   8
 13   8 278 |   4   .   . |   .   .   6
```

### 2

**Issue:** Blank spaces in the candidate display mean the cell has more than 3 candidates (normally shown as `.` for 4+ candidates).

However, cells with exactly 1 candidate (naked singles not yet filled) should also be displayed, as should cells with 0 candidates (error state).

**Fix:** Updated `print_candidates()` to:
- Show `XX` for cells with 0 candidates (error state - sudoku violation)
- Always show cells with exactly 1 candidate (not just 2-3)
- Show `.` only for cells with 4+ candidates

This makes it easy to spot naked singles that haven't been filled yet (they now appear with their single digit) and error states (cells with no valid candidates).

