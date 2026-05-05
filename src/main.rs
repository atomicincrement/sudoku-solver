// Phase 1: Core Data Structure for Sudoku Solver
// Using u16 bit sets to represent possible values in each cell

use std::fmt;

/// Each bit represents a possible digit (1-9)
/// Bit 0 is unused, bits 1-9 represent digits 1-9
/// All 9 bits set (0x3FE) = 511 = all possibilities
type CellMask = u16;

const ALL_POSSIBLE: CellMask = 0x3FE; // Binary: 1111111110 (bits 1-9 set)

/// Represents a single cell on the board
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cell {
    mask: CellMask, // Bitmask of possible values
}

impl Cell {
    /// Create a new empty cell with all possibilities
    fn new_empty() -> Self {
        Cell { mask: ALL_POSSIBLE }
    }

    /// Create a cell with a single fixed value (1-9)
    fn new_fixed(value: u8) -> Self {
        if value < 1 || value > 9 {
            panic!("Invalid cell value: {}", value);
        }
        Cell {
            mask: 1 << value,
        }
    }

    /// Check if a value is possible in this cell (1-9)
    fn is_possible(&self, value: u8) -> bool {
        if value < 1 || value > 9 {
            return false;
        }
        (self.mask & (1 << value)) != 0
    }

    /// Eliminate a value from this cell's possibilities
    fn eliminate(&mut self, value: u8) {
        if value >= 1 && value <= 9 {
            self.mask &= !(1 << value);
        }
    }

    /// Get the fixed value if this cell has exactly one possibility
    fn get_value(&self) -> Option<u8> {
        // Check if exactly one bit is set
        if self.mask.count_ones() == 1 {
            for i in 1..=9 {
                if (self.mask & (1 << i)) != 0 {
                    return Some(i as u8);
                }
            }
        }
        None
    }

    /// Check if this cell is filled (has exactly one possibility)
    fn is_filled(&self) -> bool {
        self.get_value().is_some()
    }

    /// Get all possible values as a vector
    fn possibilities(&self) -> Vec<u8> {
        (1..=9).filter(|&v| self.is_possible(v)).collect()
    }

    /// Count the number of possibilities
    fn count_possibilities(&self) -> u32 {
        self.mask.count_ones()
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.get_value() {
            Some(v) => write!(f, "{}", v),
            None => write!(f, "."),
        }
    }
}

/// The main Sudoku board structure
/// Uses bit sets for cell values and constraint tracking
struct Board {
    cells: [[Cell; 9]; 9],
    row_constraints: [CellMask; 9],    // Which values are possible in each row
    col_constraints: [CellMask; 9],    // Which values are possible in each column
    box_constraints: [CellMask; 9],    // Which values are possible in each 3x3 box
}

impl Board {
    /// Create a new empty board with all cells and constraints initialized
    fn new() -> Self {
        Board {
            cells: [[Cell::new_empty(); 9]; 9],
            row_constraints: [ALL_POSSIBLE; 9],
            col_constraints: [ALL_POSSIBLE; 9],
            box_constraints: [ALL_POSSIBLE; 9],
        }
    }

    /// Set a cell to a fixed value and propagate constraints
    /// Eliminates the value from all cells in the same row, column, and box
    /// Returns true if valid, false if conflict detected
    fn set_cell(&mut self, row: usize, col: usize, value: u8) -> bool {
        if row >= 9 || col >= 9 || value < 1 || value > 9 {
            return false;
        }

        let box_idx = (row / 3) * 3 + (col / 3);
        self.cells[row][col] = Cell::new_fixed(value);

        // Update constraints by removing this value from possibilities
        let value_bit = 1 << value;
        self.row_constraints[row] &= !value_bit;
        self.col_constraints[col] &= !value_bit;
        self.box_constraints[box_idx] &= !value_bit;

        // Eliminate this value from all cells in the same row
        for c in 0..9 {
            if c != col {
                self.cells[row][c].eliminate(value);
            }
        }

        // Eliminate this value from all cells in the same column
        for r in 0..9 {
            if r != row {
                self.cells[r][col].eliminate(value);
            }
        }

        // Eliminate this value from all cells in the same box
        let box_cells = Self::get_box_cells(box_idx);
        for (r, c) in box_cells {
            if (r, c) != (row, col) {
                self.cells[r][c].eliminate(value);
            }
        }

        true
    }

    /// Get the box index (0-8) for a given row and column
    #[allow(dead_code)]
    fn get_box_index(row: usize, col: usize) -> usize {
        (row / 3) * 3 + (col / 3)
    }

