//! Converting to the axial `(q, r)` coordinates hex panels use.
//!
//! Puzzles often store hex grids as rows and columns with every other row
//! (or column) shifted. These convert those "offset" coordinates. See
//! <https://www.redblobgames.com/grids/hexagons/> for pictures.

/// Pointy-top, odd rows shifted right.
pub fn odd_r(col: i64, row: i64) -> (i64, i64) {
    (col - (row - (row & 1)) / 2, row)
}

/// Pointy-top, even rows shifted right.
pub fn even_r(col: i64, row: i64) -> (i64, i64) {
    (col - (row + (row & 1)) / 2, row)
}

/// Flat-top, odd columns shifted down.
pub fn odd_q(col: i64, row: i64) -> (i64, i64) {
    (col, row - (col - (col & 1)) / 2)
}

/// Flat-top, even columns shifted down.
pub fn even_q(col: i64, row: i64) -> (i64, i64) {
    (col, row - (col + (col & 1)) / 2)
}

/// Cube `(x, y, z)` with `x + y + z == 0`, as used by many puzzle
/// write-ups, to axial.
pub fn cube(x: i64, _y: i64, z: i64) -> (i64, i64) {
    (x, z)
}

/// The six neighbours of an axial cell, starting east and going
/// counter-clockwise on a pointy-top board.
pub fn neighbours((q, r): (i64, i64)) -> [(i64, i64); 6] {
    [
        (q + 1, r),
        (q + 1, r - 1),
        (q, r - 1),
        (q - 1, r),
        (q - 1, r + 1),
        (q, r + 1),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets() {
        assert_eq!(odd_r(1, 1), (1, 1));
        assert_eq!(odd_r(0, 2), (-1, 2));
        assert_eq!(odd_r(0, -1), (1, -1));
        assert_eq!(even_r(0, 1), (-1, 1));
        assert_eq!(odd_q(2, 0), (2, -1));
        assert_eq!(even_q(1, 0), (1, -1));
        assert_eq!(cube(1, -3, 2), (1, 2));
        assert!(neighbours((0, 0)).contains(&(1, -1)));
    }
}
