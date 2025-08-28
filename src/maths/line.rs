use crate::{
    error::TesseraError,
    geometry::{LinePrimitive, PointPrimitive},
    maths::vec::Vec3,
};

const PARALLEL_LINE_EPSILON: f64 = 1e-10;

pub fn get_shortest_distance_between_points_and_lines(
    a: &PointPrimitive,
    b: &LinePrimitive,
) -> Result<f64, TesseraError> {
    let mut shortest_distance = f64::INFINITY;

    for a_point in a.iter_vertices() {
        for (b_start, b_end) in b.iter_vertices() {
            let distance = distance_from_point_to_line_segment_squared(a_point, b_start, b_end);

            if distance < shortest_distance {
                shortest_distance = distance;
            }
        }
    }

    return Ok(shortest_distance.sqrt());
}

pub fn get_shortest_distance_between_lines(
    a: &LinePrimitive,
    b: &LinePrimitive,
) -> Result<f64, TesseraError> {
    let mut shortest_distance = f64::INFINITY;

    for (a_start, a_end) in a.iter_vertices() {
        for (b_start, b_end) in b.iter_vertices() {
            let distance = line_distance_squared(&a_start, &a_end, &b_start, &b_end);

            if distance < shortest_distance {
                shortest_distance = distance;
            }
        }
    }

    return Ok(shortest_distance.sqrt());
}

/*
    Finds the distance between two line segments in 3D space.

    a_start: The start point of the first line segment
    a_end: The end point of the first line segment
    b_start: The start point of the second line segment
    b_end: The end point of the second line segment

    Returns the squared distance between the two line segments.
*/
fn line_distance_squared(
    a_start: &[f32; 3],
    a_end: &[f32; 3],
    b_start: &[f32; 3],
    b_end: &[f32; 3],
) -> f64 {
    // Convert points to f64 for precision
    let a_start = Vec3::new(a_start[0] as f64, a_start[1] as f64, a_start[2] as f64);
    let a_end = Vec3::new(a_end[0] as f64, a_end[1] as f64, a_end[2] as f64);
    let b_start = Vec3::new(b_start[0] as f64, b_start[1] as f64, b_start[2] as f64);
    let b_end = Vec3::new(b_end[0] as f64, b_end[1] as f64, b_end[2] as f64);

    let a: Vec3 = a_end - a_start;
    let b = b_end - b_start;
    // Vector between start points
    let c = a_start - b_start;

    // Dot products
    let a_dot_a = a.dot(&a);
    let a_dot_b = a.dot(&b);
    let b_dot_b = b.dot(&b);
    let a_dot_c = a.dot(&c);
    let b_dot_c = b.dot(&c);

    /*

    All points on the line segments can be expressed as:
        A point on a => A(s) = a_start + s * (a_end - a_start) for 0 <= s <= 1
        A point on b => B(t) = b_start + t * (b_end - b_start) for 0 <= t <= 1

    Firstly, we find the closest point on the infinite versions of the provided line segments
    such that S and T are unbounded, by solving for S and T like so:

    S = (a · a) (b · c) - (a · b) (a · c)
        ---------------------------------
          (a · a) (b · b) - (a · b)^2

    T = (a · b) (b · c) - (b · b) (a · c)
        ---------------------------------
          (a · a) (b · b) - (a · b)^2

    if the lines are parallel, the denominator will be 0 (or close to) and we
    can skip this part and just calculate from an endpoint of one of the lines.
    */

    let denominator = a_dot_a * b_dot_b - a_dot_b * a_dot_b;

    if denominator.abs() < PARALLEL_LINE_EPSILON {
        // Lines are parallel, find distance from endpoints to line
        // TODO: i think we can optimise this by projecting the ends into the segments first to find the
        // overlap, or closest point otherwise then we only need to do one distance calculation instead of 4
        let distance_to_a_start =
            vec_distance_from_point_to_line_segment_squared(a_start, b_start, b_end);
        let distance_to_a_end =
            vec_distance_from_point_to_line_segment_squared(a_end, b_start, b_end);
        let distance_to_b_start =
            vec_distance_from_point_to_line_segment_squared(b_start, a_start, a_end);
        let distance_to_b_end =
            vec_distance_from_point_to_line_segment_squared(b_end, a_start, a_end);

        return distance_to_a_start
            .min(distance_to_a_end)
            .min(distance_to_b_start)
            .min(distance_to_b_end);
    }

    // Lines are not parallel, find closest points
    let s = (a_dot_a * b_dot_c - a_dot_b * a_dot_c) / denominator;
    let t = (a_dot_b * b_dot_c - b_dot_b * a_dot_c) / denominator;

    // Clamp parameters to line segments
    let s = s.clamp(0.0, 1.0);
    let t = t.clamp(0.0, 1.0);

    // Calculate closest points
    let closest_on_a = a_start + s * a;

    let closest_on_b = b_start + t * b;

    return (closest_on_a - closest_on_b).length_squared();
}

