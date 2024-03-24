use bevy::{ecs::schedule::common_conditions::resource_added, math::primitives::Line2d};
use bevy_rapier2d::math::Vect;

pub struct PolylineSet {
    segments: Vec<Vec<Vect>>,
}

impl PolylineSet {
    pub fn new() -> Self {
        PolylineSet {
            segments: Vec::new(),
        }
    }

    pub fn add(&mut self, from: Vect, to: Vect) {
        let mut matched = false;
        for segment in self.segments.iter_mut() {
            // Try inserting back first as thats cheaper
            if segment.last().unwrap().distance_squared(from) < 0.1 {
                segment.push(to);
                matched = true;
            }
            // Don't want to join both ends need an else
            else if segment.first().unwrap().distance_squared(to) < 0.1 {
                segment.insert(0, from);
                matched = true;
            }
        }

        if !matched {
            let mut new_seg = Vec::new();
            new_seg.push(from);
            new_seg.push(to);
            self.segments.push(new_seg);
        }
    }

    

    fn simplify_segment(segment: &Vec<Vect>, start: usize, end: usize, epsilon: f32) -> Vec<Vect> {
        let mut dmax = 0.;
        let mut index = 0;

        for i in (start + 1)..(end - 1) {
            let d = point_line_distance(segment[start], segment[end], segment[i]);
            if d > dmax {
                index = i;
                dmax = d;
            }
        }

        let mut result = Vec::new();
        result.reserve(end - start);

        if dmax > epsilon {
            let mut a = PolylineSet::simplify_segment(segment, start, index, epsilon);
            let mut b = PolylineSet::simplify_segment(segment, index, end, epsilon);
            a.pop(); // Remove the last point of a since it will be the same as the first of b

            result.append(&mut a);
            result.append(&mut b);
        }
        else {
            result.push(segment[start]);
            result.push(segment[end]);
        }

        result
    }

    pub fn simplify(&mut self, epsilon: f32) {
        for segment in self.segments.iter_mut() {
            *segment = PolylineSet::simplify_segment(segment, 0, segment.len() - 1, epsilon);
        }
    }

    pub fn to_verts_and_inds(&self) -> (Vec<Vect>, Vec<[u32; 2]>) {
        let mut verts = Vec::new(); 
        let mut indices = Vec::new();

        for segment in self.segments.iter() {
            let mut first = true;
            for vert in segment.iter() {
                let added_index = verts.len() as u32;
                verts.push(*vert);

                if !first {
                    indices.push([added_index - 1, added_index]);
                }
                else {
                    first = false;
                }
            }
        }

        (verts, indices)
    }
}

fn point_line_distance(p1: Vect, p2: Vect, point: Vect) -> f32 {
    ((p2.x - p1.x) * (p1.y - point.y) - (p1.x - point.x) *  (p2.y - p1.y)).abs() / 
        ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use crate::polyline::*;
    use bevy::math::Vec2;

    #[test]
    fn basic_insertion() {
        let mut line = PolylineSet::new();

        line.add(Vec2::new(0., 0.), Vec2::new(0., 1.));
        line.add(Vec2::new(0., 1.), Vec2::new(0., 2.));
        line.add(Vec2::new(0., 2.), Vec2::new(0., 3.));

        assert_eq!(line.segments.len(), 1);
        assert_eq!(line.segments[0].len(), 4);
    }

    #[test]
    fn basic_insertion_backwards() {
        let mut line = PolylineSet::new();

        line.add(Vec2::new(0., 2.), Vec2::new(0., 3.));
        line.add(Vec2::new(0., 1.), Vec2::new(0., 2.));
        line.add(Vec2::new(0., 0.), Vec2::new(0., 1.));

        assert_eq!(line.segments.len(), 1);
        assert_eq!(line.segments[0].len(), 4);
    }

    #[test]
    fn basic_simplification() {
        let mut line = PolylineSet::new();

        line.add(Vec2::new(0., 0.), Vec2::new(0., 1.));
        line.add(Vec2::new(0., 1.), Vec2::new(0., 2.));
        line.add(Vec2::new(0., 2.), Vec2::new(0., 3.));

        line.simplify(1.0);

        assert_eq!(line.segments.len(), 1);
        assert_eq!(line.segments[0].len(), 2);
    }

    #[test]
    fn zigzag_insertion() {
        let mut line = PolylineSet::new();

        line.add(Vec2::new(3., 2.), Vec2::new(4., 0.));
        line.add(Vec2::new(2., 0.), Vec2::new(3., 2.));
        line.add(Vec2::new(1., 0.5), Vec2::new(2., 0.));
        line.add(Vec2::new(0., 0.), Vec2::new(1., 0.5));

        assert_eq!(line.segments.len(), 1);
        assert_eq!(line.segments[0].len(), 5);
    }

    #[test]
    fn zigzag_insertion_backwards() {
        let mut line = PolylineSet::new();

        line.add(Vec2::new(0., 0.), Vec2::new(1., 0.5));
        line.add(Vec2::new(1., 0.5), Vec2::new(2., 0.));
        line.add(Vec2::new(2., 0.), Vec2::new(3., 2.));
        line.add(Vec2::new(3., 2.), Vec2::new(4., 0.));

        assert_eq!(line.segments.len(), 1);
        assert_eq!(line.segments[0].len(), 5);
    }

    #[test]
    fn zigzag_simplification() {
        let mut line = PolylineSet::new();

        line.add(Vec2::new(0., 0.), Vec2::new(1., 0.5));
        line.add(Vec2::new(1., 0.5), Vec2::new(2., 0.));
        line.add(Vec2::new(2., 0.), Vec2::new(3., 0.5));
        line.add(Vec2::new(3., 0.5), Vec2::new(4., 0.));

        line.simplify(1.0);

        assert_eq!(line.segments.len(), 1);
        assert_eq!(line.segments[0].len(), 2);
    }

}