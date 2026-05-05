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

    /// Set a cell to a fixed value
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

        true
    }

    /// Get the box index (0-8) for a given row and column
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

    /// Check if the board is valid (no conflicts)
    fn is_valid(&self) -> bool {
        // All cells must have at least one possibility
        if self.cells.iter().any(|row| row.iter().any(|cell| cell.count_possibilities() == 0)) {
            return false;
        }

        // Each constraint must have at least one bit set
        if self.row_constraints.iter().any(|&c| c == 0) {
            return false;
        }
        if self.col_constraints.iter().any(|&c| c == 0) {
            return false;
        }
        if self.box_constraints.iter().any(|&c| c == 0) {
            return false;
        }

        true
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
    // Test Phase 1: Core data structure
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

    println!("Sudoku Board (Vincent's Challenge):");
    println!("{}", board);

    println!("Board is valid: {}", board.is_valid());
    println!("Board is solved: {}", board.is_solved());

    // Test cell mask operations
    let mut test_cell = Cell::new_empty();
    println!("\nTest cell possibilities: {:?}", test_cell.possibilities());
    test_cell.eliminate(5);
    println!("After eliminating 5: {:?}", test_cell.possibilities());
}
