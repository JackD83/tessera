use crate::{
    error::TesseraError,
    geometry::{
        Geometry,
        prepared::{
            PreparedGeometry, PreparedLinePrimitive, PreparedPointPrimitive, PreparedPrimitive,
            PreparedTileGeometry, PreparedTrianglePrimitive,
        },
    },
    maths::{
        line::{
            longest_distance_between_lines_squared,
            shortest_distance_from_point_to_line_segment_squared, shortest_line_distance_squared,
        },
        triangle::{
            longest_distance_between_line_segment_and_triangle_squared,
            longest_distance_between_triangle_and_line_segment_squared,
            longest_distance_between_triangles_squared,
            shortest_distance_from_line_segment_to_triangle_squared,
            shortest_distance_from_point_to_triangle_squared, shortest_triangle_distance_squared,
        },
    },
};
use tracing::debug;

pub fn get_geometric_error_between_geometries(
    geometries: &Vec<&Geometry>,
    parent_geometries: &Vec<&Geometry>,
) -> Result<f64, TesseraError> {
    let prepared_geometries = geometries
        .iter()
        .map(|geometry| PreparedGeometry::from_geometry(geometry))
        .collect::<Vec<_>>();
    let prepared_parent_geometries = parent_geometries
        .iter()
        .map(|geometry| PreparedGeometry::from_geometry(geometry))
        .collect::<Vec<_>>();

    get_geometric_error_between_prepared_geometries(
        &prepared_geometries,
        &prepared_parent_geometries,
    )
}

pub fn get_geometric_error_between_prepared_tile_geometries(
    geometries: &PreparedTileGeometry,
    parent_geometries: &PreparedTileGeometry,
) -> Result<f64, TesseraError> {
    get_geometric_error_between_prepared_geometries(
        &geometries.geometries,
        &parent_geometries.geometries,
    )
}