/*
    Finds the shortest distance between a point and a line segment in 3D space.

    point: The point to find the distance to
    line_start: The start point of the line segment
    line_end: The end point of the line segment

    Returns the squared distance between the point and the line segment.
*/
fn vec_distance_from_point_to_line_segment_squared(
    point: Vec3,
    line_start: Vec3,
    line_end: Vec3,
) -> f64 {
    let line = line_end - line_start;
    let start_to_point = point - line_start;

    // A line segment can be expressed as:
    //   P(t) = line_start + t * line for 0 <= t <= 1
    // therefore project the point onto the line to find where it is relative to
    // the line segment. if the point is outside the segment, it's nearest point will
    // be one of the vertices of the line segment. otherwise a point on the line segment will
    // be the closest point.

    let t = start_to_point.dot(&line) / line.dot(&line);
    let t = t.clamp(0.0, 1.0);

    let closest_point_on_segment = line_start + t * line;
    return (point - closest_point_on_segment).length_squared();
}

/*
    Finds the shortest distance between a point and a line segment in 3D space.

    point: The point to find the distance to
    line_start: The start point of the line segment
    line_end: The end point of the line segment

    Returns the squared distance between the point and the line segment.
*/
fn distance_from_point_to_line_segment_squared(
    point: &[f32; 3],
    line_start: &[f32; 3],
    line_end: &[f32; 3],
) -> f64 {
    let line_start = Vec3::new(
        line_start[0] as f64,
        line_start[1] as f64,
        line_start[2] as f64,
    );
    let line_end = Vec3::new(line_end[0] as f64, line_end[1] as f64, line_end[2] as f64);
    let point = Vec3::new(point[0] as f64, point[1] as f64, point[2] as f64);

    return vec_distance_from_point_to_line_segment_squared(point, line_start, line_end);
}

#[cfg(test)]
mod tests {
    use crate::geometry::{LinePrimitive, Vertices};

    use super::*;

    #[test]
    fn test_get_shortest_distance_between_lines_that_are_parallel() {
        let mut a = LinePrimitive::new();
        a.set_vertices(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);

        let mut b = LinePrimitive::new();
        b.set_vertices(vec![[3.0, 0.0, 0.0], [3.0, 1.0, 0.0]]);

        let distance = get_shortest_distance_between_lines(&a, &b);
        assert!(distance.is_ok());
        assert_eq!(distance.unwrap(), 3.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_that_are_parallel_where_one_is_longer() {
        let mut a = LinePrimitive::new();
        a.set_vertices(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);

        let mut b = LinePrimitive::new();
        b.set_vertices(vec![[3.0, -10.0, 0.0], [3.0, 10.0, 0.0]]);

        let distance = get_shortest_distance_between_lines(&a, &b);
        assert!(distance.is_ok());
        // shortest distance should still be between line segments
        assert_eq!(distance.unwrap(), 3.0);
    }

    #[test]
    fn test_get_shortest_distance_between_parallel_lines_that_meet_at_a_vertex() {
        let mut a = LinePrimitive::new();
        a.set_vertices(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);

        let mut b = LinePrimitive::new();
        b.set_vertices(vec![[1.0, 1.0, 1.0], [2.0, 2.0, 2.0]]);

        let distance = get_shortest_distance_between_lines(&a, &b);
        assert!(distance.is_ok());
        assert_eq!(distance.unwrap(), 0.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_that_meet_at_a_vertex_but_are_not_parallel() {
        let mut a = LinePrimitive::new();
        a.set_vertices(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);

        let mut b = LinePrimitive::new();
        b.set_vertices(vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);

        let distance = get_shortest_distance_between_lines(&a, &b);
        assert!(distance.is_ok());
        assert_eq!(distance.unwrap(), 0.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_that_are_the_same() {
        let mut a = LinePrimitive::new();
        a.set_vertices(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);

        let mut b = LinePrimitive::new();
        b.set_vertices(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);

        let distance = get_shortest_distance_between_lines(&a, &b);
        assert!(distance.is_ok());
        assert_eq!(distance.unwrap(), 0.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_that_intersect() {
        let mut a = LinePrimitive::new();
        a.set_vertices(vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);

        let mut b = LinePrimitive::new();
        b.set_vertices(vec![[0.0, -1.0, 0.0], [0.0, 1.0, 0.0]]);

        let distance = get_shortest_distance_between_lines(&a, &b);
        assert!(distance.is_ok());
        // should intersect at (0, 0, 0)
        assert_eq!(distance.unwrap(), 0.0);
    }

    // TODO: this test is failing because we calculate non-parallel lines incorrectly
    // think we can rewrite the algorithm to use fewer comparisons by parameterising each line
    // individually and then clamping where required.
    // #[test]
    // fn test_get_shortest_distance_between_lines_where_line_edge_and_line_vertex_are_closest() {
    //     let mut a = LinePrimitive::new();
    //     a.set_vertices(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);

    //     let mut b = LinePrimitive::new();
    //     b.set_vertices(vec![[0.5, 1.0, 0.0], [1.0, 2.0, 0.0]]);

    //     let distance = get_shortest_distance_between_lines(&a, &b);
    //     assert!(distance.is_ok());
    //     // should be the distance between b's start and a's segment
    //     assert_eq!(distance.unwrap(), 1.0);
    // }
}