    /// Get all cells in a box as a vector of (row, col) indices
    fn get_box_cells(box_idx: usize) -> Vec<(usize, usize)> {
        let start_row = (box_idx / 3) * 3;
        let start_col = (box_idx % 3) * 3;
        let mut cells = Vec::new();

        for row in start_row..start_row + 3 {
            for col in start_col..start_col + 3 {
                cells.push((row, col));
            }
        }
        cells
    }

    /// Get a cell value
    fn get_cell(&self, row: usize, col: usize) -> Option<u8> {
        if row < 9 && col < 9 {
            self.cells[row][col].get_value()
        } else {
            None
        }
    }

    /// Check if the board is completely solved
    fn is_solved(&self) -> bool {
        self.cells.iter().all(|row| row.iter().all(|cell| cell.is_filled()))
    }

    /// Check for duplicate values in a row
    fn has_row_conflicts(&self, row: usize) -> bool {
        let mut seen = 0u16;
        for col in 0..9 {
            if let Some(value) = self.cells[row][col].get_value() {
                let bit = 1 << value;
                if (seen & bit) != 0 {
                    return true; // Duplicate found
                }
                seen |= bit;
            }
        }
        false
    }

    /// Check for duplicate values in a column
    fn has_col_conflicts(&self, col: usize) -> bool {
        let mut seen = 0u16;
        for row in 0..9 {
            if let Some(value) = self.cells[row][col].get_value() {
                let bit = 1 << value;
                if (seen & bit) != 0 {
                    return true; // Duplicate found
                }
                seen |= bit;
            }
        }
        false
    }

    /// Check for duplicate values in a box
    fn has_box_conflicts(&self, box_idx: usize) -> bool {
        let mut seen = 0u16;
        let box_cells = Self::get_box_cells(box_idx);
        for (r, c) in box_cells {
            if let Some(value) = self.cells[r][c].get_value() {
                let bit = 1 << value;
                if (seen & bit) != 0 {
                    return true; // Duplicate found
                }
                seen |= bit;
            }
        }
        false
    }

    /// Check if the board is valid (no conflicts)
    fn is_valid(&self) -> bool {
        // All cells must have at least one possibility
        if self.cells.iter().any(|row| row.iter().any(|cell| cell.count_possibilities() == 0)) {
            return false;
        }

        // Check for duplicate values in each row
        for row in 0..9 {
            if self.has_row_conflicts(row) {
                return false;
            }
        }

        // Check for duplicate values in each column
        for col in 0..9 {
            if self.has_col_conflicts(col) {
                return false;
            }
        }

        // Check for duplicate values in each box
        for box_idx in 0..9 {
            if self.has_box_conflicts(box_idx) {
                return false;
            }
        }

        true
    }

    /// Phase 3: Find all naked singles
    /// A naked single is a cell with exactly one possible value
    /// Returns a vector of (row, col, value) tuples for each naked single found
    fn find_naked_singles(&self) -> Vec<(usize, usize, u8)> {
        let mut singles = Vec::new();

        for row in 0..9 {
            for col in 0..9 {
                // Skip already filled cells
                if !self.cells[row][col].is_filled() {
                    // Check if this cell has exactly one possibility
                    if self.cells[row][col].count_possibilities() == 1 {
                        if let Some(value) = self.cells[row][col].get_value() {
                            singles.push((row, col, value));
                        }
                    }
                }
            }
        }

        singles
    }

    /// Find hidden singles in a specific row
    /// Returns (row, col, value) for each hidden single found
    fn find_hidden_singles_in_row(&self, row: usize) -> Vec<(usize, usize, u8)> {
        let mut result = Vec::new();

        // For each value 1-9, find where it can go in this row
        for value in 1..=9 {
            let mut possible_cols = Vec::new();
            let mut already_placed = false;

            for col in 0..9 {
                // Skip filled cells
                if self.cells[row][col].is_filled() {
                    // If it's already filled with this value, mark it and skip
                    if let Some(v) = self.cells[row][col].get_value() {
                        if v == value {
                            already_placed = true;
                            break;
                        }
                    }
                } else if self.cells[row][col].is_possible(value) {
                    possible_cols.push(col);
                }
            }

            // If value can only go in one place and isn't already placed, it's a hidden single
            if !already_placed && possible_cols.len() == 1 {
                result.push((row, possible_cols[0], value));
            }
        }

        result
    }

