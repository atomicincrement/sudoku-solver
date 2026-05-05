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
    mask: CellMask,              // Bitmask of possible values
    filled_value: Option<u8>,    // Set only when cell has been explicitly filled (via set_cell)
}

impl Cell {
    /// Create a new empty cell with all possibilities
    fn new_empty() -> Self {
        Cell { mask: ALL_POSSIBLE, filled_value: None }
    }

    /// Create a cell with a single fixed value (1-9)
    fn new_fixed(value: u8) -> Self {
        if value < 1 || value > 9 {
            panic!("Invalid cell value: {}", value);
        }
        Cell {
            mask: 1 << value,
            filled_value: Some(value),
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

    /// Get the fixed value if this cell has been filled or has exactly one possibility
    fn get_value(&self) -> Option<u8> {
        if let Some(v) = self.filled_value {
            return Some(v);
        }
        // If not explicitly filled, check if mask has exactly one bit set
        if self.mask.count_ones() == 1 {
            let bit_pos = self.mask.trailing_zeros() as u8;
            if bit_pos >= 1 && bit_pos <= 9 {
                return Some(bit_pos);
            }
        }
        None
    }

    /// Check if this cell is filled (has been explicitly set to a value)
    fn is_filled(&self) -> bool {
        self.filled_value.is_some()
    }

    /// Get all possible values as a vector (in bitmask order)
    #[allow(dead_code)]
    fn possibilities(&self) -> Vec<u8> {
        if self.filled_value.is_some() {
            self.filled_value.map(|v| vec![v]).unwrap_or_default()
        } else {
            (1..=9).filter(|&v| self.is_possible(v)).collect()
        }
    }

    /// Count the number of possibilities
    fn count_possibilities(&self) -> u32 {
        if self.filled_value.is_some() {
            1
        } else {
            self.mask.count_ones()
        }
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
    constraints: [CellMask; 27],    // Unified constraint matrix:
                                    // [0-8]: rows, [9-17]: columns, [18-26]: boxes
}

impl Board {
    /// Create a new empty board with all cells and constraints initialized
    fn new() -> Self {
        Board {
            cells: [[Cell::new_empty(); 9]; 9],
            constraints: [ALL_POSSIBLE; 27],
        }
    }

    /// Initialize the board from a string representation
    /// Supports newlines and any non-digit characters as empty cells
    /// Digits 1-9 are placed on the board
    fn init_from_string(&mut self, puzzle: &str) {
        let mut row = 0;
        for line in puzzle.lines() {
            for (col, ch) in line.chars().enumerate() {
                if ch.is_digit(10) && ch != '0' {
                    let value = ch.to_digit(10).unwrap() as u8;
                    self.set_cell(row, col, value);
                }
            }
            row += 1;
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
        self.constraints[row] &= !value_bit;                    // row constraint
        self.constraints[9 + col] &= !value_bit;                // col constraint
        self.constraints[18 + box_idx] &= !value_bit;           // box constraint

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
    #[allow(dead_code)]
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
    /// A naked single is a cell with exactly one possible value that hasn't been explicitly filled yet
    /// Returns a vector of (row, col, value) tuples for each naked single found
    fn find_naked_singles(&self) -> Vec<(usize, usize, u8)> {
        let mut singles = Vec::new();

        for row in 0..9 {
            for col in 0..9 {
                // Find cells that have exactly 1 candidate but haven't been explicitly filled
                if !self.cells[row][col].is_filled() && self.cells[row][col].count_possibilities() == 1 {
                    if let Some(value) = self.cells[row][col].get_value() {
                        singles.push((row, col, value));
                    }
                }
            }
        }

        singles
    }

    /// Find hidden singles in a group of 9 cells (row, column, or box)
    /// Uses constraints matrix to skip values already placed in the group
    /// constraint_idx: index into constraints array (row, column, or box index)
    /// Returns (row, col, value) for each hidden single found
    fn find_hidden_singles_in_group(&self, cells: &[(usize, usize)], constraint_idx: usize) -> Vec<(usize, usize, u8)> {
        let mut result = Vec::new();
        let group_constraint = self.constraints[constraint_idx];

        // For each value 1-9, find where it can go in this group
        for value in 1..=9 {
            // Skip if value already placed in this group (bit not set in constraint)
            if (group_constraint & (1 << value)) == 0 {
                continue;
            }

            let mut count = 0;
            let mut last_cell = (0, 0);

            for (r, c) in cells {
                if !self.cells[*r][*c].is_filled() && self.cells[*r][*c].is_possible(value) {
                    count += 1;
                    last_cell = (*r, *c);
                    if count > 1 {
                        break; // Early exit - not a hidden single
                    }
                }
            }

            // If value appears in exactly one cell, it's a hidden single
            if count == 1 {
                result.push((last_cell.0, last_cell.1, value));
            }
        }

        result
    }

    /// Find hidden singles in a specific row
    /// Returns (row, col, value) for each hidden single found
    fn find_hidden_singles_in_row(&self, row: usize) -> Vec<(usize, usize, u8)> {
        let cells: Vec<_> = (0..9).map(|c| (row, c)).collect();
        self.find_hidden_singles_in_group(&cells, row)
    }

    /// Find hidden singles in a specific column
    /// Returns (row, col, value) for each hidden single found
    fn find_hidden_singles_in_col(&self, col: usize) -> Vec<(usize, usize, u8)> {
        let cells: Vec<_> = (0..9).map(|r| (r, col)).collect();
        self.find_hidden_singles_in_group(&cells, 9 + col)
    }

    /// Find hidden singles in a specific box
    /// Returns (row, col, value) for each hidden single found
    fn find_hidden_singles_in_box(&self, box_idx: usize) -> Vec<(usize, usize, u8)> {
        let cells = Self::get_box_cells(box_idx);
        self.find_hidden_singles_in_group(&cells, 18 + box_idx)
    }

    /// Find all hidden singles in the board
    /// Returns a vector of (row, col, value) tuples
    fn find_hidden_singles(&self) -> Vec<(usize, usize, u8)> {
        let mut result = Vec::new();
        let mut seen = 0u128; // Bitmask for 81 cells (row * 9 + col)

        // Check all rows
        for row in 0..9 {
            for (r, c, v) in self.find_hidden_singles_in_row(row) {
                let bit_index = r * 9 + c;
                if (seen & (1u128 << bit_index)) == 0 {
                    result.push((r, c, v));
                    seen |= 1u128 << bit_index;
                }
            }
        }

        // Check all columns
        for col in 0..9 {
            for (r, c, v) in self.find_hidden_singles_in_col(col) {
                let bit_index = r * 9 + c;
                if (seen & (1u128 << bit_index)) == 0 {
                    result.push((r, c, v));
                    seen |= 1u128 << bit_index;
                }
            }
        }

        // Check all boxes
        for box_idx in 0..9 {
            for (r, c, v) in self.find_hidden_singles_in_box(box_idx) {
                let bit_index = r * 9 + c;
                if (seen & (1u128 << bit_index)) == 0 {
                    result.push((r, c, v));
                    seen |= 1u128 << bit_index;
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
                let mut rows_with_value = 0u16; // Bitmask for rows 0-8
                let mut cols_with_value = 0u16; // Bitmask for cols 0-8
                let mut found_unfilled = false;

                // Find which rows and columns in this box can have this value
                for (r, c) in &box_cells {
                    if !self.cells[*r][*c].is_filled() && self.cells[*r][*c].is_possible(value) {
                        rows_with_value |= 1u16 << r;
                        cols_with_value |= 1u16 << c;
                        found_unfilled = true;
                    }
                }

                // If value appears in only one row within this box
                if found_unfilled && rows_with_value.count_ones() == 1 {
                    let target_row = rows_with_value.trailing_zeros() as usize;
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
                if found_unfilled && cols_with_value.count_ones() == 1 {
                    let target_col = cols_with_value.trailing_zeros() as usize;
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

    /// Apply box/line reduction
    /// If a value in a row/column appears only in one box,
    /// eliminate it from the rest of that box in that row/column
    /// Returns the number of candidates eliminated
    fn apply_box_line_reduction(&mut self) -> usize {
        let mut eliminated = 0;

        // For each row
        for row in 0..9 {
            let row_box_start = (row / 3) * 3;

            // For each value 1-9
            for value in 1..=9 {
                let mut boxes_with_value = 0u16; // Bitmask for boxes 0-2 (within this row band)
                let mut found_unfilled = false;

                // Find which boxes in this row can have this value
                for col in 0..9 {
                    if !self.cells[row][col].is_filled() && self.cells[row][col].is_possible(value) {
                        let box_col_idx = col / 3;
                        boxes_with_value |= 1u16 << box_col_idx;
                        found_unfilled = true;
                    }
                }

                // If value appears in only one box within this row
                if found_unfilled && boxes_with_value.count_ones() == 1 {
                    let target_box_col = boxes_with_value.trailing_zeros() as usize;
                    let box_col_start = target_box_col * 3;
                    let _box_idx = row_box_start / 3 * 3 + target_box_col;

                    // Eliminate from rest of box in different rows
                    for box_row_offset in 0..3 {
                        let box_row = row_box_start + box_row_offset;
                        if box_row != row {
                            for box_col in box_col_start..box_col_start + 3 {
                                if self.cells[box_row][box_col].is_possible(value) {
                                    self.cells[box_row][box_col].eliminate(value);
                                    eliminated += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // For each column
        for col in 0..9 {
            let col_box_start = (col / 3) * 3;

            // For each value 1-9
            for value in 1..=9 {
                let mut boxes_with_value = 0u16; // Bitmask for boxes 0-2 (within this column band)
                let mut found_unfilled = false;

                // Find which boxes in this column can have this value
                for row in 0..9 {
                    if !self.cells[row][col].is_filled() && self.cells[row][col].is_possible(value) {
                        let box_row_idx = row / 3;
                        boxes_with_value |= 1u16 << box_row_idx;
                        found_unfilled = true;
                    }
                }

                // If value appears in only one box within this column
                if found_unfilled && boxes_with_value.count_ones() == 1 {
                    let target_box_row = boxes_with_value.trailing_zeros() as usize;
                    let box_row_start = target_box_row * 3;
                    let _box_idx = target_box_row * 3 + col_box_start / 3;

                    // Eliminate from rest of box in different columns
                    for box_col_offset in 0..3 {
                        let box_col = col_box_start + box_col_offset;
                        if box_col != col {
                            for box_row in box_row_start..box_row_start + 3 {
                                if self.cells[box_row][box_col].is_possible(value) {
                                    self.cells[box_row][box_col].eliminate(value);
                                    eliminated += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        eliminated
    }

    /// Apply X-Wing strategy
    /// Find a value appearing exactly twice in each of two rows (or columns)
    /// in the same two columns (or rows), then eliminate from rest of those columns/rows
    /// Returns the number of candidates eliminated
    fn apply_xwing(&mut self) -> usize {
        let mut eliminated = 0;

        // Check rows for X-Wing pattern
        for value in 1..=9 {
            let mut row_col_pairs: Vec<(usize, u16)> = Vec::new();

            // For each row, find columns where this value appears as candidate
            for row in 0..9 {
                let mut cols_mask = 0u16;
                for col in 0..9 {
                    if !self.cells[row][col].is_filled() && self.cells[row][col].is_possible(value) {
                        cols_mask |= 1u16 << col;
                    }
                }

                // Only interested in rows with exactly 2 candidates for this value
                if cols_mask.count_ones() == 2 {
                    row_col_pairs.push((row, cols_mask));
                }
            }

            // Look for two rows with same column pattern
            for i in 0..row_col_pairs.len() {
                for j in (i + 1)..row_col_pairs.len() {
                    if row_col_pairs[i].1 == row_col_pairs[j].1 {
                        // Found X-Wing pattern: value appears in same 2 columns in rows i and j
                        let row1 = row_col_pairs[i].0;
                        let row2 = row_col_pairs[j].0;
                        let cols_mask = row_col_pairs[i].1;

                        // Extract the two column indices
                        let col1 = cols_mask.trailing_zeros() as usize;
                        let col2 = (cols_mask & !(1u16 << col1)).trailing_zeros() as usize;

                        // Eliminate value from other rows in these columns
                        for row in 0..9 {
                            if row != row1 && row != row2 {
                                if self.cells[row][col1].is_possible(value) {
                                    self.cells[row][col1].eliminate(value);
                                    eliminated += 1;
                                }
                                if self.cells[row][col2].is_possible(value) {
                                    self.cells[row][col2].eliminate(value);
                                    eliminated += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check columns for X-Wing pattern
        for value in 1..=9 {
            let mut col_row_pairs: Vec<(usize, u16)> = Vec::new();

            // For each column, find rows where this value appears as candidate
            for col in 0..9 {
                let mut rows_mask = 0u16;
                for row in 0..9 {
                    if !self.cells[row][col].is_filled() && self.cells[row][col].is_possible(value) {
                        rows_mask |= 1u16 << row;
                    }
                }

                // Only interested in columns with exactly 2 candidates for this value
                if rows_mask.count_ones() == 2 {
                    col_row_pairs.push((col, rows_mask));
                }
            }

            // Look for two columns with same row pattern
            for i in 0..col_row_pairs.len() {
                for j in (i + 1)..col_row_pairs.len() {
                    if col_row_pairs[i].1 == col_row_pairs[j].1 {
                        // Found X-Wing pattern: value appears in same 2 rows in columns i and j
                        let col1 = col_row_pairs[i].0;
                        let col2 = col_row_pairs[j].0;
                        let rows_mask = col_row_pairs[i].1;

                        // Extract the two row indices
                        let row1 = rows_mask.trailing_zeros() as usize;
                        let row2 = (rows_mask & !(1u16 << row1)).trailing_zeros() as usize;

                        // Eliminate value from other columns in these rows
                        for col in 0..9 {
                            if col != col1 && col != col2 {
                                if self.cells[row1][col].is_possible(value) {
                                    self.cells[row1][col].eliminate(value);
                                    eliminated += 1;
                                }
                                if self.cells[row2][col].is_possible(value) {
                                    self.cells[row2][col].eliminate(value);
                                    eliminated += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        eliminated
    }

    /// Phase 4: Solver loop - perform one iteration using naked and hidden singles
    /// Applies Swordfish strategy: if a digit appears in exactly 2-3 candidates in each of 3 rows,
    /// and these candidates span only 3 columns, eliminate the digit from other rows in those columns.
    /// Similarly for columns. Returns the count of eliminated candidates.
    fn apply_swordfish(&mut self) -> usize {
        let mut eliminated = 0;

        // Check rows for Swordfish pattern
        for value in 1..=9 {
            let mut row_col_pairs: Vec<(usize, u16)> = Vec::new();

            // For each row, find columns where this value appears as candidate
            for row in 0..9 {
                let mut cols_mask = 0u16;
                for col in 0..9 {
                    if !self.cells[row][col].is_filled() && self.cells[row][col].is_possible(value) {
                        cols_mask |= 1u16 << col;
                    }
                }

                // Only interested in rows with 2-3 candidates for this value
                if cols_mask.count_ones() >= 2 && cols_mask.count_ones() <= 3 {
                    row_col_pairs.push((row, cols_mask));
                }
            }

            // Look for three rows with combined column pattern spanning exactly 3 columns
            for i in 0..row_col_pairs.len() {
                for j in (i + 1)..row_col_pairs.len() {
                    for k in (j + 1)..row_col_pairs.len() {
                        // Combine the column masks
                        let combined_cols = row_col_pairs[i].1 | row_col_pairs[j].1 | row_col_pairs[k].1;

                        // Check if combined pattern spans exactly 3 columns
                        if combined_cols.count_ones() == 3 {
                            // Check if each row is confined to subset of these 3 columns
                            if (row_col_pairs[i].1 & combined_cols) == row_col_pairs[i].1
                                && (row_col_pairs[j].1 & combined_cols) == row_col_pairs[j].1
                                && (row_col_pairs[k].1 & combined_cols) == row_col_pairs[k].1
                            {
                                // Found Swordfish pattern in rows
                                let row1 = row_col_pairs[i].0;
                                let row2 = row_col_pairs[j].0;
                                let row3 = row_col_pairs[k].0;

                                // Extract the three column indices
                                let mut cols = [0usize; 3];
                                let mut col_idx = 0;
                                for c in 0..9 {
                                    if (combined_cols & (1u16 << c)) != 0 {
                                        cols[col_idx] = c;
                                        col_idx += 1;
                                    }
                                }

                                // Eliminate value from other rows in these columns
                                for row in 0..9 {
                                    if row != row1 && row != row2 && row != row3 {
                                        for &col in &cols {
                                            if self.cells[row][col].is_possible(value) {
                                                self.cells[row][col].eliminate(value);
                                                eliminated += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check columns for Swordfish pattern
        for value in 1..=9 {
            let mut col_row_pairs: Vec<(usize, u16)> = Vec::new();

            // For each column, find rows where this value appears as candidate
            for col in 0..9 {
                let mut rows_mask = 0u16;
                for row in 0..9 {
                    if !self.cells[row][col].is_filled() && self.cells[row][col].is_possible(value) {
                        rows_mask |= 1u16 << row;
                    }
                }

                // Only interested in columns with 2-3 candidates for this value
                if rows_mask.count_ones() >= 2 && rows_mask.count_ones() <= 3 {
                    col_row_pairs.push((col, rows_mask));
                }
            }

            // Look for three columns with combined row pattern spanning exactly 3 rows
            for i in 0..col_row_pairs.len() {
                for j in (i + 1)..col_row_pairs.len() {
                    for k in (j + 1)..col_row_pairs.len() {
                        // Combine the row masks
                        let combined_rows = col_row_pairs[i].1 | col_row_pairs[j].1 | col_row_pairs[k].1;

                        // Check if combined pattern spans exactly 3 rows
                        if combined_rows.count_ones() == 3 {
                            // Check if each column is confined to subset of these 3 rows
                            if (col_row_pairs[i].1 & combined_rows) == col_row_pairs[i].1
                                && (col_row_pairs[j].1 & combined_rows) == col_row_pairs[j].1
                                && (col_row_pairs[k].1 & combined_rows) == col_row_pairs[k].1
                            {
                                // Found Swordfish pattern in columns
                                let col1 = col_row_pairs[i].0;
                                let col2 = col_row_pairs[j].0;
                                let col3 = col_row_pairs[k].0;

                                // Extract the three row indices
                                let mut rows = [0usize; 3];
                                let mut row_idx = 0;
                                for r in 0..9 {
                                    if (combined_rows & (1u16 << r)) != 0 {
                                        rows[row_idx] = r;
                                        row_idx += 1;
                                    }
                                }

                                // Eliminate value from other columns in these rows
                                for col in 0..9 {
                                    if col != col1 && col != col2 && col != col3 {
                                        for &row in &rows {
                                            if self.cells[row][col].is_possible(value) {
                                                self.cells[row][col].eliminate(value);
                                                eliminated += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        eliminated
    }

    /// Finds all naked and hidden singles and fills them in
    /// Returns (count, vec of (row, col, value, technique)) for logging
    fn solve_one_iteration(&mut self) -> (usize, Vec<(usize, usize, u8, String)>) {
        // First, apply constraint propagation techniques
        let pointing_pairs_elim = self.apply_pointing_pairs();
        let box_line_elim = self.apply_box_line_reduction();
        let xwing_elim = self.apply_xwing();
        let swordfish_elim = self.apply_swordfish();
        
        if pointing_pairs_elim > 0 {
            println!("  Pointing Pairs eliminated {} candidates", pointing_pairs_elim);
        }
        if box_line_elim > 0 {
            println!("  Box/Line Reduction eliminated {} candidates", box_line_elim);
        }
        if xwing_elim > 0 {
            println!("  X-Wing eliminated {} candidates", xwing_elim);
        }
        if swordfish_elim > 0 {
            println!("  Swordfish eliminated {} candidates", swordfish_elim);
        }
        
        let mut all_singles = Vec::new();
        let mut seen = 0u128; // Bitmask for 81 cells (row * 9 + col)
        let mut result = Vec::new();

        // Find naked singles
        for (r, c, v) in self.find_naked_singles() {
            let bit_index = r * 9 + c;
            if (seen & (1u128 << bit_index)) == 0 {
                all_singles.push((r, c, v, "Naked Single"));
                result.push((r, c, v, "Naked Single".to_string()));
                seen |= 1u128 << bit_index;
            }
        }

        // Find hidden singles
        for (r, c, v) in self.find_hidden_singles() {
            let bit_index = r * 9 + c;
            if (seen & (1u128 << bit_index)) == 0 {
                all_singles.push((r, c, v, "Hidden Single"));
                result.push((r, c, v, "Hidden Single".to_string()));
                seen |= 1u128 << bit_index;
            }
        }

        let count = all_singles.len();

        for (row, col, value, _technique) in all_singles {
            // Only set if not already filled (set_cell will propagate constraints)
            if !self.cells[row][col].is_filled() {
                self.set_cell(row, col, value);
            }
        }

        // Return the count and detailed info for logging
        (count, result)
    }

    /// Display board showing candidates: filled cells as digit, empty cells as candidate list
    /// Only shows cells with N or fewer candidates (or filled cells)
    fn print_candidates(&self, max_candidates: usize) {
        let max_cand_u32 = max_candidates as u32;
        for (row_idx, row) in self.cells.iter().enumerate() {
            if row_idx > 0 && row_idx % 3 == 0 {
                println!("---------+---------+---------");
            }

            for (col_idx, cell) in row.iter().enumerate() {
                if col_idx > 0 && col_idx % 3 == 0 {
                    print!("| ");
                }

                if cell.is_filled() {
                    if let Some(val) = cell.get_value() {
                        print!("{:>3}", val);
                    }
                } else if cell.count_possibilities() == 0 {
                    // Error state: no candidates
                    print!(" XX");
                } else if cell.count_possibilities() == 1 {
                    // Naked single not yet filled
                    let mut candidates = String::new();
                    for digit in 1..=9 {
                        if cell.is_possible(digit) {
                            candidates.push_str(&digit.to_string());
                        }
                    }
                    print!("{:>3}", candidates);
                } else if cell.count_possibilities() <= max_cand_u32 {
                    // Show candidates for cells with <= max
                    let mut candidates = String::new();
                    for digit in 1..=9 {
                        if cell.is_possible(digit) {
                            candidates.push_str(&digit.to_string());
                        }
                    }
                    print!("{:>3}", candidates);
                } else {
                    // More than max candidates, show dot
                    print!("  .");
                }
                
                if col_idx < 8 {
                    print!(" ");
                }
            }
            println!();
        }
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

    board.init_from_string(vincents_challenge);

    println!("Initial board:");
    println!("{}", board);

    // Solve with naked and hidden singles
    println!("\n=== Solving ===");
    let mut iteration = 0;
    let mut total_moves = 0;
    
    loop {
        iteration += 1;
        let (moves, solved_cells) = board.solve_one_iteration();
        total_moves += moves;
        
        println!("Iteration {}: {} cells filled", iteration, moves);
        for (row, col, value, technique) in &solved_cells {
            println!("  ({},{}) = {} via {}", row, col, value, technique);
        }
        
        if moves == 0 {
            break;
        }
        
        if board.is_solved() {
            println!("Puzzle solved!");
            break;
        }
    }
    
    println!("\nTotal moves made: {}", total_moves);
    println!("Board solved: {}", board.is_solved());
    println!("Board valid: {}", board.is_valid());
    
    // Validate constraint propagation
    let mut has_violations = false;
    for row in 0..9 {
        for col in 0..9 {
            if let Some(filled_val) = board.cells[row][col].get_value() {
                let box_idx = (row / 3) * 3 + (col / 3);
                let box_cells = Board::get_box_cells(box_idx);
                for (r, c) in box_cells {
                    if (r, c) != (row, col) && board.cells[r][c].is_possible(filled_val) {
                        println!("ERROR: ({},{}) = {} but ({},{}) still has {} as candidate", 
                                 row, col, filled_val, r, c, filled_val);
                        has_violations = true;
                    }
                }
            }
        }
    }
    if !has_violations {
        println!("✓ Constraint propagation validated - no violations!");
    }
    
    if board.is_solved() {
        println!("\n✓ Puzzle Solved!");
        println!("{}", board);
    } else {
        println!("\nBoard after solving with singles:");
        println!("{}", board);
        
        println!("\nCells with 3 or fewer candidates:");
        board.print_candidates(3);
    }
}

impl Clone for Board {
    fn clone(&self) -> Self {
        Board {
            cells: self.cells,
            constraints: self.constraints,
        }
    }
}
