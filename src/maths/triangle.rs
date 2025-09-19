use crate::maths::{line::closest_points_on_lines, vec::Vec3};

const TRIANGLE_VERTEX_COUNT: usize = 3;
const EPSILON: f64 = 1e-15;

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
    if let Some(closest_point_on_a) = maybe_shortest_distance.closest_point_found_on_a
        && let Some(closest_point_on_b) = maybe_shortest_distance.closest_point_found_on_b
    {
        return (closest_point_on_a - closest_point_on_b).length_squared();
    }
    // otherwise, we
    triangles_are_separated = triangles_are_separated || maybe_shortest_distance.separated;

    // step 3 (TODO: walk through algo just to prove step 3 is correct)
    // vertex of triangle a and face of triangle b, same as step 2 but with triangles swapped
    let maybe_shortest_distance = maybe_shortest_distance_between_face_of_a_and_vertex_of_b(
        &triangle_b,
        &triangle_a,
        &edges_b,
    );
    // if we found a pair of closest points, we can exit early
    if let Some(closest_point_on_a) = maybe_shortest_distance.closest_point_found_on_a
        && let Some(closest_point_on_b) = maybe_shortest_distance.closest_point_found_on_b
    {
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
