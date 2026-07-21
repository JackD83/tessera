use crate::maths::{
    line::{
        closest_points_on_lines, shortest_distance_from_point_to_line_segment_squared,
        shortest_distance_from_point_to_line_segment_squared_vec,
    },
    vec::Vec3,
};

const TRIANGLE_VERTEX_COUNT: usize = 3;
const EPSILON: f64 = 1e-15;

// Finds the longest squared distance between a line segment and a triangle in 3D space.
// This is effectively the adversarial part of the hausdorff distance.
// todo: add tests, formalise better in docstrings
pub fn longest_distance_between_line_segment_and_triangle_squared(
    line_start: &[f32; 3],
    line_end: &[f32; 3],
    triangle_a: &[f32; 3],
    triangle_b: &[f32; 3],
    triangle_c: &[f32; 3],
) -> f64 {
    // the longest distance traversed from a line segment to a triangle must be from one of the line segment's
    // vertices to some point on the triangle.
    // if the line segment and triangle are parallel, the orthogonal distance is the same everywhere
    // if the line segment and triangle are not parallel, it's just the distance between them from the furthest vertex
    let line_start_distance = shortest_distance_from_point_to_triangle_squared(
        line_start, triangle_a, triangle_b, triangle_c,
    );
    let line_end_distance = shortest_distance_from_point_to_triangle_squared(
        line_end, triangle_a, triangle_b, triangle_c,
    );

    return line_start_distance.max(line_end_distance);
}

// Finds the longest squared distance between a line segment and a triangle in 3D space.
// This is effectively the adversarial part of the hausdorff distance.
// todo: add tests, formalise better in docstrings
pub fn longest_distance_between_triangle_and_line_segment_squared(
    triangle_a: &[f32; 3],
    triangle_b: &[f32; 3],
    triangle_c: &[f32; 3],
    line_start: &[f32; 3],
    line_end: &[f32; 3],
) -> f64 {
    // the longest distance traversed from a triangle to a line segment should be from one of the triangle's
    // vertices to some point on the line segment.
    // if the triangle and line segment are parallel, the orthogonal distance is the same everywhere
    // if the triangle and line segment are not parallel, it's just the distance between them from the furthest vertex
    let triangle_a_distance =
        shortest_distance_from_point_to_line_segment_squared(triangle_a, line_start, line_end);
    let triangle_b_distance =
        shortest_distance_from_point_to_line_segment_squared(triangle_b, line_start, line_end);
    let triangle_c_distance =
        shortest_distance_from_point_to_line_segment_squared(triangle_c, line_start, line_end);

    return triangle_a_distance
        .max(triangle_b_distance)
        .max(triangle_c_distance);
}

// Finds the longest shortest squared distance between two triangles in 3D space.
// This is effectively the adversarial part of the hausdorff distance.
// todo: add tests, formalise better in docstrings
pub fn longest_distance_between_triangles_squared(
    a_a: &[f32; 3],
    a_b: &[f32; 3],
    a_c: &[f32; 3],
    b_a: &[f32; 3],
    b_b: &[f32; 3],
    b_c: &[f32; 3],
) -> f64 {
    // the longest distance traversed from a triangle a to triangle b must be from one of triangle a's
    // vertices to some point on triangle b.
    // if A and B are parallel, the orthogonal distance is the same everywhere
    // if A and B are not parallel, it's just the distance between them from the furthest vertex
    let a_a_distance = shortest_distance_from_point_to_triangle_squared(a_a, b_a, b_b, b_c);
    let a_b_distance = shortest_distance_from_point_to_triangle_squared(a_b, b_a, b_b, b_c);
    let a_c_distance = shortest_distance_from_point_to_triangle_squared(a_c, b_a, b_b, b_c);

    return a_a_distance.max(a_b_distance).max(a_c_distance);
}

