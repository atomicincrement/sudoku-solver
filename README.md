# Sudoku Solver

A high-performance constraint propagation solver for sudoku puzzles, implemented in Rust. This solver uses advanced logical deduction techniques to solve even extremely difficult puzzles without backtracking.

## The Challenge

This project solves **Vincent's Challenge**, a 43-clue sudoku puzzle from LinkedIn that requires sophisticated constraint propagation techniques:

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

## Getting Started

### Prerequisites
- Rust 1.70 or later

### Build and Run

```bash
cargo build --release
cargo run --release
```

The solver will load Vincent's Challenge and display the solution:

```
7 3 6 | 1 5 4 | 8 9 2 
2 5 4 | 9 7 8 | 6 3 1 
9 1 8 | 3 6 2 | 4 5 7 
------+-------+------
5 7 1 | 6 4 3 | 2 8 9 
8 2 3 | 7 9 5 | 1 6 4 
4 6 9 | 8 2 1 | 3 7 5 
------+-------+------
6 4 5 | 2 8 7 | 9 1 3 
3 9 2 | 5 1 6 | 7 4 8 
1 8 7 | 4 3 9 | 5 2 6 
```

## Architecture

### Data Structures

#### Cell Representation
Each cell uses a `u16` bitmask to efficiently represent the set of possible values (1-9):
- Bit 0: Unused
- Bits 1-9: Represent whether values 1-9 are possible
- `0x3FE` (binary: `1111111110`) = All 9 values possible

Each `Cell` contains:
- `mask: u16` - Bitmask of possible values
- `filled_value: Option<u8>` - Set only when the cell is explicitly filled (critical for distinguishing filled cells from naked singles)

#### Constraint Matrix
A unified `[CellMask; 27]` array efficiently tracks constraints:
- Indices 0-8: Row constraints
- Indices 9-17: Column constraints  
- Indices 18-26: Box constraints

This enables O(1) lookups for "what values are already used in row X?"

#### Board
The main `Board` struct contains:
- 9×9 grid of `Cell` objects
- Unified `[CellMask; 27]` constraint matrix
- Helper methods for accessing rows, columns, and 3×3 boxes

### The `filled_value` Innovation

A critical architectural insight resolved the constraint propagation bug:

**Problem**: When constraint elimination left a cell with exactly 1 candidate, the solver couldn't distinguish it from explicitly filled cells. This broke cascading propagation.

**Example**: Cell (8,1) reduced from [1,8] → [8], but wasn't actually "filled" in the data structure. The value 8 wasn't eliminated from peers, leaving invalid state.

**Solution**: The `filled_value: Option<u8>` field:
- `filled_value = Some(v)` ← Only set by `set_cell()` which propagates constraints
- `filled_value = None` ← Cells with candidates (including naked singles)
- `is_filled()` checks `filled_value`, not mask count
- `get_value()` returns `filled_value` first, then checks if mask has exactly 1 bit

This separation enables:
1. **Correct naked single detection**: Find cells with 1 candidate but `!filled_value`
2. **Automatic propagation**: Only `set_cell()` propagates; found singles are filled via `set_cell()`
3. **Eliminated ghost candidates**: All value eliminations properly cascade through peers

## Solving Strategies

The solver applies logical deduction techniques in this order:

### 1. **Pointing Pairs** (Box-Line Reduction)
If a candidate value in a box appears only within one row (or column) of that box, eliminate it from that row/column outside the box.

**Example**: If 7 appears only in row 3 within box 1, remove 7 from all other cells in row 3.

### 2. **Box/Line Reduction** (Converse Pointing Pairs)
If a candidate value appears only within one box along a row (or column), eliminate it from other cells in that row/column outside the box.

**Example**: If 5 appears only in box 5 along row 5, remove 5 from the rest of row 5.

### 3. **X-Wing**
Identifies rectangular patterns where a candidate appears in exactly 2 rows and 2 columns. When this pattern exists:
- If the corners form a rectangle, eliminate the candidate from other cells in those rows/columns

**Pattern**: 
```
Row A: _ X _ _ X _
Row B: _ _ _ _ _ X (candidate X at positions (A,col1), (A,col2), (B,col2))
```

When all 4 corners contain the candidate, eliminate from other cells in cols 1-2, rows A-B.

### 4. **Swordfish**
Generalization of X-Wing to 3 rows/columns. Identifies patterns where a candidate appears in exactly 2-3 instances across exactly 3 rows and 3 columns.

**Pattern Example**:
```
Row 1: X _ X _
Row 3: X _ X _
Row 7: _ _ _ X
Cols:  1 2 3 4
```

When all row-column intersections form the pattern, eliminate the candidate from other cells in those rows/columns.

### 5. **Naked Singles**
Find cells with exactly one possible value and fill them. Automatically propagates constraints to eliminate that value from all peers (same row, column, and 3×3 box).

### 6. **Hidden Singles**
Find values that can only go in one cell within a row, column, or box, then fill that cell.

## Algorithm Flow

The main solver loop (`solve_one_iteration`):

1. Apply constraint propagation strategies (Pointing Pairs, Box/Line Reduction, X-Wing, Swordfish)
2. Find all naked singles (cells with 1 candidate)
3. Find all hidden singles (values fitting in 1 cell)
4. Fill found singles using `set_cell()`, which propagates constraints to peers
5. Repeat until puzzle is solved or no progress is made

## Results

**Vincent's Challenge**: Solved in **42 moves over 8 iterations**

- Pure logical deduction (no guessing/backtracking)
- All 81 cells correctly filled
- Constraint propagation validated with zero violations
- Demonstrates the power of proper constraint cascading

### Iteration Breakdown
```
Iteration 1: 2 cells filled
Iteration 2: 2 cells filled
Iteration 3: 7 cells filled
Iteration 4: 1 cell filled
Iteration 5: 6 cells filled
Iteration 6: 10 cells filled
Iteration 7: 10 cells filled
Iteration 8: 4 cells filled
Total: 42 cells filled
```

## Performance Characteristics

- **Time Complexity**: O(n) iterations per strategy (where n = number of cells = 81)
- **Space Complexity**: O(1) with fixed 9×9 board and 27-element constraint matrix
- **Bit Operations**: Bitmasks enable O(1) candidate checking and elimination
- **Propagation**: Constraint propagation cascades automatically via `set_cell()`

## Implementation Details

### Key Functions

- `Board::set_cell(row, col, value)`: Fills a cell and propagates constraints
- `Board::solve_one_iteration()`: Applies all strategies and fills singles
- `Board::apply_pointing_pairs()`: Implements pointing pairs logic
- `Board::apply_xwing()`: Identifies X-Wing patterns
- `Cell::eliminate(value)`: Removes a candidate from a cell's possibilities
- `Cell::get_value()`: Returns the cell's value (handling both filled and naked singles)

### Constraint Propagation

When a cell is filled via `set_cell()`:
1. The value is eliminated from all row peers
2. The value is eliminated from all column peers
3. The value is eliminated from all box peers
4. Next iteration applies strategies on updated state

This cascading effect is why the `filled_value` field is critical—it ensures `set_cell()` is the only path for propagation.

## Potential Enhancements

- **More Strategies**: Implement jellyfish, skyscraper, or other advanced techniques
- **Backtracking**: Add guess-and-check for unsolvable-by-logic puzzles
- **Performance Optimization**: Cache constraint updates or use SIMD for bit operations
- **Web Interface**: Build an interactive solver with visualization
- **Difficulty Analysis**: Measure puzzle difficulty by required strategy level

## License

This project is provided as-is for educational and puzzle-solving purposes.
