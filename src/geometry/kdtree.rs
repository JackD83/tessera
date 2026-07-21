use crate::maths::point::point_distance_squared;

#[derive(Debug)]
pub struct PointKdTree {
    root: Option<Box<PointKdNode>>,
}

#[derive(Debug)]
struct PointKdNode {
    point: [f32; 3],
    axis: usize,
    left: Option<Box<PointKdNode>>,
    right: Option<Box<PointKdNode>>,
}

impl PointKdTree {
    pub fn from_points(points: &[[f32; 3]]) -> Self {
        let mut points = points.to_vec();
        Self {
            root: build_node(&mut points, 0),
        }
    }

    pub fn nearest_distance_squared(&self, query: &[f32; 3]) -> Option<f64> {
        let root = self.root.as_ref()?;
        let mut best_distance = f64::INFINITY;
        nearest_node(root, query, &mut best_distance);
        Some(best_distance)
    }
}

fn build_node(points: &mut [[f32; 3]], depth: usize) -> Option<Box<PointKdNode>> {
    if points.is_empty() {
        return None;
    }

    let axis = depth % 3;
    points.sort_by(|a, b| a[axis].total_cmp(&b[axis]));
    let median = points.len() / 2;
    let (left, median_and_right) = points.split_at_mut(median);
    let (median_point, right) = median_and_right.split_first_mut().unwrap();

    Some(Box::new(PointKdNode {
        point: *median_point,
        axis,
        left: build_node(left, depth + 1),
        right: build_node(right, depth + 1),
    }))
}

fn nearest_node(node: &PointKdNode, query: &[f32; 3], best_distance: &mut f64) {
    let distance = point_distance_squared(query, &node.point);
    if distance < *best_distance {
        *best_distance = distance;
    }

    let axis = node.axis;
    let axis_delta = query[axis] as f64 - node.point[axis] as f64;
    let (near, far) = if axis_delta <= 0.0 {
        (&node.left, &node.right)
    } else {
        (&node.right, &node.left)
    };

    if let Some(near) = near {
        nearest_node(near, query, best_distance);
    }

    // This is an exact search. The far side can only be skipped when the
    // splitting-plane lower bound is strictly greater than the best exact
    // distance found so far.
    if axis_delta * axis_delta <= *best_distance {
        if let Some(far) = far {
            nearest_node(far, query, best_distance);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brute_force_nearest_distance_squared(query: &[f32; 3], points: &[[f32; 3]]) -> Option<f64> {
        points
            .iter()
            .map(|point| point_distance_squared(query, point))
            .reduce(f64::min)
    }

    fn deterministic_points(count: usize) -> Vec<[f32; 3]> {
        let mut state = 0x1234_5678_u64;
        let mut points = Vec::with_capacity(count);

        for _ in 0..count {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let x = ((state >> 32) as i32 % 2000) as f32 / 10.0;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let y = ((state >> 32) as i32 % 2000) as f32 / 10.0;
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let z = ((state >> 32) as i32 % 2000) as f32 / 10.0;
            points.push([x, y, z]);
        }

        points
    }

    #[test]
    fn nearest_distance_returns_none_for_empty_tree() {
        let tree = PointKdTree::from_points(&[]);

        assert_eq!(tree.nearest_distance_squared(&[0.0, 0.0, 0.0]), None);
    }

    #[test]
    fn nearest_distance_matches_brute_force_for_duplicates() {
        let points = vec![
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [-2.0, 0.0, 0.0],
        ];
        let tree = PointKdTree::from_points(&points);
        let query = [0.0, 0.0, 0.0];

        assert_eq!(
            tree.nearest_distance_squared(&query),
            brute_force_nearest_distance_squared(&query, &points)
        );
    }

    #[test]
    fn nearest_distance_matches_brute_force_for_deterministic_cloud() {
        let points = deterministic_points(257);
        let queries = deterministic_points(64);
        let tree = PointKdTree::from_points(&points);

        for query in queries {
            assert_eq!(
                tree.nearest_distance_squared(&query),
                brute_force_nearest_distance_squared(&query, &points)
            );
        }
    }
}