// Finds the shortest squared distance between a point and a triangle in 3D space.
pub fn shortest_distance_from_point_to_triangle_squared(
    point: &[f32; 3],
    triangle_a: &[f32; 3],
    triangle_b: &[f32; 3],
    triangle_c: &[f32; 3],
) -> f64 {
    let point = Vec3::from_array(point);
    let triangle = [
        Vec3::from_array(triangle_a),
        Vec3::from_array(triangle_b),
        Vec3::from_array(triangle_c),
    ];

    if let Some(distance_to_plane) =
        get_point_projected_to_triangle_distance_squared_if_inside(point, &triangle)
    {
        return distance_to_plane;
    }

    // the closest point is on one of the edges of the triangle, we just need to find which
    let point_to_ab =
        shortest_distance_from_point_to_line_segment_squared_vec(point, triangle[0], triangle[1]);
    let point_to_bc =
        shortest_distance_from_point_to_line_segment_squared_vec(point, triangle[1], triangle[2]);
    let point_to_ca =
        shortest_distance_from_point_to_line_segment_squared_vec(point, triangle[2], triangle[0]);

    return point_to_ab.min(point_to_bc).min(point_to_ca);
}

fn get_point_projected_to_triangle_distance_squared_if_inside(
    point: Vec3,
    triangle: &[Vec3; 3],
) -> Option<f64> {
    let ab = triangle[1] - triangle[0];
    let ac = triangle[2] - triangle[0];
    let ap = point - triangle[0];

    // project p onto the plane of the triangle
    let normal = ab.cross(&ac).normalize();
    let distance_to_plane = ap.dot(&normal);
    let projected_point = point - normal * distance_to_plane;

    // check if the projected point is inside the triangle
    // we want to find U and V such that: projected_point = a + u * ab + v * ac
    // which simplifies to: (projected_point - a) = u * ab + v * ac
    // using dot products, we can solve for u and v via 2 linear equations
    let a_to_projected_point = projected_point - triangle[0];
    let d00 = ab.dot(&ab);
    let d01 = ab.dot(&ac);
    let d11 = ac.dot(&ac);
    let d20 = a_to_projected_point.dot(&ab);
    let d21 = a_to_projected_point.dot(&ac);

    // solve for u and v
    let denom = d00 * d11 - d01 * d01;
    let u = (d11 * d20 - d01 * d21) / denom;
    let v = (d00 * d21 - d01 * d20) / denom;

    // if u and v are positive and their sum is less than or equal to 1, the point is inside the triangle
    if u >= 0.0 && v >= 0.0 && u + v <= 1.0 {
        // we already have the squared value, but it's cheaper than always
        // sqrting the alternative path which is more common, so we normalise here.
        return Some(distance_to_plane.powi(2));
    }
    // else it's outside the triangle
    return None;
}

