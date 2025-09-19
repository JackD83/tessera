use crate::maths::vec::Vec3;

// Finds the longest squared distance between two line segments in 3D space.
// This is effectively the adversarial part of the hausdorff distance.
// todo: add tests
pub fn longest_distance_between_lines_squared(
    a_start: &[f32; 3],
    a_end: &[f32; 3],
    b_start: &[f32; 3],
    b_end: &[f32; 3],
) -> f64 {
    // furthest point on A must be a vertex
    // if A and B are parallel, the orthogonal distance is the same
    // if A and B are collinear, it's just the distance between them from the furthest vertex
    // if neither, one of A's vertices must be further than the other, making it the furthest point
    let a_start_distance =
        shortest_distance_from_point_to_line_segment_squared(a_start, b_start, b_end);
    let a_end_distance =
        shortest_distance_from_point_to_line_segment_squared(a_end, b_start, b_end);

    return a_start_distance.max(a_end_distance);
}

/*
    Finds the shortest distance between two line segments in 3D space.

    a_start: The start point of the first line segment
    a_end: The end point of the first line segment
    b_start: The start point of the second line segment
    b_end: The end point of the second line segment

    Returns the squared shortest distance between the two line segments.
*/
pub fn shortest_line_distance_squared(
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

    let (closest_on_a, closest_on_b) = closest_points_on_lines(a_start, a_end, b_start, b_end);

    return (closest_on_a - closest_on_b).length_squared();
}

/*
    Finds the closest points on two line segments in 3D space.

    a_start: The start point of the first line segment
    a_end: The end point of the first line segment
    b_start: The start point of the second line segment
    b_end: The end point of the second line segment

    Returns the closest points on the two line segments.
*/
pub fn closest_points_on_lines(
    a_start: Vec3,
    a_end: Vec3,
    b_start: Vec3,
    b_end: Vec3,
) -> (Vec3, Vec3) {
    let a = a_end - a_start;
    let b = b_end - b_start;
    // Vector between start points
    let c = b_start - a_start;

    // Dot products
    let a_dot_a = a.dot(&a);
    let a_dot_b = a.dot(&b);
    let b_dot_b = b.dot(&b);
    let a_dot_c = a.dot(&c);
    let b_dot_c = b.dot(&c);

    let mut s: f64;
    let mut t: f64;
    /*

    All points on the line segments can be expressed as:
        A point on a => A(s) = a_start + s * (a_end - a_start) for 0 <= s <= 1
        A point on b => B(t) = b_start + t * (b_end - b_start) for 0 <= t <= 1

    Firstly, we find the closest point on the infinite versions of the provided line segments
    where S is unbounded, by solving for S like so:

    S = (b · b) (a · c) - (a · b) (b · c)
        ---------------------------------
          (a · a) (b · b) - (a · b)^2

    by clamping S to [0-1], we ensure that S is the closest point from the first line segment A
    to the unbounded second line B.

    We can then derive T from S like so:

    T = S (a · b) - (b · c)
        -----------------
            (b · b)

    If T is inside the range [0-1], S and T are the closest points.

    If T is outside the range [0-1], it is not on the segment B, so we clamp it to [0-1] and
    recompute S, which we can then re-clamp to [0-1] to find the closest point on the first line
    segment A.
    */

    let denominator = a_dot_a * b_dot_b - a_dot_b * a_dot_b;

    // TODO: can we just check for zero here or do we need the epsilon?
    if denominator != 0.0 {
        // lines are not parallel, calculate parameter S for first line segment
        s = (b_dot_b * a_dot_c - a_dot_b * b_dot_c) / denominator;
        s = s.clamp(0.0, 1.0);
    } else {
        // lines are parallel, so pick an arbitrary point on the first line
        s = 0.0;
    }

    // find t for second line segment, closest to s
    if b_dot_b != 0.0 {
        t = (s * a_dot_b - b_dot_c) / b_dot_b;

        // if t is on the second line segment, s and t are closest points.
        // if not, clamp t and recompute s, then reclamp s.
        if t < 0.0 {
            // t is before the start of the second line segment
            t = 0.0;
            if a_dot_a != 0.0 {
                s = a_dot_c / a_dot_a;
                s = s.clamp(0.0, 1.0);
            } else {
                s = 0.0;
            }
        } else if t > 1.0 {
            // t is after the end of the second line segment
            t = 1.0;
            if a_dot_a != 0.0 {
                s = (a_dot_b + a_dot_c) / a_dot_a;
                s = s.clamp(0.0, 1.0);
            } else {
                s = 0.0;
            }
        }
    } else {
        // lines are parallel? so pick an arbitrary point on the second line
        t = 0.0;
        if a_dot_a != 0.0 {
            s = a_dot_c / a_dot_a;
            s = s.clamp(0.0, 1.0);
        } else {
            s = 0.0;
        }
    }

    // calculate closest points
    let closest_on_a = a_start + s * a;
    let closest_on_b = b_start + t * b;
    return (closest_on_a, closest_on_b);
}