    /// Find hidden singles in a specific column
    /// Returns (row, col, value) for each hidden single found
    fn find_hidden_singles_in_col(&self, col: usize) -> Vec<(usize, usize, u8)> {
        let mut result = Vec::new();

        for value in 1..=9 {
            let mut possible_rows = Vec::new();
            let mut already_placed = false;

            for row in 0..9 {
                if self.cells[row][col].is_filled() {
                    if let Some(v) = self.cells[row][col].get_value() {
                        if v == value {
                            already_placed = true;
                            break;
                        }
                    }
                } else if self.cells[row][col].is_possible(value) {
                    possible_rows.push(row);
                }
            }

            if !already_placed && possible_rows.len() == 1 {
                result.push((possible_rows[0], col, value));
            }
        }

        result
    }

    /// Find hidden singles in a specific box
    /// Returns (row, col, value) for each hidden single found
    fn find_hidden_singles_in_box(&self, box_idx: usize) -> Vec<(usize, usize, u8)> {
        let mut result = Vec::new();
        let box_cells = Self::get_box_cells(box_idx);

        for value in 1..=9 {
            let mut possible_cells = Vec::new();
            let mut already_placed = false;

            for (r, c) in &box_cells {
                if self.cells[*r][*c].is_filled() {
                    if let Some(v) = self.cells[*r][*c].get_value() {
                        if v == value {
                            already_placed = true;
                            break;
                        }
                    }
                } else if self.cells[*r][*c].is_possible(value) {
                    possible_cells.push((*r, *c));
                }
            }

            if !already_placed && possible_cells.len() == 1 {
                let (r, c) = possible_cells[0];
                result.push((r, c, value));
            }
        }

        result
    }

    /// Find all hidden singles in the board
    /// Returns a vector of (row, col, value) tuples
    fn find_hidden_singles(&self) -> Vec<(usize, usize, u8)> {
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Check all rows
        for row in 0..9 {
            for (r, c, v) in self.find_hidden_singles_in_row(row) {
                let key = (r, c);
                if !seen.contains(&key) {
                    result.push((r, c, v));
                    seen.insert(key);
                }
            }
        }

        // Check all columns
        for col in 0..9 {
            for (r, c, v) in self.find_hidden_singles_in_col(col) {
                let key = (r, c);
                if !seen.contains(&key) {
                    result.push((r, c, v));
                    seen.insert(key);
                }
            }
        }

        // Check all boxes
        for box_idx in 0..9 {
            for (r, c, v) in self.find_hidden_singles_in_box(box_idx) {
                let key = (r, c);
                if !seen.contains(&key) {
                    result.push((r, c, v));
                    seen.insert(key);
                }
            }
        }

        result
    }

    /// Apply pointing pairs/box-line reduction
    /// If a value in a box appears only in one row/column,
    /// eliminate it from that row/column outside the box
    /// Returns the number of candidates eliminated
    fn apply_pointing_pairs(&mut self) -> usize {
        let mut eliminated = 0;

        // For each box
        for box_idx in 0..9 {
            let box_cells = Self::get_box_cells(box_idx);
            let box_start_row = (box_idx / 3) * 3;
            let box_start_col = (box_idx % 3) * 3;

            // For each value 1-9
            for value in 1..=9 {
                let mut rows_with_value = std::collections::HashSet::new();
                let mut cols_with_value = std::collections::HashSet::new();
                let mut found_unfilled = false;

                // Find which rows and columns in this box can have this value
                for (r, c) in &box_cells {
                    if !self.cells[*r][*c].is_filled() && self.cells[*r][*c].is_possible(value) {
                        rows_with_value.insert(*r);
                        cols_with_value.insert(*c);
                        found_unfilled = true;
                    }
                }

                // If value appears in only one row within this box
                if found_unfilled && rows_with_value.len() == 1 {
                    let target_row = *rows_with_value.iter().next().unwrap();
                    // Eliminate this value from rest of row outside this box
                    for col in 0..9 {
                        if col < box_start_col || col >= box_start_col + 3 {
                            if self.cells[target_row][col].is_possible(value) {
                                self.cells[target_row][col].eliminate(value);
                                eliminated += 1;
                            }
                        }
                    }
                }

                // If value appears in only one column within this box
                if found_unfilled && cols_with_value.len() == 1 {
                    let target_col = *cols_with_value.iter().next().unwrap();
                    // Eliminate this value from rest of column outside this box
                    for row in 0..9 {
                        if row < box_start_row || row >= box_start_row + 3 {
                            if self.cells[row][target_col].is_possible(value) {
                                self.cells[row][target_col].eliminate(value);
                                eliminated += 1;
                            }
                        }
                    }
                }
            }
        }

        eliminated
    }