pub fn get_geometric_error_between_prepared_geometries(
    geometries: &[PreparedGeometry],
    parent_geometries: &[PreparedGeometry],
) -> Result<f64, TesseraError> {
    let mut shortest_distance = f64::INFINITY;
    let mut stats = PrimitiveComparisonStats::default();

    for geometry in geometries {
        for parent_geometry in parent_geometries {
            let distance = get_renderable_delta_between_prepared_geometries_with_stats(
                geometry,
                parent_geometry,
                &mut stats,
            )?;

            if distance < shortest_distance {
                shortest_distance = distance;
            }
        }
    }

    debug!(
        primitive_pairs_compared = stats.compared,
        primitive_pairs_pruned = stats.pruned,
        "Compared geometry primitive pairs"
    );

    Ok(shortest_distance)
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
struct PrimitiveComparisonStats {
    compared: usize,
    pruned: usize,
}

fn get_renderable_delta_between_prepared_geometries_with_stats(
    geometry: &PreparedGeometry,
    parent_geometry: &PreparedGeometry,
    stats: &mut PrimitiveComparisonStats,
) -> Result<f64, TesseraError> {
    let mut shortest_distance = f64::INFINITY;

    for primitive in &geometry.primitives {
        for parent_primitive in &parent_geometry.primitives {
            let lower_bound = primitive
                .bounding_sphere()
                .min_distance_to(parent_primitive.bounding_sphere());

            if lower_bound > shortest_distance {
                stats.pruned += 1;
                continue;
            }

            stats.compared += 1;
            let renderable_delta =
                get_renderable_delta_between_prepared_primitives(primitive, parent_primitive)?;

            // in this case, we want the smallest renderable delta across all primitives
            // compared, as many of the primitives will have larger values as they represent
            // other objects. We assume the closest primitive is the one that represents the
            // simplified version of the original object(s).
            shortest_distance = shortest_distance.min(renderable_delta);
        }
    }

    Ok(shortest_distance)
}

fn get_renderable_delta_between_prepared_primitives(
    primitive: &PreparedPrimitive,
    parent_primitive: &PreparedPrimitive,
) -> Result<f64, TesseraError> {
    match (primitive, parent_primitive) {
        (PreparedPrimitive::Points(a), PreparedPrimitive::Points(b)) => {
            get_renderable_delta_between_prepared_points(a, b)
        }
        (PreparedPrimitive::Points(a), PreparedPrimitive::Lines(b)) => {
            get_renderable_delta_between_prepared_point_and_line(a, b)
        }
        (PreparedPrimitive::Points(a), PreparedPrimitive::Triangles(b)) => {
            get_renderable_delta_between_prepared_point_and_triangle(a, b)
        }
        (PreparedPrimitive::Lines(a), PreparedPrimitive::Points(b)) => {
            get_renderable_delta_between_prepared_line_and_point(a, b)
        }
        (PreparedPrimitive::Lines(a), PreparedPrimitive::Lines(b)) => {
            get_renderable_delta_between_prepared_lines(a, b)
        }
        (PreparedPrimitive::Lines(a), PreparedPrimitive::Triangles(b)) => {
            get_renderable_delta_between_prepared_line_and_triangle(a, b)
        }
        (PreparedPrimitive::Triangles(a), PreparedPrimitive::Points(b)) => {
            get_renderable_delta_between_prepared_triangle_and_point(a, b)
        }
        (PreparedPrimitive::Triangles(a), PreparedPrimitive::Lines(b)) => {
            get_renderable_delta_between_prepared_triangle_and_line(a, b)
        }
        (PreparedPrimitive::Triangles(a), PreparedPrimitive::Triangles(b)) => {
            get_renderable_delta_between_prepared_triangles(a, b)
        }
    }
}

fn get_renderable_delta_between_prepared_points(
    leaf: &PreparedPointPrimitive,
    parent: &PreparedPointPrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for a_point in &leaf.points {
        let closest_distance = parent
            .kdtree
            .nearest_distance_squared(a_point)
            .unwrap_or(f64::INFINITY);

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    Ok(max_renderable_delta_across_primitive.sqrt())
}

fn get_renderable_delta_between_prepared_point_and_line(
    leaf: &PreparedPointPrimitive,
    parent: &PreparedLinePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for a_point in &leaf.points {
        let mut closest_distance = f64::INFINITY;
        let mut candidates = parent
            .line_aabbs
            .iter()
            .enumerate()
            .map(|(index, aabb)| (index, aabb.min_distance_to_point_squared(a_point)))
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));

        for (index, lower_bound) in candidates {
            if lower_bound > closest_distance {
                break;
            }

            let (b_start, b_end) = &parent.lines[index];
            closest_distance = closest_distance.min(
                shortest_distance_from_point_to_line_segment_squared(a_point, b_start, b_end),
            );
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    Ok(max_renderable_delta_across_primitive.sqrt())
}

fn get_renderable_delta_between_prepared_point_and_triangle(
    leaf: &PreparedPointPrimitive,
    parent: &PreparedTrianglePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for a_point in &leaf.points {
        let mut closest_distance = f64::INFINITY;
        let mut candidates = parent
            .triangle_aabbs
            .iter()
            .enumerate()
            .map(|(index, aabb)| (index, aabb.min_distance_to_point_squared(a_point)))
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));

        for (index, lower_bound) in candidates {
            if lower_bound > closest_distance {
                break;
            }

            let (b_a, b_b, b_c) = &parent.triangles[index];
            closest_distance = closest_distance.min(
                shortest_distance_from_point_to_triangle_squared(a_point, b_a, b_b, b_c),
            );
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    Ok(max_renderable_delta_across_primitive.sqrt())
}

fn get_renderable_delta_between_prepared_line_and_point(
    leaf: &PreparedLinePrimitive,
    parent: &PreparedPointPrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_start, a_end) in &leaf.lines {
        let mut closest_distance = f64::INFINITY;

        for b_point in &parent.points {
            closest_distance = closest_distance.min(
                shortest_distance_from_point_to_line_segment_squared(b_point, a_start, a_end),
            );
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    Ok(max_renderable_delta_across_primitive.sqrt())
}

fn get_renderable_delta_between_prepared_lines(
    leaf: &PreparedLinePrimitive,
    parent: &PreparedLinePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (leaf_index, (a_start, a_end)) in leaf.lines.iter().enumerate() {
        let mut closest_distance = f64::INFINITY;
        let mut min_renderable_delta_for_closest = f64::INFINITY;
        let mut candidates = parent
            .line_aabbs
            .iter()
            .enumerate()
            .map(|(index, aabb)| {
                (
                    index,
                    leaf.line_aabbs[leaf_index].min_distance_to_aabb_squared(aabb),
                )
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));

        for (index, lower_bound) in candidates {
            if lower_bound > closest_distance {
                break;
            }

            let (b_start, b_end) = &parent.lines[index];
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

    Ok(max_renderable_delta_across_primitive.sqrt())
}

fn get_renderable_delta_between_prepared_line_and_triangle(
    leaf: &PreparedLinePrimitive,
    parent: &PreparedTrianglePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (leaf_index, (a_start, a_end)) in leaf.lines.iter().enumerate() {
        let mut closest_distance = f64::INFINITY;
        let mut min_renderable_delta_for_closest = f64::INFINITY;
        let mut candidates = parent
            .triangle_aabbs
            .iter()
            .enumerate()
            .map(|(index, aabb)| {
                (
                    index,
                    leaf.line_aabbs[leaf_index].min_distance_to_aabb_squared(aabb),
                )
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));

        for (index, lower_bound) in candidates {
            if lower_bound > closest_distance {
                break;
            }

            let (b_a, b_b, b_c) = &parent.triangles[index];
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

    Ok(max_renderable_delta_across_primitive.sqrt())
}

fn get_renderable_delta_between_prepared_triangle_and_point(
    leaf: &PreparedTrianglePrimitive,
    parent: &PreparedPointPrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (a_a, a_b, a_c) in &leaf.triangles {
        let mut closest_distance = f64::INFINITY;

        for b_point in &parent.points {
            closest_distance = closest_distance.min(
                shortest_distance_from_point_to_triangle_squared(b_point, a_a, a_b, a_c),
            );
        }

        max_renderable_delta_across_primitive =
            max_renderable_delta_across_primitive.max(closest_distance);
    }

    Ok(max_renderable_delta_across_primitive.sqrt())
}

fn get_renderable_delta_between_prepared_triangle_and_line(
    leaf: &PreparedTrianglePrimitive,
    parent: &PreparedLinePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (leaf_index, (a_a, a_b, a_c)) in leaf.triangles.iter().enumerate() {
        let mut closest_distance = f64::INFINITY;
        let mut min_renderable_delta_for_closest = f64::INFINITY;
        let mut candidates = parent
            .line_aabbs
            .iter()
            .enumerate()
            .map(|(index, aabb)| {
                (
                    index,
                    leaf.triangle_aabbs[leaf_index].min_distance_to_aabb_squared(aabb),
                )
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));

        for (index, lower_bound) in candidates {
            if lower_bound > closest_distance {
                break;
            }

            let (b_start, b_end) = &parent.lines[index];
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

    Ok(max_renderable_delta_across_primitive.sqrt())
}

fn get_renderable_delta_between_prepared_triangles(
    leaf: &PreparedTrianglePrimitive,
    parent: &PreparedTrianglePrimitive,
) -> Result<f64, TesseraError> {
    let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

    for (leaf_index, (a_a, a_b, a_c)) in leaf.triangles.iter().enumerate() {
        let mut closest_distance = f64::INFINITY;
        let mut min_renderable_delta_for_closest = f64::INFINITY;
        let mut candidates = parent
            .triangle_aabbs
            .iter()
            .enumerate()
            .map(|(index, aabb)| {
                (
                    index,
                    leaf.triangle_aabbs[leaf_index].min_distance_to_aabb_squared(aabb),
                )
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));

        for (index, lower_bound) in candidates {
            if lower_bound > closest_distance {
                break;
            }

            let (b_a, b_b, b_c) = &parent.triangles[index];
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

    Ok(max_renderable_delta_across_primitive.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{LinePrimitive, PointPrimitive, Primitive, TrianglePrimitive, Vertices};

    fn point_geometry(name: &str, primitive_points: Vec<Vec<[f32; 3]>>) -> Geometry {
        let mut geometry = Geometry::new(name.to_string());

        for points in primitive_points {
            let mut primitive = PointPrimitive::new();
            primitive.set_vertices(points);
            geometry.add_primitive(Primitive::PointPrimitive(primitive));
        }

        geometry
    }

    fn prepared_points(points: Vec<[f32; 3]>) -> PreparedPointPrimitive {
        let mut primitive = PointPrimitive::new();
        primitive.set_vertices(points);
        PreparedPointPrimitive::from_primitive(&primitive)
    }

    fn prepared_lines(lines: Vec<([f32; 3], [f32; 3])>) -> PreparedLinePrimitive {
        let mut primitive = LinePrimitive::new();
        primitive.set_vertices(lines.iter().flat_map(|(a, b)| [*a, *b]).collect());
        PreparedLinePrimitive::from_primitive(&primitive)
    }

    fn prepared_triangles(
        triangles: Vec<([f32; 3], [f32; 3], [f32; 3])>,
    ) -> PreparedTrianglePrimitive {
        let mut primitive = TrianglePrimitive::new();
        primitive.set_vertices(
            triangles
                .iter()
                .flat_map(|(a, b, c)| [*a, *b, *c])
                .collect(),
        );
        PreparedTrianglePrimitive::from_primitive(&primitive)
    }

    fn brute_force_point_line_delta(
        leaf: &PreparedPointPrimitive,
        parent: &PreparedLinePrimitive,
    ) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for a_point in &leaf.points {
            let mut closest_distance = f64::INFINITY;

            for (b_start, b_end) in &parent.lines {
                closest_distance = closest_distance.min(
                    shortest_distance_from_point_to_line_segment_squared(a_point, b_start, b_end),
                );
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(closest_distance);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    fn brute_force_point_triangle_delta(
        leaf: &PreparedPointPrimitive,
        parent: &PreparedTrianglePrimitive,
    ) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for a_point in &leaf.points {
            let mut closest_distance = f64::INFINITY;

            for (b_a, b_b, b_c) in &parent.triangles {
                closest_distance = closest_distance.min(
                    shortest_distance_from_point_to_triangle_squared(a_point, b_a, b_b, b_c),
                );
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(closest_distance);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    fn assert_same_float_result(a: f64, b: f64) {
        if a.is_nan() || b.is_nan() {
            assert!(a.is_nan() && b.is_nan(), "{a} != {b}");
        } else {
            assert_eq!(a, b);
        }
    }

    fn brute_force_line_line_delta(
        leaf: &PreparedLinePrimitive,
        parent: &PreparedLinePrimitive,
    ) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for (a_start, a_end) in &leaf.lines {
            let mut closest_distance = f64::INFINITY;
            let mut min_renderable_delta_for_closest = f64::INFINITY;

            for (b_start, b_end) in &parent.lines {
                let distance = shortest_line_distance_squared(a_start, a_end, b_start, b_end);

                if distance < closest_distance {
                    closest_distance = distance;
                    min_renderable_delta_for_closest =
                        longest_distance_between_lines_squared(a_start, a_end, b_start, b_end);
                } else if distance == closest_distance {
                    let renderable_delta =
                        longest_distance_between_lines_squared(a_start, a_end, b_start, b_end);
                    min_renderable_delta_for_closest =
                        min_renderable_delta_for_closest.min(renderable_delta);
                }
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(min_renderable_delta_for_closest);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    fn brute_force_line_triangle_delta(
        leaf: &PreparedLinePrimitive,
        parent: &PreparedTrianglePrimitive,
    ) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for (a_start, a_end) in &leaf.lines {
            let mut closest_distance = f64::INFINITY;
            let mut min_renderable_delta_for_closest = f64::INFINITY;

            for (b_a, b_b, b_c) in &parent.triangles {
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
                    let renderable_delta =
                        longest_distance_between_line_segment_and_triangle_squared(
                            a_start, a_end, b_a, b_b, b_c,
                        );
                    min_renderable_delta_for_closest =
                        min_renderable_delta_for_closest.min(renderable_delta);
                }
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(min_renderable_delta_for_closest);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    fn brute_force_triangle_line_delta(
        leaf: &PreparedTrianglePrimitive,
        parent: &PreparedLinePrimitive,
    ) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for (a_a, a_b, a_c) in &leaf.triangles {
            let mut closest_distance = f64::INFINITY;
            let mut min_renderable_delta_for_closest = f64::INFINITY;

            for (b_start, b_end) in &parent.lines {
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
                    let renderable_delta =
                        longest_distance_between_triangle_and_line_segment_squared(
                            a_a, a_b, a_c, b_start, b_end,
                        );
                    min_renderable_delta_for_closest =
                        min_renderable_delta_for_closest.min(renderable_delta);
                }
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(min_renderable_delta_for_closest);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    fn brute_force_triangle_triangle_delta(
        leaf: &PreparedTrianglePrimitive,
        parent: &PreparedTrianglePrimitive,
    ) -> f64 {
        let mut max_renderable_delta_across_primitive = f64::NEG_INFINITY;

        for (a_a, a_b, a_c) in &leaf.triangles {
            let mut closest_distance = f64::INFINITY;
            let mut min_renderable_delta_for_closest = f64::INFINITY;

            for (b_a, b_b, b_c) in &parent.triangles {
                let distance = shortest_triangle_distance_squared(a_a, a_b, a_c, b_a, b_b, b_c);

                if distance < closest_distance {
                    closest_distance = distance;
                    min_renderable_delta_for_closest =
                        longest_distance_between_triangles_squared(a_a, a_b, a_c, b_a, b_b, b_c);
                } else if distance == closest_distance {
                    let renderable_delta =
                        longest_distance_between_triangles_squared(a_a, a_b, a_c, b_a, b_b, b_c);
                    min_renderable_delta_for_closest =
                        min_renderable_delta_for_closest.min(renderable_delta);
                }
            }

            max_renderable_delta_across_primitive =
                max_renderable_delta_across_primitive.max(min_renderable_delta_for_closest);
        }

        max_renderable_delta_across_primitive.sqrt()
    }

    #[test]
    fn prepared_comparison_matches_iterator_comparison() {
        let leaf = point_geometry("leaf", vec![vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]]]);
        let parent = point_geometry("parent", vec![vec![[1.0, 0.0, 0.0], [4.0, 0.0, 0.0]]]);
        let leaf_refs = vec![&leaf];
        let parent_refs = vec![&parent];
        let prepared_leaf = PreparedGeometry::from_geometry(&leaf);
        let prepared_parent = PreparedGeometry::from_geometry(&parent);

        let iterator_error =
            get_geometric_error_between_geometries(&leaf_refs, &parent_refs).unwrap();
        let prepared_error =
            get_geometric_error_between_prepared_geometries(&[prepared_leaf], &[prepared_parent])
                .unwrap();

        assert_eq!(prepared_error, iterator_error);
    }

    #[test]
    fn point_to_line_aabb_ordered_traversal_matches_brute_force() {
        let leaf = prepared_points(vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]]);
        let parent = prepared_lines(vec![
            ([100.0, 0.0, 0.0], [101.0, 0.0, 0.0]),
            ([1.0, -1.0, 0.0], [1.0, 1.0, 0.0]),
            ([-1.0, -1.0, 0.0], [-1.0, 1.0, 0.0]),
        ]);

        assert_eq!(
            get_renderable_delta_between_prepared_point_and_line(&leaf, &parent).unwrap(),
            brute_force_point_line_delta(&leaf, &parent)
        );
    }

    #[test]
    fn point_to_triangle_aabb_ordered_traversal_matches_brute_force() {
        let leaf = prepared_points(vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]]);
        let parent = prepared_triangles(vec![
            ([100.0, 0.0, 0.0], [101.0, 0.0, 0.0], [100.0, 1.0, 0.0]),
            ([1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [1.0, 0.0, 1.0]),
            ([-1.0, -1.0, -1.0], [-1.0, 1.0, -1.0], [-1.0, 0.0, 1.0]),
        ]);

        assert_eq!(
            get_renderable_delta_between_prepared_point_and_triangle(&leaf, &parent).unwrap(),
            brute_force_point_triangle_delta(&leaf, &parent)
        );
    }

    #[test]
    fn line_to_line_aabb_ordered_traversal_matches_brute_force() {
        let leaf = prepared_lines(vec![
            ([0.0, 0.0, 0.0], [0.0, 2.0, 0.0]),
            ([3.0, 0.0, 0.0], [3.0, 2.0, 0.0]),
        ]);
        let parent = prepared_lines(vec![
            ([100.0, 0.0, 0.0], [101.0, 0.0, 0.0]),
            ([1.0, 0.0, 0.0], [1.0, 2.0, 0.0]),
            ([-1.0, 0.0, 0.0], [-1.0, 2.0, 0.0]),
        ]);

        assert_eq!(
            get_renderable_delta_between_prepared_lines(&leaf, &parent).unwrap(),
            brute_force_line_line_delta(&leaf, &parent)
        );
    }

    #[test]
    fn line_to_triangle_aabb_ordered_traversal_matches_brute_force() {
        let leaf = prepared_lines(vec![
            ([0.0, 0.0, 0.0], [0.0, 2.0, 0.0]),
            ([3.0, 0.0, 0.0], [3.0, 2.0, 0.0]),
        ]);
        let parent = prepared_triangles(vec![
            ([100.0, 0.0, 0.0], [101.0, 0.0, 0.0], [100.0, 1.0, 0.0]),
            ([1.0, -1.0, -1.0], [1.0, 3.0, -1.0], [1.0, 0.0, 1.0]),
            ([-1.0, -1.0, -1.0], [-1.0, 3.0, -1.0], [-1.0, 0.0, 1.0]),
        ]);

        assert_eq!(
            get_renderable_delta_between_prepared_line_and_triangle(&leaf, &parent).unwrap(),
            brute_force_line_triangle_delta(&leaf, &parent)
        );
    }

    #[test]
    fn triangle_to_line_aabb_ordered_traversal_matches_brute_force() {
        let leaf = prepared_triangles(vec![
            ([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 2.0]),
            ([3.0, 0.0, 0.0], [3.0, 2.0, 0.0], [3.0, 0.0, 2.0]),
        ]);
        let parent = prepared_lines(vec![
            ([100.0, 0.0, 0.0], [101.0, 0.0, 0.0]),
            ([1.0, 0.0, 0.0], [1.0, 2.0, 0.0]),
            ([-1.0, 0.0, 0.0], [-1.0, 2.0, 0.0]),
        ]);

        assert_eq!(
            get_renderable_delta_between_prepared_triangle_and_line(&leaf, &parent).unwrap(),
            brute_force_triangle_line_delta(&leaf, &parent)
        );
    }

    #[test]
    fn triangle_to_triangle_aabb_ordered_traversal_matches_brute_force() {
        let leaf = prepared_triangles(vec![
            ([0.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 2.0]),
            ([3.0, 0.0, 0.0], [3.0, 2.0, 0.0], [3.0, 0.0, 2.0]),
        ]);
        let parent = prepared_triangles(vec![
            ([100.0, 0.0, 0.0], [101.0, 0.0, 0.0], [100.0, 1.0, 0.0]),
            ([1.0, 0.0, 0.0], [1.0, 2.0, 0.0], [1.0, 0.0, 2.0]),
            ([-1.0, 0.0, 0.0], [-1.0, 2.0, 0.0], [-1.0, 0.0, 2.0]),
        ]);

        assert_eq!(
            get_renderable_delta_between_prepared_triangles(&leaf, &parent).unwrap(),
            brute_force_triangle_triangle_delta(&leaf, &parent)
        );
    }

    #[test]
    fn degenerate_lines_aabb_ordered_traversal_matches_brute_force() {
        let leaf = prepared_lines(vec![
            ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            ([2.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
        ]);
        let parent = prepared_lines(vec![
            ([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            ([100.0, 0.0, 0.0], [100.0, 0.0, 0.0]),
        ]);

        assert_same_float_result(
            get_renderable_delta_between_prepared_lines(&leaf, &parent).unwrap(),
            brute_force_line_line_delta(&leaf, &parent),
        );
    }

    #[test]
    fn degenerate_triangles_aabb_ordered_traversal_matches_brute_force() {
        let leaf = prepared_triangles(vec![
            ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            ([2.0, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
        ]);
        let parent = prepared_triangles(vec![
            ([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
            ([100.0, 0.0, 0.0], [100.0, 0.0, 0.0], [100.0, 0.0, 0.0]),
        ]);

        assert_same_float_result(
            get_renderable_delta_between_prepared_triangles(&leaf, &parent).unwrap(),
            brute_force_triangle_triangle_delta(&leaf, &parent),
        );
    }

    #[test]
    fn bounding_sphere_pruning_skips_pairs_that_cannot_improve_current_best() {
        let leaf =
            PreparedGeometry::from_geometry(&point_geometry("leaf", vec![vec![[0.0, 0.0, 0.0]]]));
        let parent = PreparedGeometry::from_geometry(&point_geometry(
            "parent",
            vec![vec![[0.0, 0.0, 0.0]], vec![[100.0, 0.0, 0.0]]],
        ));
        let mut stats = PrimitiveComparisonStats::default();

        let error =
            get_renderable_delta_between_prepared_geometries_with_stats(&leaf, &parent, &mut stats)
                .unwrap();

        assert_eq!(error, 0.0);
        assert_eq!(stats.compared, 1);
        assert_eq!(stats.pruned, 1);
    }

    #[test]
    fn bounding_sphere_pruning_keeps_pairs_that_can_tie_current_best() {
        let leaf =
            PreparedGeometry::from_geometry(&point_geometry("leaf", vec![vec![[0.0, 0.0, 0.0]]]));
        let parent = PreparedGeometry::from_geometry(&point_geometry(
            "parent",
            vec![vec![[1.0, 0.0, 0.0]], vec![[-1.0, 0.0, 0.0]]],
        ));
        let mut stats = PrimitiveComparisonStats::default();

        let error =
            get_renderable_delta_between_prepared_geometries_with_stats(&leaf, &parent, &mut stats)
                .unwrap();

        assert_eq!(error, 1.0);
        assert_eq!(stats.compared, 2);
        assert_eq!(stats.pruned, 0);
    }

    #[test]
    fn bounding_sphere_pruning_does_not_skip_before_current_best_is_known() {
        let leaf =
            PreparedGeometry::from_geometry(&point_geometry("leaf", vec![vec![[0.0, 0.0, 0.0]]]));
        let parent = PreparedGeometry::from_geometry(&point_geometry(
            "parent",
            vec![vec![[100.0, 0.0, 0.0]]],
        ));
        let mut stats = PrimitiveComparisonStats::default();

        let error =
            get_renderable_delta_between_prepared_geometries_with_stats(&leaf, &parent, &mut stats)
                .unwrap();

        assert_eq!(error, 100.0);
        assert_eq!(stats.compared, 1);
        assert_eq!(stats.pruned, 0);
    }
}
