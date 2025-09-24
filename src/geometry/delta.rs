use crate::{
    error::TesseraError,
    geometry::{LinePrimitive, PointPrimitive, TrianglePrimitive},
    maths::{
        line::{
            longest_distance_between_lines_squared,
            shortest_distance_from_point_to_line_segment_squared, shortest_line_distance_squared,
        },
        point::point_distance_squared,
        triangle::{
            longest_distance_between_line_segment_and_triangle_squared,
            longest_distance_between_triangles_squared,
            shortest_distance_from_line_segment_to_triangle_squared,
            shortest_distance_from_point_to_triangle_squared, shortest_triangle_distance_squared,
        },
    },
};

// Calculates the maximum perceptible delta between two sets of point primitives.
//
// We calculate the error introduced if a leaf point primitive was replaced by a
// simplified parent point primitive, aka the geometric error.
//
// It is important to note that this is a directional algorithm meaning that the results
// will be different if the order of the primitives is swapped.
pub fn get_renderable_delta_between_points(
    leaf: &PointPrimitive,
    parent: &PointPrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for a_point in leaf.iter_vertices() {
        let mut closest_distance = f64::INFINITY;

        for b_point in parent.iter_vertices() {
            let distance = point_distance_squared(&a_point, &b_point);

            closest_distance = closest_distance.min(distance);
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

// Calculates the maximum perceptible delta between a point primitive and a line primitive.
//
// We calculate the error introduced if a leaf point primitive was replaced by a
// simplified parent line primitive, aka the geometric error.
//
// This sound unintuitive, but it's possible that a set of points could be simplified to a line
//
// It is important to note that this is a directional algorithm meaning that the results
// will be different if the order of the primitives is swapped.
pub fn get_renderable_delta_between_point_and_line(
    leaf: &PointPrimitive,
    parent: &LinePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for a_point in leaf.iter_vertices() {
        let mut closest_distance = f64::INFINITY;

        for (b_start, b_end) in parent.iter_vertices() {
            let distance =
                shortest_distance_from_point_to_line_segment_squared(a_point, b_start, b_end);

            // because a point occupies no space, any of the nearest matches will have the same distance
            closest_distance = closest_distance.min(distance);
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

// Calculates the maximum perceptible delta between a point primitive and a triangle primitive.
//
// We calculate the error introduced if a leaf point primitive was replaced by a
// simplified parent triangle primitive, aka the geometric error.
//
// This sound unintuitive, but it's possible that a set of points could be simplified to a triangle
//
// It is important to note that this is a directional algorithm meaning that the results
// will be different if the order of the primitives is swapped.
pub fn get_renderable_delta_between_point_and_triangle(
    leaf: &PointPrimitive,
    parent: &TrianglePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for a_point in leaf.iter_vertices() {
        let mut closest_distance = f64::INFINITY;

        for (b_a, b_b, b_c) in parent.iter_vertices() {
            let distance = shortest_distance_from_point_to_triangle_squared(a_point, b_a, b_b, b_c);

            // because a point occupies no space, any of the nearest matches will have the same distance
            closest_distance = closest_distance.min(distance);
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

// Calculates the maximum perceptible delta between a line primitive and a point primitive.
//
// We calculate the error introduced if a leaf line primitive was replaced by a
// simplified parent point primitive, aka the geometric error.
//
// It is important to note that this is a directional algorithm meaning that the results
// will be different if the order of the primitives is swapped.
pub fn get_renderable_delta_between_line_and_point(
    leaf: &LinePrimitive,
    parent: &PointPrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_start, a_end) in leaf.iter_vertices() {
        let mut closest_distance = f64::INFINITY;

        for b_point in parent.iter_vertices() {
            let distance =
                shortest_distance_from_point_to_line_segment_squared(b_point, a_start, a_end);

            // because a point occupies no space, any of the nearest matches will have the same distance
            closest_distance = closest_distance.min(distance);
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

// Calculates the maximum perceptible delta between two sets of line primitives.
//
// We calculate the error introduced if a leaf line primitive was replaced by a
// simplified parent line primitive, aka the geometric error.
//
// It is important to note that this is a directional algorithm meaning that the results
// will be different if the order of the primitives is swapped.
pub fn get_renderable_delta_between_lines(
    leaf: &LinePrimitive,
    parent: &LinePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_start, a_end) in leaf.iter_vertices() {
        let mut closest_representations: Vec<([f32; 3], [f32; 3])> = vec![];
        let mut closest_distance = f64::INFINITY;

        // find the closest possible representations for the current line
        // todo: might be faster to save indicies but then need a lookup
        for (b_start, b_end) in parent.iter_vertices() {
            let line_dist_squared = shortest_line_distance_squared(a_start, a_end, b_start, b_end);

            if line_dist_squared < closest_distance {
                closest_representations = vec![(*b_start, *b_end)];
                closest_distance = line_dist_squared;
            } else if line_dist_squared == closest_distance {
                closest_representations.push((*b_start, *b_end));
            }
        }

        // then find the best case renderable delta between the source and it's simplification(s)
        // this will be the simplified entity that best represents the source entity and thus the
        // largest delta from this point will be the simplification error
        let mut min_renderable_delta = f64::INFINITY;
        for (b_start, b_end) in closest_representations {
            let line_distance_squared =
                longest_distance_between_lines_squared(&a_start, &a_end, &b_start, &b_end);

            min_renderable_delta = min_renderable_delta.min(line_distance_squared);
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(min_renderable_delta);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

pub fn get_renderable_delta_between_line_and_triangle(
    leaf: &LinePrimitive,
    parent: &TrianglePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_start, a_end) in leaf.iter_vertices() {
        let mut closest_representations: Vec<([f32; 3], [f32; 3], [f32; 3])> = vec![];
        let mut closest_distance = f64::INFINITY;

        // find the closest possible representations for the current line
        for (b_a, b_b, b_c) in parent.iter_vertices() {
            let distance = shortest_distance_from_line_segment_to_triangle_squared(
                a_start, a_end, b_a, b_b, b_c,
            );

            if distance < closest_distance {
                closest_representations = vec![(*b_a, *b_b, *b_c)];
                closest_distance = distance;
            } else if distance == closest_distance {
                closest_representations.push((*b_a, *b_b, *b_c));
            }
        }

        // then find the best case renderable delta between the source and it's simplification(s)
        // this will be the simplified entity that best represents the source entity and thus the
        // largest delta from this point will be the simplification error
        let mut min_renderable_delta = f64::INFINITY;
        for (b_a, b_b, b_c) in closest_representations {
            let triangle_distance_squared =
                longest_distance_between_line_segment_and_triangle_squared(
                    &a_start, &a_end, &b_a, &b_b, &b_c,
                );

            min_renderable_delta = min_renderable_delta.min(triangle_distance_squared);
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(min_renderable_delta);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

pub fn get_renderable_delta_between_triangles(
    leaf: &TrianglePrimitive,
    parent: &TrianglePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_a, a_b, a_c) in leaf.iter_vertices() {
        let mut closest_representations: Vec<([f32; 3], [f32; 3], [f32; 3])> = vec![];
        let mut closest_distance = f64::INFINITY;

        // find the closest possible representations for the current triangle
        // todo: might be faster to save indicies but then need a lookup
        for (b_a, b_b, b_c) in parent.iter_vertices() {
            let triangle_dist_squared =
                shortest_triangle_distance_squared(a_a, a_b, a_c, b_a, b_b, b_c);

            if triangle_dist_squared < closest_distance {
                closest_representations = vec![(*b_a, *b_b, *b_c)];
                closest_distance = triangle_dist_squared;
            } else if triangle_dist_squared == closest_distance {
                closest_representations.push((*b_a, *b_b, *b_c));
            }
        }

        // then find the best case renderable delta between the source and it's simplification(s)
        // this will be the simplified entity that best represents the source entity and thus the
        // largest delta from this point will be the simplification error
        let mut min_renderable_delta = f64::INFINITY;
        for (b_a, b_b, b_c) in closest_representations {
            let triangle_distance_squared =
                longest_distance_between_triangles_squared(&a_a, &a_b, &a_c, &b_a, &b_b, &b_c);

            min_renderable_delta = min_renderable_delta.min(triangle_distance_squared);
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(min_renderable_delta);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}
