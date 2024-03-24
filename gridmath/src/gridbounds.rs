use std::cmp;
use rand::Rng;
use rand::rngs::ThreadRng;

use crate::gridline::GridLine;
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

    pub fn containing(points: &Vec<GridVec>) -> Self {
        let mut bound = GridBounds::new(points[0], GridVec::new(0,0));

        for point in points.iter() {
            bound = bound.union(GridBounds::new(*point, GridVec::new(0,0)));
        }

        bound
    }

    pub fn bottom(&self) -> i32 {
        self.bottom_left.y
    }

    pub fn left(&self) -> i32 {
        self.bottom_left.x
    }

    pub fn top(&self) -> i32 {
        self.top_right.y
    }

    pub fn right(&self) -> i32 {
        self.top_right.x
    }

    pub fn bottom_line(&self) -> GridLine {
        GridLine::new(self.bottom_left(), self.bottom_right())
    }

    pub fn left_line(&self) -> GridLine {
        GridLine::new(self.bottom_left(), self.top_left())
    }

    pub fn top_line(&self) -> GridLine {
        GridLine::new(self.top_left(), self.top_right())
    }

    pub fn right_line(&self) -> GridLine {
        GridLine::new(self.top_right(), self.bottom_right())
    }

    pub fn bottom_left(&self) -> GridVec {
        self.bottom_left   
    }

    pub fn bottom_right(&self) -> GridVec {
        GridVec { x: self.top_right.x, y: self.bottom_left.y }   
    }

    pub fn top_left(&self) -> GridVec {
        GridVec { x: self.bottom_left.x, y: self.top_right.y }      
    }

    pub fn top_right(&self) -> GridVec {
        self.top_right   
    }

    pub fn width(&self) -> u32 {
        (self.top_right.x - self.bottom_left.x) as u32 + 1
    }

    pub fn height(&self) -> u32 {
        (self.top_right.y - self.bottom_left.y) as u32 + 1
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

    pub fn inflated_by(&self, radial_increase: i32) -> GridBounds {
        let change_vec = GridVec::new(radial_increase, radial_increase);
        GridBounds::new_from_extents(self.bottom_left - change_vec, self.top_right + change_vec)
    }

    pub fn contains(&self, point: GridVec) -> bool {
        point.x >= self.left() 
        && point.x <= self.right() 
        && point.y >= self.bottom()
        && point.y <= self.top()
    }

    pub fn is_boundary(&self, point: GridVec) -> bool {
        self.contains(point) 
        && (point.x == self.bottom_left().x 
            || point.x == self.top_right().x - 1
            || point.y == self.bottom_left().y
            || point.y == self.top_right().y - 1
        )
    }

    // Based on Cohen-Sutherland algorithm
    // each out of bounds direction for a point has a corresponding bit
    // if two points bitwise-and to a non-zero value then the line between them cannot be within this bounds
    fn get_boundbits(&self, point: &GridVec) -> u8 {
        let mut val = 0;

        if point.y > self.top() { val |= 1 << 0; }
        if point.y < self.bottom() { val |= 1 << 1; }
        if point.x > self.right() { val |= 1 << 2; }
        if point.x < self.left() { val |= 1 << 3; }

        val
    }

    pub fn clip_line(&self, line: GridLine) -> Option<GridLine> {
        let a_bits = self.get_boundbits(&line.a);
        let b_bits = self.get_boundbits(&line.b);
        if (a_bits | b_bits) == 0 {
            //println!("passed bitcheck, both within bounds");
            return Some(line);
        }
        else if (a_bits & b_bits) == 0 {
            //println!("didn't fail bitcheck, endpoints are in different regions on {}", line);
            // Some intersection might be happening

            // Check for intersections with the edges of the bounds
            let mut intersections = vec![];
            intersections.push(self.left_line().intersect(&line));
            intersections.push(self.right_line().intersect(&line));
            intersections.push(self.top_line().intersect(&line));
            intersections.push(self.bottom_line().intersect(&line));

            let intersections: Vec<GridVec> = intersections.iter().filter_map(|intersect| { *intersect }).collect();

            //println!("found {} intersections", intersections.len());

            if intersections.len() == 0 {
                if self.contains(line.a) {
                    return Some(line);
                }
            }
            else if intersections.len() == 1 {
                // One of the points is within the bounds, determine which and replace the other
                if self.contains(line.a) {
                    //println!("a is within bounds, replacing b with clippped point {}", intersections[0]);
                    return Some(GridLine::new(line.a, intersections[0]));
                }
                else {
                    //println!("a is not within bounds, replacing a with clippped point {}", intersections[0]);
                    return Some(GridLine::new(intersections[0], line.b));
                }
            }
            else if intersections.len() == 2 {
                // We're making a new line with new points, try to preserve directionality
                if line.a.sq_distance(intersections[0]) <= line.a.sq_distance(intersections[1]) {
                    return Some(GridLine::new(intersections[0], intersections[1]));
                }
                else {
                    return Some(GridLine::new(intersections[1], intersections[0]));
                }
            }
            else {
                println!("weird shit, {} intersections found on clip", intersections.len())
            }

            None
        } 
        else {
            // Impossible for intersection to occur
            //println!("failed bitcheck, points are on same outside side");
            None
        }
    }

    pub fn area(&self) -> usize {
        self.width() as usize * self.height() as usize
    }

    pub fn get_index(&self, pos: GridVec) -> Option<usize> {
        if !self.contains(pos) {
            return None;
        }

        let x = (pos.x - self.left()) as usize;
        let y = (pos.y - self.bottom()) as usize;

        Some(y * self.width() as usize + x)
    }

    pub fn at_index(&self, index: usize) -> GridVec {
        let x = index % (self.width() as usize);
        let y = index / (self.width() as usize);

        GridVec::new(self.left() + x as i32, self.bottom() + y as i32)
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
    fn to_from_index_symetric() {
        let bottom_left = GridVec::new(0, 0);
        let size = GridVec::new(16, 16);
        
        let bounds = GridBounds::new_from_corner(bottom_left, size);

        let in_point = GridVec::new(3, 7);

        let index_1 = bounds.get_index(in_point).expect("point is within bounds");
        println!("index: {}", index_1);
        let out_point = bounds.at_index(index_1);
        assert_eq!(in_point, out_point);

        let index_2 = bounds.get_index(out_point).expect("point is within bounds");
        assert_eq!(index_1, index_2);
    }

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

    #[test]
    fn line_clip_contained() {
        let bounds = GridBounds::new(GridVec::new(0, 0), GridVec::new(10, 10));
        let line = GridLine::new(GridVec::new(1, 1), GridVec::new(9, 9));

        let result = bounds.clip_line(line);
        assert_eq!(result, Some(line));
    }

    #[test]
    fn line_clip_one_edge() {
        let bounds = GridBounds::new_from_extents(GridVec::new(0, 0), GridVec::new(10, 10));

        let line_right = GridLine::new(GridVec::new(5, 5), GridVec::new(15, 5));
        let expected_line_right = GridLine::new(GridVec::new(5, 5), GridVec::new(10, 5));
        assert_eq!(bounds.clip_line(line_right), Some(expected_line_right));
        assert_eq!(bounds.clip_line(line_right.reversed()), Some(expected_line_right.reversed()));

        let line_left = GridLine::new(GridVec::new(-5, 5), GridVec::new(5, 5));
        let expected_line_left = GridLine::new(GridVec::new(0, 5), GridVec::new(5, 5));
        assert_eq!(bounds.clip_line(line_left), Some(expected_line_left));
        assert_eq!(bounds.clip_line(line_left.reversed()), Some(expected_line_left.reversed()));

        let line_up = GridLine::new(GridVec::new(5, 5), GridVec::new(5, 15));
        let expected_line_up = GridLine::new(GridVec::new(5, 5), GridVec::new(5, 10));
        assert_eq!(bounds.clip_line(line_up), Some(expected_line_up));
        assert_eq!(bounds.clip_line(line_up.reversed()), Some(expected_line_up.reversed()));

        let line_down = GridLine::new(GridVec::new(5, -5), GridVec::new(5, 5));
        let expected_line_down = GridLine::new(GridVec::new(5, 0), GridVec::new(5, 5));
        assert_eq!(bounds.clip_line(line_down), Some(expected_line_down));
        assert_eq!(bounds.clip_line(line_down.reversed()), Some(expected_line_down.reversed()));
    }

    #[test]
    fn line_clip_two_opposite_edges() {
        let bounds = GridBounds::new_from_extents(GridVec::new(0, 0), GridVec::new(10, 10));

        let horizontal_line = GridLine::new(GridVec::new(-5, 5), GridVec::new(15, 5));
        let expected_h_line = GridLine::new(GridVec::new(0, 5), GridVec::new(10, 5));
        assert_eq!(bounds.clip_line(horizontal_line), Some(expected_h_line));
        assert_eq!(bounds.clip_line(horizontal_line.reversed()), Some(expected_h_line.reversed()));

        let vertical_line = GridLine::new(GridVec::new(5, -5), GridVec::new(5, 15));
        let expected_v_line = GridLine::new(GridVec::new(5, 0), GridVec::new(5, 10));
        assert_eq!(bounds.clip_line(vertical_line), Some(expected_v_line));
        assert_eq!(bounds.clip_line(vertical_line.reversed()), Some(expected_v_line.reversed()));
    }

    #[test]
    fn line_clip_line_along_edge() {
        let bounds = GridBounds::new_from_extents(GridVec::new(0, 0), GridVec::new(5, 5));

        let horizontal_line = GridLine::new(GridVec::new(-5, 5), GridVec::new(15, 5));
        let expected_h_line = GridLine::new(GridVec::new(0, 5), GridVec::new(5, 5));
        assert_eq!(bounds.clip_line(horizontal_line), Some(expected_h_line));
        assert_eq!(bounds.clip_line(horizontal_line.reversed()), Some(expected_h_line.reversed()));

        let vertical_line = GridLine::new(GridVec::new(5, -5), GridVec::new(5, 15));
        let expected_v_line = GridLine::new(GridVec::new(5, 0), GridVec::new(5, 5));
        assert_eq!(bounds.clip_line(vertical_line), Some(expected_v_line));
        assert_eq!(bounds.clip_line(vertical_line.reversed()), Some(expected_v_line.reversed()));
    }

    #[test]
    fn line_clip_two_diagonal_edges() {
        let bounds = GridBounds::new_from_extents(GridVec::new(0, 0), GridVec::new(10, 10));

        let line_pos_a = GridLine::new(GridVec::new(-3, 5), GridVec::new(5, 13));
        let expected_pos_a = GridLine::new(GridVec::new(0, 8), GridVec::new(2, 10));
        assert_eq!(bounds.clip_line(line_pos_a), Some(expected_pos_a));
        assert_eq!(bounds.clip_line(line_pos_a.reversed()), Some(expected_pos_a.reversed()));

        let line_pos_b = GridLine::new(GridVec::new(5, -3), GridVec::new(13, 5));
        let expected_pos_b = GridLine::new(GridVec::new(8, 0), GridVec::new(10, 2));
        assert_eq!(bounds.clip_line(line_pos_b), Some(expected_pos_b));
        assert_eq!(bounds.clip_line(line_pos_b.reversed()), Some(expected_pos_b.reversed()));

        let line_neg_a = GridLine::new(GridVec::new(-3, 5), GridVec::new(5, -3));
        let expected_neg_a = GridLine::new(GridVec::new(0, 2), GridVec::new(2, 0));
        assert_eq!(bounds.clip_line(line_neg_a), Some(expected_neg_a));
        assert_eq!(bounds.clip_line(line_neg_a.reversed()), Some(expected_neg_a.reversed()));

        let line_neg_b = GridLine::new(GridVec::new(5, 13), GridVec::new(13, 5));
        let expected_neg_b = GridLine::new(GridVec::new(8, 10), GridVec::new(10, 8));
        assert_eq!(bounds.clip_line(line_neg_b), Some(expected_neg_b));
        assert_eq!(bounds.clip_line(line_neg_b.reversed()), Some(expected_neg_b.reversed()));
    }
}