    /// Phase 4: Solver loop - perform one iteration using naked and hidden singles
    /// Finds all naked and hidden singles and fills them in
    /// Returns the number of cells filled in this iteration
    fn solve_one_iteration(&mut self) -> usize {
        // First, apply constraint propagation techniques
        self.apply_pointing_pairs();
        
        let mut all_singles = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Find naked singles
        for (r, c, v) in self.find_naked_singles() {
            let key = (r, c);
            if !seen.contains(&key) {
                all_singles.push((r, c, v));
                seen.insert(key);
            }
        }

        // Find hidden singles
        for (r, c, v) in self.find_hidden_singles() {
            let key = (r, c);
            if !seen.contains(&key) {
                all_singles.push((r, c, v));
                seen.insert(key);
            }
        }

        let count = all_singles.len();

        for (row, col, value) in all_singles {
            self.set_cell(row, col, value);
        }

        // Return the number of cells filled (not candidates eliminated)
        count
    }

    /// Solve using naked and hidden singles strategies
    /// Applies strategies repeatedly until no more moves can be made
    /// Returns the total number of cells filled
    fn solve_with_singles(&mut self) -> usize {
        let mut total_moves = 0;

        loop {
            let moves = self.solve_one_iteration();
            total_moves += moves;

            if moves == 0 {
                break; // No more progress
            }
        }

        total_moves
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (row_idx, row) in self.cells.iter().enumerate() {
            if row_idx > 0 && row_idx % 3 == 0 {
                writeln!(f, "------+-------+------")?;
            }

            for (col_idx, cell) in row.iter().enumerate() {
                if col_idx > 0 && col_idx % 3 == 0 {
                    write!(f, "| ")?;
                }
                write!(f, "{} ", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn main() {
    // Phase 2: Board Initialization with Constraint Propagation
    let mut board = Board::new();

    // Vincent's challenge - use _ for empty cells
    let vincents_challenge = "73___4___
2___7__3_
9__3__4__
571643289
8237__164
469821_7_
__5__7___
_9__1___8
___4____6";

    println!("=== Phase 2: Board Initialization ===\n");
    println!("Loading Vincent's Challenge puzzle:");
    println!("{}\n", vincents_challenge);

    // Parse and initialize the board
    let mut row = 0;
    for line in vincents_challenge.lines() {
        for (col, ch) in line.chars().enumerate() {
            if ch.is_digit(10) {
                let value = ch.to_digit(10).unwrap() as u8;
                board.set_cell(row, col, value);
            }
        }
        row += 1;
    }

    println!("Board after initialization:");
    println!("{}", board);

    println!("Board validation:");
    println!("  Valid: {}", board.is_valid());
    println!("  Solved: {}", board.is_solved());

    println!("\nCell possibilities after constraint propagation:");
    // Display a few example cells and their possibilities
    for (row, col) in &[(0, 2), (0, 3), (1, 1), (7, 0)] {
        if let Some(value) = board.get_cell(*row, *col) {
            println!("  Cell ({}, {}) = {} (fixed)", row, col, value);
        } else {
            let possibilities = board.cells[*row][*col].possibilities();
            println!("  Cell ({}, {}) = {:?}", row, col, possibilities);
        }
    }

    println!("\nConstraint tracking:");
    println!("  Row 0 possible values: {:?}", 
             (1..=9).filter(|v| (board.row_constraints[0] & (1 << v)) != 0).collect::<Vec<_>>());
    println!("  Col 0 possible values: {:?}",
             (1..=9).filter(|v| (board.col_constraints[0] & (1 << v)) != 0).collect::<Vec<_>>());
    println!("  Box 0 possible values: {:?}",
             (1..=9).filter(|v| (board.box_constraints[0] & (1 << v)) != 0).collect::<Vec<_>>());

    // Phase 3 & 4: Solver Loop with Naked Singles
    println!("\n=== Phase 3 & 4: Solver Loop - Naked Singles ===\n");
    
    // First, show initial state - no naked singles
    println!("Step 0 - Initial state:");
    let initial_singles = board.find_naked_singles();
    println!("Found {} naked singles", initial_singles.len());
    
    // Apply first iteration
    println!("\nStep 1 - Applying first iteration of naked and hidden singles:");
    
    let naked_before = board.find_naked_singles();
    let hidden_before = board.find_hidden_singles();
    
    println!("Found {} naked singles, {} hidden singles", naked_before.len(), hidden_before.len());
    
    if !hidden_before.is_empty() {
        println!("Hidden singles found:");
        for (r, c, v) in hidden_before.iter().take(5) {
            println!("  Cell ({}, {}) = {} (hidden single)", r, c, v);
        }
        if hidden_before.len() > 5 {
            println!("  ... and {} more", hidden_before.len() - 5);
        }
    }
    
    let moves_step_1 = board.solve_one_iteration();
    println!("Filled {} cells total", moves_step_1);
    
    if moves_step_1 > 0 {
        println!("\nBoard after step 1:");
        println!("{}", board);
        
        println!("Board validation:");
        println!("  Valid: {}", board.is_valid());
        println!("  Solved: {}", board.is_solved());
        
        // Check for naked singles after step 1
        let step_1_singles = board.find_naked_singles();
        println!("\nFound {} naked singles after step 1", step_1_singles.len());
        if !step_1_singles.is_empty() {
            println!("Next naked singles to fill:");
            for (r, c, v) in step_1_singles.iter().take(5) {
                println!("  Cell ({}, {}) = {}", r, c, v);
            }
            if step_1_singles.len() > 5 {
                println!("  ... and {} more", step_1_singles.len() - 5);
            }
        }
    }
    
    // Test pointing pairs in the center box
    println!("\n=== Testing Pointing Pairs - Center Box (Box 4) ===");
    
    // Show center box before pointing pairs
    println!("\nCenter box cells and their possibilities:");
    let center_box_cells = Board::get_box_cells(4);
    for (r, c) in center_box_cells.iter() {
        let poss = board.cells[*r][*c].possibilities();
        if let Some(v) = board.cells[*r][*c].get_value() {
            println!("  Cell ({}, {}) = {} (filled)", r, c, v);
        } else {
            println!("  Cell ({}, {}) = {:?}", r, c, poss);
        }
    }
    
    println!("\nLooking for 5 in center box:");
    let mut cells_with_5 = Vec::new();
    for (r, c) in &center_box_cells {
        if !board.cells[*r][*c].is_filled() && board.cells[*r][*c].is_possible(5) {
            cells_with_5.push((*r, *c));
        }
    }
    println!("  5 can be placed in {} cells", cells_with_5.len());
    for (r, c) in cells_with_5 {
        println!("    - Cell ({}, {})", r, c);
    }
    
    println!("\nLooking for 9 in center box:");
    let mut cells_with_9 = Vec::new();
    for (r, c) in &center_box_cells {
        if !board.cells[*r][*c].is_filled() && board.cells[*r][*c].is_possible(9) {
            cells_with_9.push((*r, *c));
        }
    }
    println!("  9 can be placed in {} cells", cells_with_9.len());
    for (r, c) in cells_with_9 {
        println!("    - Cell ({}, {})", r, c);
    }
    
    // Apply pointing pairs specifically
    println!("\nApplying pointing pairs...");
    let mut test_board = board.clone();
    let eliminated = test_board.apply_pointing_pairs();
    println!("Eliminated {} candidates", eliminated);
    
    println!("\nCenter box after pointing pairs:");
    let mut cells_with_5_after = Vec::new();
    let mut cells_with_9_after = Vec::new();
    for (r, c) in &center_box_cells {
        if !test_board.cells[*r][*c].is_filled() && test_board.cells[*r][*c].is_possible(5) {
            cells_with_5_after.push((*r, *c));
        }
        if !test_board.cells[*r][*c].is_filled() && test_board.cells[*r][*c].is_possible(9) {
            cells_with_9_after.push((*r, *c));
        }
    }
    println!("  5 can now be placed in {} cells", cells_with_5_after.len());
    for (r, c) in cells_with_5_after {
        println!("    - Cell ({}, {})", r, c);
    }
    println!("  9 can now be placed in {} cells", cells_with_9_after.len());
    for (r, c) in cells_with_9_after {
        println!("    - Cell ({}, {})", r, c);
    }
    
    // Try full solve with naked and hidden singles
    println!("\n=== Full Solve with Naked & Hidden Singles ===");
    let mut solve_board = board.clone();
    let total_moves = solve_board.solve_with_singles();
    println!("Total moves made: {}", total_moves);
    println!("Board solved: {}", solve_board.is_solved());
    println!("Board valid: {}", solve_board.is_valid());
    
    if solve_board.is_solved() {
        println!("\n✓ Puzzle Solved!");
        println!("{}", solve_board);
    } else {
        println!("\nBoard after solving with singles:");
        println!("{}", solve_board);
    }
}

impl Clone for Board {
    fn clone(&self) -> Self {
        Board {
            cells: self.cells,
            row_constraints: self.row_constraints,
            col_constraints: self.col_constraints,
            box_constraints: self.box_constraints,
        }
    }
}