/*
    Finds the shortest squared distance between a line segment and a triangle in 3D space.

    The algorithm checks for the following:
    1. If the segment intersects the triangle, distance is 0.
    2. The shortest distance between an edge of the triangle and the segment
    3. The shortest distance between a vertex of the segment and the face of the triangle
*/
pub fn shortest_distance_from_line_segment_to_triangle_squared(
    line_start: &[f32; 3],
    line_end: &[f32; 3],
    triangle_a: &[f32; 3],
    triangle_b: &[f32; 3],
    triangle_c: &[f32; 3],
) -> f64 {
    let p = Vec3::from_array(line_start);
    let q = Vec3::from_array(line_end);
    let triangle = [
        Vec3::from_array(triangle_a),
        Vec3::from_array(triangle_b),
        Vec3::from_array(triangle_c),
    ];

    // vectors for the segment (line PQ) and triangle edges (AB, BC, AC)
    let pq = q - p;
    let ab = triangle[1] - triangle[0];
    let ac = triangle[2] - triangle[0];
    let _bc = triangle[2] - triangle[1];
    let ap = p - triangle[0];
    let aq = q - triangle[0];

    // step 1
    // Pre-calculate dot products needed for barycentric coordinate calculation.
    // Barycentric coordinates (u,v,w) are used to represent any point on the triangle's
    // plane as a weighted average of its vertices: P = u*a + v*b + w*c.
    let ab_dot_ab = ab.dot(&ab);
    let ab_dot_ac = ab.dot(&ac);
    let ac_dot_ac = ac.dot(&ac);
    let barycentric_denominator = ab_dot_ab * ac_dot_ac - ab_dot_ac * ab_dot_ac;
    let reciprocal_barycentric_denominator = if barycentric_denominator > 0.0 {
        1.0 / barycentric_denominator
    } else {
        0.0
    };

    let triangle_normal = ab.cross(&ac).normalize();

    // step 2
    // check for direct intersection between the segment and the triangle
    let p_distance_to_plane = ap.dot(&triangle_normal);
    let q_distance_to_plane = aq.dot(&triangle_normal);

    // if distances have opposite signs, they lie on opposite sides of the plane
    // and thus the segment intersects the triangle's plane
    if p_distance_to_plane * q_distance_to_plane <= 0.0 {
        // find the point where segment intersects the triangle's plane
        // where t is the segment parameter such that intersection = p + t * pq
        let t = -ap.dot(&triangle_normal) / pq.dot(&triangle_normal);
        let intersection_point = p + t * pq;

        // find barycentric coordinates of intersection point to determine if inside triangle
        let intersection_to_a = intersection_point - triangle[0];
        let intersection_dot_ab = intersection_to_a.dot(&ab);
        let intersection_dot_ac = intersection_to_a.dot(&ac);

        let barycentric_v = ac_dot_ac * intersection_dot_ab
            - ab_dot_ac * intersection_dot_ac * reciprocal_barycentric_denominator;
        let barycentric_w = ab_dot_ab * intersection_dot_ac
            - ab_dot_ab * intersection_dot_ab * reciprocal_barycentric_denominator;

        if barycentric_v >= 0.0 && barycentric_w >= 0.0 && barycentric_v + barycentric_w <= 1.0 {
            // intersection point is inside the triangle
            return 0.0;
        }
    }

    // step 3
    // check for distances to triangle edges
    // if no intersection with the triangle, the closest point will be on the boundary (edge or vertex)
    let edge_closest_points = [
        closest_points_on_lines(triangle[0], triangle[1], p, q), // ab -> pq
        closest_points_on_lines(triangle[1], triangle[2], p, q), // bc -> pq
        closest_points_on_lines(triangle[2], triangle[0], p, q), // ca -> pq
    ];
    let closest_edge_distance = edge_closest_points
        .iter()
        .map(|(a, b)| (*a - *b).length_squared())
        .min_by(|a, b| a.total_cmp(b))
        .expect("Tried to compare a NaN value, this should never happen unless a vertex was NaN");

    // step 4
    // check for distances between a segment vertex and the triangle face
    // by projecting the endpoints onto the plane of the triangle, and seeing if it is inside the triangle
    let p_distance_to_face_if_projects_inside =
        get_point_projected_to_triangle_distance_squared_if_inside(p, &triangle);
    let q_distance_to_face_if_projects_inside =
        get_point_projected_to_triangle_distance_squared_if_inside(q, &triangle);

    // step 5
    // find the shortest distance from the previous calculations
    // either:
    // 1. segment <-> triangle edge,
    // 2. p <-> projection of p on triangle face if p projects inside,
    // 3. q <-> projection of q on triangle face if q projects inside,
    let shortest_distance = [
        closest_edge_distance,
        p_distance_to_face_if_projects_inside.unwrap_or(f64::INFINITY),
        q_distance_to_face_if_projects_inside.unwrap_or(f64::INFINITY),
    ]
    .into_iter()
    .min_by(|a, b| a.total_cmp(b))
    .expect("Tried to compare a NaN value, this should never happen unless a vertex was NaN");

    return shortest_distance;
}

