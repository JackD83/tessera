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
            longest_distance_between_triangle_and_line_segment_squared,
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
        let mut closest_distance = f64::INFINITY;
        let mut min_renderable_delta_for_closest = f64::INFINITY;

        // Find the closest possible representations for the current line. Track the
        // best renderable delta for all exact ties without allocating a temporary vec.
        for (b_start, b_end) in parent.iter_vertices() {
            let line_dist_squared = shortest_line_distance_squared(a_start, a_end, b_start, b_end);

            if line_dist_squared < closest_distance {
                closest_distance = line_dist_squared;
                min_renderable_delta_for_closest =
                    longest_distance_between_lines_squared(a_start, a_end, b_start, b_end);
            } else if line_dist_squared == closest_distance {
                let line_distance_squared =
                    longest_distance_between_lines_squared(a_start, a_end, b_start, b_end);
                min_renderable_delta_for_closest =
                    min_renderable_delta_for_closest.min(line_distance_squared);
            }
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(min_renderable_delta_for_closest);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

pub fn get_renderable_delta_between_line_and_triangle(
    leaf: &LinePrimitive,
    parent: &TrianglePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_start, a_end) in leaf.iter_vertices() {
        let mut closest_distance = f64::INFINITY;
        let mut min_renderable_delta_for_closest = f64::INFINITY;

        // Find the closest possible representations for the current line. Track the
        // best renderable delta for all exact ties without allocating a temporary vec.
        for (b_a, b_b, b_c) in parent.iter_vertices() {
            let distance = shortest_distance_from_line_segment_to_triangle_squared(
                a_start, a_end, b_a, b_b, b_c,
            );

            if distance < closest_distance {
                closest_distance = distance;
                min_renderable_delta_for_closest =
                    longest_distance_between_line_segment_and_triangle_squared(
                        a_start, a_end, b_a, b_b, b_c,
                    );
            } else if distance == closest_distance {
                let triangle_distance_squared =
                    longest_distance_between_line_segment_and_triangle_squared(
                        a_start, a_end, b_a, b_b, b_c,
                    );
                min_renderable_delta_for_closest =
                    min_renderable_delta_for_closest.min(triangle_distance_squared);
            }
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(min_renderable_delta_for_closest);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

pub fn get_renderable_delta_between_triangle_and_point(
    leaf: &TrianglePrimitive,
    parent: &PointPrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_a, a_b, a_c) in leaf.iter_vertices() {
        let mut closest_distance = f64::INFINITY;

        for b_point in parent.iter_vertices() {
            let distance = shortest_distance_from_point_to_triangle_squared(b_point, a_a, a_b, a_c);

            // because a point occupies no space, any of the nearest matches will have the same distance
            closest_distance = closest_distance.min(distance);
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

pub fn get_renderable_delta_between_triangle_and_line(
    leaf: &TrianglePrimitive,
    parent: &LinePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_a, a_b, a_c) in leaf.iter_vertices() {
        let mut closest_distance = f64::INFINITY;
        let mut min_renderable_delta_for_closest = f64::INFINITY;

        for (b_start, b_end) in parent.iter_vertices() {
            let distance = shortest_distance_from_line_segment_to_triangle_squared(
                b_start, b_end, a_a, a_b, a_c,
            );

            if distance < closest_distance {
                closest_distance = distance;
                min_renderable_delta_for_closest =
                    longest_distance_between_triangle_and_line_segment_squared(
                        a_a, a_b, a_c, b_start, b_end,
                    );
            } else if distance == closest_distance {
                let triangle_distance_squared =
                    longest_distance_between_triangle_and_line_segment_squared(
                        a_a, a_b, a_c, b_start, b_end,
                    );
                min_renderable_delta_for_closest =
                    min_renderable_delta_for_closest.min(triangle_distance_squared);
            }
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(min_renderable_delta_for_closest);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

pub fn get_renderable_delta_between_triangles(
    leaf: &TrianglePrimitive,
    parent: &TrianglePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_a, a_b, a_c) in leaf.iter_vertices() {
        let mut closest_distance = f64::INFINITY;
        let mut min_renderable_delta_for_closest = f64::INFINITY;

        // Find the closest possible representations for the current triangle. Track the
        // best renderable delta for all exact ties without allocating a temporary vec.
        for (b_a, b_b, b_c) in parent.iter_vertices() {
            let triangle_dist_squared =
                shortest_triangle_distance_squared(a_a, a_b, a_c, b_a, b_b, b_c);

            if triangle_dist_squared < closest_distance {
                closest_distance = triangle_dist_squared;
                min_renderable_delta_for_closest =
                    longest_distance_between_triangles_squared(a_a, a_b, a_c, b_a, b_b, b_c);
            } else if triangle_dist_squared == closest_distance {
                let triangle_distance_squared =
                    longest_distance_between_triangles_squared(a_a, a_b, a_c, b_a, b_b, b_c);
                min_renderable_delta_for_closest =
                    min_renderable_delta_for_closest.min(triangle_distance_squared);
            }
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(min_renderable_delta_for_closest);
    }

    return Ok(max_renderable_delta_across_primitive.sqrt());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Vertices;

    fn assert_close(a: f64, b: f64) {
        assert!((a - b).abs() <= 1e-12, "{a} != {b}");
    }

    fn line_primitive(vertices: Vec<[f32; 3]>) -> LinePrimitive {
        let mut primitive = LinePrimitive::new();
        primitive.set_vertices(vertices);
        primitive
    }

    fn triangle_primitive(vertices: Vec<[f32; 3]>) -> TrianglePrimitive {
        let mut primitive = TrianglePrimitive::new();
        primitive.set_vertices(vertices);
        primitive
    }

    fn legacy_renderable_delta_between_lines(leaf: &LinePrimitive, parent: &LinePrimitive) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for (a_start, a_end) in leaf.iter_vertices() {
            let mut closest_representations: Vec<([f32; 3], [f32; 3])> = vec![];
            let mut closest_distance = f64::INFINITY;

            for (b_start, b_end) in parent.iter_vertices() {
                let distance = shortest_line_distance_squared(a_start, a_end, b_start, b_end);

                if distance < closest_distance {
                    closest_representations = vec![(*b_start, *b_end)];
                    closest_distance = distance;
                } else if distance == closest_distance {
                    closest_representations.push((*b_start, *b_end));
                }
            }

            let mut min_renderable_delta = f64::INFINITY;
            for (b_start, b_end) in closest_representations {
                let distance =
                    longest_distance_between_lines_squared(a_start, a_end, &b_start, &b_end);
                min_renderable_delta = min_renderable_delta.min(distance);
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(min_renderable_delta);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    fn legacy_renderable_delta_between_line_and_triangle(
        leaf: &LinePrimitive,
        parent: &TrianglePrimitive,
    ) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for (a_start, a_end) in leaf.iter_vertices() {
            let mut closest_representations: Vec<([f32; 3], [f32; 3], [f32; 3])> = vec![];
            let mut closest_distance = f64::INFINITY;

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

            let mut min_renderable_delta = f64::INFINITY;
            for (b_a, b_b, b_c) in closest_representations {
                let distance = longest_distance_between_line_segment_and_triangle_squared(
                    a_start, a_end, &b_a, &b_b, &b_c,
                );
                min_renderable_delta = min_renderable_delta.min(distance);
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(min_renderable_delta);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    fn legacy_renderable_delta_between_triangle_and_line(
        leaf: &TrianglePrimitive,
        parent: &LinePrimitive,
    ) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for (a_a, a_b, a_c) in leaf.iter_vertices() {
            let mut closest_representations: Vec<([f32; 3], [f32; 3])> = vec![];
            let mut closest_distance = f64::INFINITY;

            for (b_start, b_end) in parent.iter_vertices() {
                let distance = shortest_distance_from_line_segment_to_triangle_squared(
                    b_start, b_end, a_a, a_b, a_c,
                );

                if distance < closest_distance {
                    closest_representations = vec![(*b_start, *b_end)];
                    closest_distance = distance;
                } else if distance == closest_distance {
                    closest_representations.push((*b_start, *b_end));
                }
            }

            let mut min_renderable_delta = f64::INFINITY;
            for (b_start, b_end) in closest_representations {
                let distance = longest_distance_between_triangle_and_line_segment_squared(
                    a_a, a_b, a_c, &b_start, &b_end,
                );
                min_renderable_delta = min_renderable_delta.min(distance);
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(min_renderable_delta);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    fn legacy_renderable_delta_between_triangles(
        leaf: &TrianglePrimitive,
        parent: &TrianglePrimitive,
    ) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for (a_a, a_b, a_c) in leaf.iter_vertices() {
            let mut closest_representations: Vec<([f32; 3], [f32; 3], [f32; 3])> = vec![];
            let mut closest_distance = f64::INFINITY;

            for (b_a, b_b, b_c) in parent.iter_vertices() {
                let distance = shortest_triangle_distance_squared(a_a, a_b, a_c, b_a, b_b, b_c);

                if distance < closest_distance {
                    closest_representations = vec![(*b_a, *b_b, *b_c)];
                    closest_distance = distance;
                } else if distance == closest_distance {
                    closest_representations.push((*b_a, *b_b, *b_c));
                }
            }

            let mut min_renderable_delta = f64::INFINITY;
            for (b_a, b_b, b_c) in closest_representations {
                let distance =
                    longest_distance_between_triangles_squared(a_a, a_b, a_c, &b_a, &b_b, &b_c);
                min_renderable_delta = min_renderable_delta.min(distance);
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(min_renderable_delta);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    #[test]
    fn refactored_line_line_delta_matches_legacy_tie_handling() {
        let leaf = line_primitive(vec![[0.0, 0.0, 0.0], [0.0, 2.0, 0.0]]);
        let parent = line_primitive(vec![
            [1.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 2.0, 0.0],
        ]);

        assert_close(
            get_renderable_delta_between_lines(&leaf, &parent).unwrap(),
            legacy_renderable_delta_between_lines(&leaf, &parent),
        );
    }

    #[test]
    fn refactored_line_triangle_delta_matches_legacy_tie_handling() {
        let leaf = line_primitive(vec![[0.0, 0.0, 0.0], [0.0, 2.0, 0.0]]);
        let parent = triangle_primitive(vec![
            [1.0, 0.0, -1.0],
            [1.0, 2.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, -1.0],
            [-1.0, 2.0, -1.0],
            [-1.0, 0.0, 1.0],
        ]);

        assert_close(
            get_renderable_delta_between_line_and_triangle(&leaf, &parent).unwrap(),
            legacy_renderable_delta_between_line_and_triangle(&leaf, &parent),
        );
    }

    #[test]
    fn refactored_triangle_line_delta_matches_legacy_tie_handling() {
        let leaf = triangle_primitive(vec![[0.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 2.0]]);
        let parent = line_primitive(vec![
            [1.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 2.0, 0.0],
        ]);

        assert_close(
            get_renderable_delta_between_triangle_and_line(&leaf, &parent).unwrap(),
            legacy_renderable_delta_between_triangle_and_line(&leaf, &parent),
        );
    }

    #[test]
    fn refactored_triangle_triangle_delta_matches_legacy_tie_handling() {
        let leaf = triangle_primitive(vec![[0.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 2.0]]);
        let parent = triangle_primitive(vec![
            [1.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            [1.0, 0.0, 2.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 2.0, 0.0],
            [-1.0, 0.0, 2.0],
        ]);

        assert_close(
            get_renderable_delta_between_triangles(&leaf, &parent).unwrap(),
            legacy_renderable_delta_between_triangles(&leaf, &parent),
        );
    }
}
