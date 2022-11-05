use std::cmp;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::gridvec::*;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct GridBounds {
    pub bottom_left: GridVec,
    pub top_right: GridVec,
}

impl std::fmt::Display for GridBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<from {} to {}>", self.bottom_left(), self.top_right())
    }
}

impl GridBounds {
    pub fn new(center: GridVec, half_extent: GridVec) -> Self {
        GridBounds { bottom_left: center - half_extent, top_right: center + half_extent }
    }

    pub fn new_from_corner(bottom_left: GridVec, size: GridVec) -> Self {
        GridBounds { bottom_left, top_right: bottom_left + size }
    }

    pub fn new_from_extents(bottom_left: GridVec, top_right: GridVec) -> Self {
        GridBounds { bottom_left, top_right }
    }

    pub fn bottom(&self) -> i32 {
        self.bottom_left.y
    }

    pub fn left(&self) -> i32 {
        self.bottom_left.x
    }

    pub fn _top(&self) -> i32 {
        self.top_right.y
    }

    pub fn right(&self) -> i32 {
        self.top_right.x
    }

    pub fn bottom_left(&self) -> GridVec {
        self.bottom_left   
    }

    pub fn _bottom_right(&self) -> GridVec {
        GridVec { x: self.top_right.x, y: self.bottom_left.y }   
    }

    pub fn top_left(&self) -> GridVec {
        GridVec { x: self.bottom_left.x, y: self.top_right.y }      
    }

    pub fn top_right(&self) -> GridVec {
        self.top_right   
    }

    pub fn width(&self) -> u32 {
        (self.top_right.x - self.bottom_left.x) as u32
    }

    pub fn height(&self) -> u32 {
        (self.top_right.y - self.bottom_left.y) as u32
    }

    pub fn center(&self) -> GridVec {
        (self.top_right + self.bottom_left) / 2
    }

    pub fn extent(&self) -> GridVec {
        self.top_right - self.bottom_left  
    }

    pub fn half_extent(&self) -> GridVec {
        self.extent() / 2
    }

    pub fn move_by(&mut self, by: GridVec) {
        self.top_right = self.top_right + by;
        self.bottom_left = self.bottom_left + by;
    }

    pub fn resize(&mut self, new_size: GridVec) {
        let difference = new_size - self.extent();
        self.top_right = self.top_right + (difference / 2) + GridVec::new(difference.x % 2, difference.y % 2);
        self.bottom_left = self.bottom_left - (difference / 2);
    }

    pub fn contains(&self, point: GridVec) -> bool {
        let delta = point - self.center();
        let half_extent = self.half_extent();
        return delta.x.abs() <= half_extent.x && delta.y.abs() <= half_extent.y;
    }

    pub fn is_boundary(&self, point: GridVec) -> bool {
        self.contains(point) 
        && (point.x == self.bottom_left().x 
            || point.x == self.top_right().x - 1
            || point.y == self.bottom_left().y
            || point.y == self.top_right().y - 1
        )
    }

    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }

    pub fn iter(&self) -> GridIterator {
        GridIterator { bounds: self.clone(), current: self.bottom_left() + GridVec::new(-1, 0) }
    }

    pub fn slide_iter(&self) -> SlideGridIterator {
        SlideGridIterator { 
            bounds: self.clone(), 
            current: self.top_left() + GridVec::new(-1, -1),
            rng: rand::thread_rng(),
            flipped_x: false,    
        }
    }

    // Returns a bounds that exactly contains both input bounds
    pub fn union(&self, other: GridBounds) -> GridBounds {
        let bottom_left = GridVec::new(
            cmp::min(self.bottom_left().x, other.bottom_left().x),
            cmp::min(self.bottom_left().y, other.bottom_left().y)
        );
        let top_right = GridVec::new(
            cmp::max(self.top_right().x, other.top_right().x),
            cmp::max(self.top_right().y, other.top_right().y)
        );
        GridBounds::new_from_extents(bottom_left, top_right)
    }

    pub fn option_union(a: Option<GridBounds>, b: Option<GridBounds>) -> Option<GridBounds> {
        if a.is_none() && b.is_none() { None }
        else if let Some(bound_a) = a {
            if let Some(bound_b) = b {
                Some(bound_a.union(bound_b))
            }
            else {
                a
            }
        }
        else {
            b
        }
    }

    pub fn intersect_option(&self, other: Option<GridBounds>) -> Option<GridBounds>  {
        if let Some(bounds) = other {
            self.intersect(bounds)
        }
        else {
            None
        }
    }

    pub fn overlaps(&self, other: GridBounds) -> bool {
        let dx = other.center().x - self.center().x;
        let px = (other.half_extent().x + self.half_extent().x) - dx.abs();
        if px <= 0 {
            return false;
        }

        let dy = other.center().y - self.center().y;
        let py = (other.half_extent().y + self.half_extent().y) - dy.abs();
        if py <= 0 {
            return false;
        }

        true
    }

    // If there is an intersection, returns the bounds of the overlapping area
    pub fn intersect(&self, other: GridBounds) -> Option<GridBounds> {
        let dx = other.center().x - self.center().x;
        let px = (other.half_extent().x + self.half_extent().x) - dx.abs();
        if px <= 0 {
            return None;
        }

        let dy = other.center().y - self.center().y;
        let py = (other.half_extent().y + self.half_extent().y) - dy.abs();
        if py <= 0 {
            return None;
        }

        let bottom_left = GridVec::new(
            cmp::max(self.bottom_left().x, other.bottom_left().x),
            cmp::max(self.bottom_left().y, other.bottom_left().y)
        );
        let top_right = GridVec::new(
            cmp::min(self.top_right().x, other.top_right().x),
            cmp::min(self.top_right().y, other.top_right().y)
        );
        Some(GridBounds::new_from_extents(bottom_left, top_right))
    }
}