/*
    Finds the shortest squared distance between two triangles in 3D space.

    a_a: The first vertex of the first triangle
    a_b: The second vertex of the first triangle
    a_c: The third vertex of the first triangle
    b_a: The first vertex of the second triangle
    b_b: The second vertex of the second triangle
    b_c: The third vertex of the second triangle

    Implements the Separating Axis Theorem by checking for separation along various axes.
    If no separation is found, the triangles are intersecting.

    The algorithm checks three main cases for the closest features:
    1. An edge of Triangle A and an edge of Triangle B.
    2. A vertex of Triangle B and the face of Triangle A.
    3. A vertex of Triangle A and the face of Triangle B.
*/
pub fn shortest_triangle_distance_squared(
    a_a: &[f32; 3],
    a_b: &[f32; 3],
    a_c: &[f32; 3],
    b_a: &[f32; 3],
    b_b: &[f32; 3],
    b_c: &[f32; 3],
) -> f64 {
    let triangle_a = [
        Vec3::from_array(a_a),
        Vec3::from_array(a_b),
        Vec3::from_array(a_c),
    ];
    let triangle_b = [
        Vec3::from_array(b_a),
        Vec3::from_array(b_b),
        Vec3::from_array(b_c),
    ];

    // edges of triangle A
    let edges_a = [
        triangle_a[1] - triangle_a[0],
        triangle_a[2] - triangle_a[1],
        triangle_a[0] - triangle_a[2],
    ];

    // edges of triangle B
    let edges_b = [
        triangle_b[1] - triangle_b[0],
        triangle_b[2] - triangle_b[1],
        triangle_b[0] - triangle_b[2],
    ];

    // we must initialise these in the unlikely case the distance is always greater than infinity
    // in which case, we will return 0 as the error is incalculable in that case.
    let mut min_distance_closest_point_a: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    let mut min_distance_closest_point_b: Vec3 = Vec3::new(0.0, 0.0, 0.0);

    let mut min_squared_distance = f64::INFINITY;
    let mut triangles_are_separated = false;

    // step 1
    // edge-edge distance checks, these are the most common for non-intersecting triangles
    for i in 0..TRIANGLE_VERTEX_COUNT {
        for j in 0..TRIANGLE_VERTEX_COUNT {
            // iterating pairs of vertices (0 -> 1, 1 -> 2, 2 -> 0) or AB, BC, CA
            let (closest_on_a, closest_on_b) = closest_points_on_lines(
                triangle_a[i],
                triangle_a[(i + 1) % TRIANGLE_VERTEX_COUNT],
                triangle_b[j],
                triangle_b[(j + 1) % TRIANGLE_VERTEX_COUNT],
            );

            let vec_between_closest_points = closest_on_b - closest_on_a;
            let squared_distance = vec_between_closest_points.length_squared();

            if squared_distance <= min_squared_distance {
                min_squared_distance = squared_distance;
                min_distance_closest_point_a = closest_on_a;
                min_distance_closest_point_b = closest_on_b;

                // voronoi region optimisation for early termination
                // we check if the triangles lie in separate voronoi regions as
                // defined by the plane perpendicular to the vector between the closest
                // points so far.

                // get third vertex of triangle A
                let third_triangle_vertex_a = triangle_a[(i + 2) % TRIANGLE_VERTEX_COUNT];
                let vec_to_third_triangle_vertex_a =
                    third_triangle_vertex_a - min_distance_closest_point_a;
                let mut dot_a = vec_to_third_triangle_vertex_a.dot(&vec_between_closest_points);

                // get third vertex of triangle B
                let third_triangle_vertex_b = triangle_b[(j + 2) % TRIANGLE_VERTEX_COUNT];
                let vec_to_third_triangle_vertex_b =
                    third_triangle_vertex_b - min_distance_closest_point_b;
                let mut dot_b = vec_to_third_triangle_vertex_b.dot(&vec_between_closest_points);

                // if dot_a is negative and dot_b is positive, the triangles are on opposite
                // sides of the separating plane, and we have found the true minimum distance.
                if dot_a <= 0.0 && dot_b >= 0.0 {
                    return min_squared_distance;
                }

                // triangles may still be provably disjoint, even if above check is false
                dot_a = dot_a.max(0.0); // clamp dot_a to at least 0
                dot_b = dot_b.min(0.0); // clamp dot_b to at most 0

                if (min_squared_distance - dot_a + dot_b) > 0.0 {
                    triangles_are_separated = true;
                }
            }
        }
    }

    // step 2
    // vertex of triangle b and face of triangle a
    let maybe_shortest_distance = maybe_shortest_distance_between_face_of_a_and_vertex_of_b(
        &triangle_a,
        &triangle_b,
        &edges_a,
    );
    // if we found a pair of closest points, we can exit early
    if let (Some(closest_point_on_a), Some(closest_point_on_b)) = (
        maybe_shortest_distance.closest_point_found_on_a,
        maybe_shortest_distance.closest_point_found_on_b,
    ) {
        return (closest_point_on_a - closest_point_on_b).length_squared();
    }
    triangles_are_separated = triangles_are_separated || maybe_shortest_distance.separated;

    // step 3 (TODO: walk through algo just to prove step 3 is correct)
    // vertex of triangle a and face of triangle b, same as step 2 but with triangles swapped
    let maybe_shortest_distance = maybe_shortest_distance_between_face_of_a_and_vertex_of_b(
        &triangle_b,
        &triangle_a,
        &edges_b,
    );
    // if we found a pair of closest points, we can exit early
    if let (Some(closest_point_on_a), Some(closest_point_on_b)) = (
        maybe_shortest_distance.closest_point_found_on_a,
        maybe_shortest_distance.closest_point_found_on_b,
    ) {
        return (closest_point_on_a - closest_point_on_b).length_squared();
    }
    triangles_are_separated = triangles_are_separated || maybe_shortest_distance.separated;

    // step 4
    // final result, if the separatation checks succeeded, the closest points must
    // be the ones found in the edge-edge checks
    if triangles_are_separated {
        return (min_distance_closest_point_a - min_distance_closest_point_b).length_squared();
    }

    // if no separation was found, the triangles must be intersecting and thus the
    // distance is 0.
    return 0.0;
}

