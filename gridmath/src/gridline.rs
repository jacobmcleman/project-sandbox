use std::fmt;

use crate::gridvec::*;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct GridLine {
    pub a: GridVec,
    pub b: GridVec,
}

pub struct GridLineIterator {
    end: GridVec,
    current: GridVec,
    done: bool,
}

impl GridLine {
    pub fn new(a: GridVec, b: GridVec) -> Self {
        Self {a, b}
    }

    pub fn sq_length(&self) -> i32 {
        (self.a.x - self.b.x).pow(2) + (self.a.y - self.b.y).pow(2)
    }

    pub fn manhattan_length(&self) -> i32 {
        (self.a.x - self.b.x).abs() + (self.a.y - self.b.y).abs()
    }

    pub fn along(&self) -> GridLineIterator {
        GridLineIterator {
            current: self.a,
            end: self.b,
            done: false,
        }
    }

    pub fn intersect(&self, other: &GridLine) -> Option<GridVec> {
        let x1 = self.a.x;
        let y1 = self.a.y;

        let x2 = self.b.x;
        let y2 = self.b.y;

        let x3 = other.a.x;
        let y3 = other.a.y;

        let x4 = other.b.x;
        let y4 = other.b.y;

        // Calculate the intersection t
        // leaving as ratio until final step because integer
        let t_num = ((x1 - x3) * (y3 - y4)) - ((y1 - y3) * (x3 - x4));
        let t_den = ((x1 - x2) * (y3 - y4)) - ((y1 - y2) * (x3 - x4));

        let u_num = ((x1 - x2) * (y1 - y3)) - ((y1 - y2) * (x1 - x3));
        let u_den = ((x1 - x2) * (y3 - y4)) - ((y1 - y2) * (x3 - x4));

        // If t and u are both in [0, 1], there is an intersection
        // Check for < 0 by making sure the signs match
        if t_den == 0 || u_den == 0 || t_num.signum() != t_den.signum() || u_num.signum() != u_den.signum() {
            return None
        }
        // Check for > 1 by making sure the numerator is not > denominator
        if t_num > t_den || u_num > u_den {
            return None
        }

        // There is an intersection
        let i_x = x1 + ((t_num * (x1 - x2)) / t_den);
        let i_y = y1 + ((t_num * (y1 - y2)) / t_den);

        Some(GridVec::new(i_x, i_y))
    }
}

impl fmt::Display for GridLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "|{0} to {1}|", self.a, self.b)
    }
}

impl Iterator for GridLineIterator {
    type Item = GridVec;

    fn next(&mut self) -> Option<GridVec> {
        if self.done {
            None
        }
        else if self.current == self.end {
            self.done = true;
            Some(self.end)
        }
        else {
            let last = self.current;
            let move_vec = self.end - self.current;
            if move_vec.x == 0 || move_vec.y == 0 {
                // Alligned on one axis, move along it
                self.current.x += move_vec.x.signum();
                self.current.y += move_vec.y.signum();
            }
            else {
                // Decide which movement option gets closer
                let x_move = self.current + GridVec::new(move_vec.x.signum(), 0);
                let y_move = self.current + GridVec::new(0, move_vec.y.signum());
            
                if self.end.sq_distance(x_move) < self.end.sq_distance(y_move) {
                    self.current = x_move;
                }
                else {
                    self.current = y_move;
                }
            }


            Some(last)
        }
        
    }
}