pub struct GridIterator {
    bounds: GridBounds,
    current: GridVec,
}

pub struct SlideGridIterator {
    bounds: GridBounds,
    current: GridVec,
    rng: ThreadRng,
    flipped_x: bool,
}

impl Iterator for GridIterator {
    type Item = GridVec;

    fn next(&mut self) -> Option<GridVec> {
        self.current.x += 1;
        if self.current.x >= self.bounds.top_right().x {
            self.current.x = self.bounds.bottom_left().x;
            self.current.y += 1;

            if self.current.y >= self.bounds.top_right().y {
                return None
            }
        }

        return Some(self.current);
    }
}

impl Iterator for SlideGridIterator {
    type Item = GridVec;

    fn next(&mut self) -> Option<GridVec> {
        self.current.x += if self.flipped_x { -1 } else { 1 };
        if (self.flipped_x && self.current.x < self.bounds.left()) 
        || (!self.flipped_x && self.current.x >= self.bounds.right()) {
            self.flipped_x = self.bounds.width() != 0 && self.rng.gen_bool(0.5);

            self.current.x = if self.flipped_x { self.bounds.top_right().x - 1 } else { self.bounds.bottom_left().x };
            self.current.y -= 1;


            if self.current.y < self.bounds.bottom() {
                return None
            }
        }

        return Some(self.current);
    }
}

#[cfg(test)]
mod tests {
    use crate::gridbounds::*;
    use crate::gridvec::*;

    #[test]
    fn bounds_resize_even() {
        let mut a = GridBounds::new(GridVec::new(0, 0), GridVec::new(4, 2));
        a.resize(GridVec::new(16, 8));
        let expected = GridBounds::new(GridVec::new(0, 0), GridVec::new(8, 4));
        assert_eq!(a.extent(), expected.extent()); // Confirm the new size is as expected
        assert_eq!(a.center(), expected.center()); // Confirm the center point moved as expected
    }

    #[test]
    fn bounds_resize_even_to_odd() {
        let mut a = GridBounds::new(GridVec::new(0, 0), GridVec::new(8, 4));
        a.resize(GridVec::new(15, 7));
        let expected = GridBounds::new_from_corner(GridVec::new(-7, -3), GridVec::new(15,7));
        assert_eq!(a.extent(), expected.extent()); // Confirm the new size is as expected
        assert_eq!(a.center(), expected.center()); // Confirm the center point moved as expected
    }

    #[test]
    fn bounds_corners() {
        let a = GridBounds::new(GridVec::new(0, 0), GridVec::new(1, 1));
        let bottom_left = a.bottom_left();
        let top_right = a.top_right();

        assert_eq!(bottom_left, GridVec::new(-1, -1));
        assert_eq!(top_right, GridVec::new(1, 1));
    }

    #[test]
    fn bounds_corners_from_corner() {
        let bottom_left = GridVec::new(0, 0);
        let size = GridVec::new(16, 16);
        let top_right = size;
        
        let a = GridBounds::new_from_corner(bottom_left, size);
        
        assert_eq!(a.bottom_left(), bottom_left);
        assert_eq!(a.top_right(), top_right);
    }

    #[test]
    fn intersection_overlap_none() {
        let a = GridBounds::new(GridVec::new(0, 0), GridVec::new(1, 1));
        let b = GridBounds::new(GridVec::new(3, 0), GridVec::new(1, 1));

        let result = a.intersect(b);
        let expected = None;
        assert_eq!(result, expected);
    }

    #[test]
    fn intersection_overlap_contained() {
        let a = GridBounds::new(GridVec::new(0, 0), GridVec::new(1, 1));
        let b = GridBounds::new(GridVec::new(0, 0), GridVec::new(10, 10));

        let result = a.intersect(b);
        let expected = Some(a);
        assert_eq!(result, expected);
    }

    #[test]
    fn intersection_overlap_partial() {
        let a = GridBounds::new(GridVec::new(0, 0), GridVec::new(2, 2));
        let b = GridBounds::new(GridVec::new(2, 2), GridVec::new(2, 2));

        let result = a.intersect(b);
        let expected = Some(GridBounds::new(GridVec::new(1, 1), GridVec::new(1, 1)));
        assert_eq!(result, expected);
    }

    #[test]
    fn union_overlap_none() {
        let a = GridBounds::new(GridVec::new(0, 0), GridVec::new(1, 1));
        let b = GridBounds::new(GridVec::new(4, 0), GridVec::new(1, 1));

        let result = a.union(b);
        let expected = GridBounds::new(GridVec::new(2, 0), GridVec::new(3, 1));
        assert_eq!(result, expected);
    }

    #[test]
    fn union_overlap_contained() {
        let a = GridBounds::new(GridVec::new(0, 0), GridVec::new(1, 1));
        let b = GridBounds::new(GridVec::new(0, 0), GridVec::new(10, 10));

        let result = a.union(b);
        let expected = b;
        assert_eq!(result, expected);
    }

    #[test]
    fn union_overlap_partial() {
        let a = GridBounds::new(GridVec::new(0, 0), GridVec::new(4, 4));
        let b = GridBounds::new(GridVec::new(2, 2), GridVec::new(4, 4));

        let result = a.union(b);
        let expected = GridBounds::new(GridVec::new(1, 1), GridVec::new(5, 5));
        assert_eq!(result, expected);
    }
}