struct MaybeShortestDistanceBetweenFaceOfAAndVertexOfB {
    // the closest point found on triangle a, if the closest vertex of b was projected onto the plane of a
    // and landed inside the triangle.
    closest_point_found_on_a: Option<Vec3>,

    // the closest point found on triangle b, if the closest vertex of b was projected onto the plane of a
    // and landed inside the triangle.
    closest_point_found_on_b: Option<Vec3>,

    // whether the triangles are separated
    separated: bool,
}

/*
    Checks if the shortest distance between two triangles is between the
    face of triangle a and a vertex of triangle b, returning the distance if so.

    Returns two values:
        - The shortest distance squared, or None if the distance is not guaranteed to be the shortest.
        - A boolean indicating if the triangles are separated.
*/
fn maybe_shortest_distance_between_face_of_a_and_vertex_of_b(
    triangle_a: &[Vec3; 3],
    triangle_b: &[Vec3; 3],
    edges_a: &[Vec3; 3],
) -> MaybeShortestDistanceBetweenFaceOfAAndVertexOfB {
    let normal_a = edges_a[0].cross(&edges_a[1]);
    let squared_normal_length = normal_a.length_squared();
    let mut triangles_are_separated = false;

    // check triangle a is not degenerate
    if squared_normal_length > EPSILON {
        // signed distances from each vertex of b to plane of a
        let signed_distances_to_plane_a = [
            (triangle_a[0] - triangle_b[0]).dot(&normal_a),
            (triangle_a[0] - triangle_b[1]).dot(&normal_a),
            (triangle_a[0] - triangle_b[2]).dot(&normal_a),
        ];

        // check if all vertices of b are on same side of plane of a
        let all_positive = signed_distances_to_plane_a.iter().all(|&d| d > 0.0);
        let all_negative = signed_distances_to_plane_a.iter().all(|&d| d < 0.0);

        if all_positive || all_negative {
            triangles_are_separated = true;

            // find the vertex of b that is closest to the plane of a
            let closest_index_on_b = signed_distances_to_plane_a
                .iter()
                .enumerate()
                // take value closest to 0
                .min_by(|(_, a), &(_, b)| a.abs().total_cmp(&b.abs()))
                // and select it's index
                .map(|(index, _)| index)
                .expect("Tried to compare a NaN value, this should never happen unless a vertex was NaN");
            let closest_vertex_on_b = triangle_b[closest_index_on_b];

            // then check if projecting the closest vertex of b onto the plane of a
            // lands inside the triangle by checking if it is on the inner side of all edges of a
            // after projection.
            let mut is_inside = true;
            for i in 0..TRIANGLE_VERTEX_COUNT {
                let edge_plane_normal = normal_a.cross(&edges_a[i]);
                let vec_to_vertex = closest_vertex_on_b - triangle_a[i];

                if vec_to_vertex.dot(&edge_plane_normal) <= 0.0 {
                    is_inside = false;
                    break;
                }
            }

            // if it was inside, this is the closest point
            if is_inside {
                let closest_point_on_a = closest_vertex_on_b
                    + normal_a
                        * (signed_distances_to_plane_a[closest_index_on_b] / squared_normal_length);
                let closest_point_on_b = closest_vertex_on_b;

                return MaybeShortestDistanceBetweenFaceOfAAndVertexOfB {
                    closest_point_found_on_a: Some(closest_point_on_a),
                    closest_point_found_on_b: Some(closest_point_on_b),
                    separated: triangles_are_separated,
                };
            }
        }
    }

    return MaybeShortestDistanceBetweenFaceOfAAndVertexOfB {
        closest_point_found_on_a: None,
        closest_point_found_on_b: None,
        separated: triangles_are_separated,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortest_distance_from_point_to_triangle_squared_where_point_is_a_vertex() {
        let point = [0.0, 0.0, 0.0];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_point_to_triangle_squared(
            &point,
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 0.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_triangle_squared_where_point_is_on_an_edge() {
        let point = [0.5, 0.0, 0.0];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_point_to_triangle_squared(
            &point,
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 0.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_triangle_squared_where_point_is_inside_the_triangle() {
        let point = [0.2, 0.2, 0.0];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_point_to_triangle_squared(
            &point,
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 0.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_triangle_squared_where_coplanar_point_is_outside_triangle_closest_to_vertex()
     {
        let point = [2.0, 0.0, 0.0];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_point_to_triangle_squared(
            &point,
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_triangle_squared_where_coplanar_point_is_outside_triangle_closest_to_edge()
     {
        let point = [0.5, -1.0, 0.0];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_point_to_triangle_squared(
            &point,
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_triangle_squared_where_point_projects_inside_triangle()
    {
        let point = [0.2, 0.2, 1.0];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_point_to_triangle_squared(
            &point,
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_triangle_squared_where_point_projects_outside_triangle_closest_to_vertex()
     {
        let point = [4.0, 0.0, 4.0];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_point_to_triangle_squared(
            &point,
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 5.0);
    }

    #[test]
    fn test_shortest_distance_from_point_to_triangle_squared_where_point_projects_outside_triangle_closest_to_edge()
     {
        let point = [0.5, -3.0, 4.0];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_point_to_triangle_squared(
            &point,
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 5.0);
    }

    #[test]
    fn test_shortest_distance_from_line_segment_to_triangle_squared_where_segment_is_an_edge() {
        let segment = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_line_segment_to_triangle_squared(
            &segment[0],
            &segment[1],
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 0.0);
    }

    #[test]
    fn test_shortest_distance_from_line_segment_to_triangle_squared_where_segment_is_a_vertex() {
        let segment = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_line_segment_to_triangle_squared(
            &segment[0],
            &segment[1],
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 0.0);
    }

    #[test]
    fn test_shortest_distance_from_line_segment_to_triangle_squared_where_parallel() {
        let segment = [[-1.0, 0.0, 1.0], [2.0, 0.0, 1.0]];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_line_segment_to_triangle_squared(
            &segment[0],
            &segment[1],
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_line_segment_to_triangle_squared_where_segment_projects_inside_triangle()
     {
        let segment = [[0.4, 0.2, 1.0], [0.6, 0.2, 1.0]];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_line_segment_to_triangle_squared(
            &segment[0],
            &segment[1],
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_line_segment_to_triangle_squared_where_segment_vertex_projects_inside_triangle()
     {
        let segment = [[0.4, 0.2, 1.0], [2.0, 0.2, 3.0]];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_line_segment_to_triangle_squared(
            &segment[0],
            &segment[1],
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_line_segment_to_triangle_squared_where_segment_edge_closest_to_triangle_vertex()
     {
        let segment = [[-1.0, 2.0, 0.0], [1.0, 2.0, 0.0]];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_line_segment_to_triangle_squared(
            &segment[0],
            &segment[1],
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_line_segment_to_triangle_squared_where_segment_edge_closest_to_triangle_edge()
     {
        let segment = [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_line_segment_to_triangle_squared(
            &segment[0],
            &segment[1],
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_line_segment_to_triangle_squared_where_segment_vertex_closest_to_triangle_vertex()
     {
        let segment = [[2.0, 0.0, 0.0], [4.0, 0.0, 0.0]];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_line_segment_to_triangle_squared(
            &segment[0],
            &segment[1],
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_distance_from_line_segment_to_triangle_squared_where_segment_vertex_closest_to_triangle_edge()
     {
        let segment = [[0.5, -1.0, 0.0], [0.5, -2.0, 0.0]];
        let triangle = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let distance = shortest_distance_from_line_segment_to_triangle_squared(
            &segment[0],
            &segment[1],
            &triangle[0],
            &triangle[1],
            &triangle[2],
        );
        assert_eq!(distance.sqrt(), 1.0);
    }

    #[test]
    fn test_shortest_triangle_distance_where_vertex_is_closest() {
        let a_a = [0.0, 0.0, 0.0];
        let a_b = [0.0, 1.0, 0.0];
        let a_c = [1.0, 0.5, 0.0];
        let b_a = [10.0, 0.0, 0.0];
        let b_b = [10.0, 1.0, 0.0];
        let b_c = [9.0, 0.5, 0.0];

        let distance = shortest_triangle_distance_squared(&a_a, &a_b, &a_c, &b_a, &b_b, &b_c);
        assert_eq!(distance.sqrt(), 8.0);
    }

    #[test]
    fn test_shortest_triangle_distance_where_edge_is_closest() {
        let a_a = [0.0, 0.0, 0.0];
        let a_b = [1.0, 1.0, 0.0];
        let a_c = [1.0, 0.0, 0.0];
        let b_a = [10.0, 0.0, 0.0];
        let b_b = [9.0, 1.0, 0.0];
        let b_c = [9.0, 0.0, 0.0];

        let distance = shortest_triangle_distance_squared(&a_a, &a_b, &a_c, &b_a, &b_b, &b_c);
        assert_eq!(distance.sqrt(), 8.0);
    }

    #[test]
    fn test_shortest_triangle_distance_with_identical_triangles() {
        let a_a = [0.0, 0.0, 0.0];
        let a_b = [1.0, 0.0, 0.0];
        let a_c = [0.0, 1.0, 0.0];
        let b_a = [0.0, 0.0, 0.0];
        let b_b = [1.0, 0.0, 0.0];
        let b_c = [0.0, 1.0, 0.0];

        let distance = shortest_triangle_distance_squared(&a_a, &a_b, &a_c, &b_a, &b_b, &b_c);
        assert_eq!(distance.sqrt(), 0.0);
    }

    #[test]
    fn test_shortest_triangle_distance_with_coplanar_triangles() {
        let a_a = [0.0, 0.0, 0.0];
        let a_b = [1.0, 0.0, 0.0];
        let a_c = [0.0, 1.0, 0.0];
        let b_a = [5.0, 0.0, 0.0];
        let b_b = [6.0, 0.0, 0.0];
        let b_c = [5.0, 6.0, 0.0];

        // distance should be between edge a_BC and vertex b_a
        let distance = shortest_triangle_distance_squared(&a_a, &a_b, &a_c, &b_a, &b_b, &b_c);
        assert_eq!(distance.sqrt(), 4.0);
    }

    #[test]
    fn test_shortest_triangle_distance_with_triangles_that_are_parallel() {
        let a_a = [0.0, 0.0, 0.0];
        let a_b = [1.0, 0.0, 0.0];
        let a_c = [0.0, 1.0, 0.0];
        let b_a = [0.0, 0.0, 3.0];
        let b_b = [1.0, 0.0, 3.0];
        let b_c = [0.0, 1.0, 3.0];

        // distance should be equal along z-axis
        let distance = shortest_triangle_distance_squared(&a_a, &a_b, &a_c, &b_a, &b_b, &b_c);
        assert_eq!(distance.sqrt(), 3.0);
    }
}