/*
    Finds the shortest distance between a point and a line segment in 3D space.

    point: The point to find the distance to
    line_start: The start point of the line segment
    line_end: The end point of the line segment

    Returns the squared distance between the point and the line segment.
*/
pub fn shortest_distance_from_point_to_line_segment_squared(
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

    return shortest_distance_from_point_to_line_segment_squared_vec(point, line_start, line_end);
}

/*
    Finds the shortest distance between a point and a line segment in 3D space.

    point: The point to find the distance to
    line_start: The start point of the line segment
    line_end: The end point of the line segment

    Returns the squared distance between the point and the line segment.
*/
pub fn shortest_distance_from_point_to_line_segment_squared_vec(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_shortest_distance_between_lines_that_are_parallel() {
        let (a_start, a_end) = ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let (b_start, b_end) = ([3.0, 0.0, 0.0], [3.0, 1.0, 0.0]);

        let distance_squared = shortest_line_distance_squared(&a_start, &a_end, &b_start, &b_end);
        assert_eq!(distance_squared.sqrt(), 3.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_that_are_parallel_where_one_is_longer() {
        let (a_start, a_end) = ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let (b_start, b_end) = ([3.0, -10.0, 0.0], [3.0, 10.0, 0.0]);

        let distance_squared = shortest_line_distance_squared(&a_start, &a_end, &b_start, &b_end);
        // shortest distance should still be between line segments
        assert_eq!(distance_squared.sqrt(), 3.0);
    }

    #[test]
    fn test_get_shortest_distance_between_collinear_lines_that_meet_at_a_vertex() {
        let (a_start, a_end) = ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let (b_start, b_end) = ([1.0, 1.0, 1.0], [2.0, 2.0, 2.0]);

        let distance_squared = shortest_line_distance_squared(&a_start, &a_end, &b_start, &b_end);
        assert_eq!(distance_squared.sqrt(), 0.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_that_meet_at_a_vertex_but_are_not_parallel() {
        let (a_start, a_end) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let (b_start, b_end) = ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

        let distance_squared = shortest_line_distance_squared(&a_start, &a_end, &b_start, &b_end);
        assert_eq!(distance_squared.sqrt(), 0.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_that_are_the_same() {
        let (a_start, a_end) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let (b_start, b_end) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        let distance_squared = shortest_line_distance_squared(&a_start, &a_end, &b_start, &b_end);
        assert_eq!(distance_squared.sqrt(), 0.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_that_intersect_orthogonally() {
        let (a_start, a_end) = ([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let (b_start, b_end) = ([0.0, -1.0, 0.0], [0.0, 1.0, 0.0]);

        let distance_squared = shortest_line_distance_squared(&a_start, &a_end, &b_start, &b_end);
        // should intersect at (0, 0, 0)
        assert_eq!(distance_squared.sqrt(), 0.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_that_intersect_but_are_not_orthogonal() {
        let (a_start, a_end) = ([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let (b_start, b_end) = ([-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);

        let distance_squared = shortest_line_distance_squared(&a_start, &a_end, &b_start, &b_end);
        // should intersect at (0, 0, 0)
        assert_eq!(distance_squared.sqrt(), 0.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_where_line_edge_and_line_vertex_are_closest() {
        let (a_start, a_end) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let (b_start, b_end) = ([0.5, 1.0, 0.0], [1.0, 2.0, 0.0]);

        let distance_squared = shortest_line_distance_squared(&a_start, &a_end, &b_start, &b_end);
        // should be the distance between b's start and a's segment
        assert_eq!(distance_squared.sqrt(), 1.0);
    }

    #[test]
    fn test_get_shortest_distance_between_lines_where_vertices_are_closest() {
        let (a_start, a_end) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let (b_start, b_end) = ([5.0, 0.0, 0.0], [10.0, 0.0, 0.0]);

        let distance_squared = shortest_line_distance_squared(&a_start, &a_end, &b_start, &b_end);
        assert_eq!(distance_squared.sqrt(), 4.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_line_segment_squared() {
        let point = [0.5, 1.0, 0.0];
        let (line_start, line_end) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        let distance_squared =
            shortest_distance_from_point_to_line_segment_squared(&point, &line_start, &line_end);
        assert_eq!(distance_squared.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_line_segment_squared_where_point_is_segment_vertex() {
        let point = [0.0, 0.0, 0.0];
        let (line_start, line_end) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        let distance_squared =
            shortest_distance_from_point_to_line_segment_squared(&point, &line_start, &line_end);
        assert_eq!(distance_squared.sqrt(), 0.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_line_segment_squared_where_point_is_on_segment() {
        let point = [0.5, 0.0, 0.0];
        let (line_start, line_end) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        let distance_squared =
            shortest_distance_from_point_to_line_segment_squared(&point, &line_start, &line_end);
        assert_eq!(distance_squared.sqrt(), 0.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_line_segment_squared_where_point_is_outside_segment_but_collinear()
     {
        let point = [2.0, 0.0, 0.0];
        let (line_start, line_end) = ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        let distance_squared =
            shortest_distance_from_point_to_line_segment_squared(&point, &line_start, &line_end);
        assert_eq!(distance_squared.sqrt(), 1.0);
    }
}
