use std::ops::Mul;

use crate::maths::vec::{Vec3, Vec4};

// column-major 4x4 matrix, as per glTF specification
#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    pub elements: [f64; 16],
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            elements: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        }
    }

    pub fn from_array(array: &[f64; 16]) -> Self {
        Self {
            elements: array.clone(),
        }
    }

    pub fn from_column_major_nested_array(array: &[[f32; 4]; 4]) -> Self {
        Self {
            elements: [
                array[0][0] as f64,
                array[0][1] as f64,
                array[0][2] as f64,
                array[0][3] as f64,
                array[1][0] as f64,
                array[1][1] as f64,
                array[1][2] as f64,
                array[1][3] as f64,
                array[2][0] as f64,
                array[2][1] as f64,
                array[2][2] as f64,
                array[2][3] as f64,
                array[3][0] as f64,
                array[3][1] as f64,
                array[3][2] as f64,
                array[3][3] as f64,
            ],
        }
    }

    pub fn from_vectors(columns: &[Vec4; 4]) -> Self {
        Self {
            elements: [
                columns[0].x,
                columns[0].y,
                columns[0].z,
                columns[0].w,
                columns[1].x,
                columns[1].y,
                columns[1].z,
                columns[1].w,
                columns[2].x,
                columns[2].y,
                columns[2].z,
                columns[2].w,
                columns[3].x,
                columns[3].y,
                columns[3].z,
                columns[3].w,
            ],
        }
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, other: Mat4) -> Self::Output {
        return self * &other;
    }
}

impl Mul<&Mat4> for Mat4 {
    type Output = Self;

    fn mul(self, other: &Self) -> Self::Output {
        let a = &self.elements;
        let b = &other.elements;

        Self {
            elements: [
                a[0] * b[0] + a[4] * b[1] + a[8] * b[2] + a[12] * b[3], // result[0]
                a[1] * b[0] + a[5] * b[1] + a[9] * b[2] + a[13] * b[3], // result[1]
                a[2] * b[0] + a[6] * b[1] + a[10] * b[2] + a[14] * b[3], // result[2]
                a[3] * b[0] + a[7] * b[1] + a[11] * b[2] + a[15] * b[3], // result[3]
                a[0] * b[4] + a[4] * b[5] + a[8] * b[6] + a[12] * b[7], // result[4]
                a[1] * b[4] + a[5] * b[5] + a[9] * b[6] + a[13] * b[7], // result[5]
                a[2] * b[4] + a[6] * b[5] + a[10] * b[6] + a[14] * b[7], // result[6]
                a[3] * b[4] + a[7] * b[5] + a[11] * b[6] + a[15] * b[7], // result[7]
                a[0] * b[8] + a[4] * b[9] + a[8] * b[10] + a[12] * b[11], // result[8]
                a[1] * b[8] + a[5] * b[9] + a[9] * b[10] + a[13] * b[11], // result[9]
                a[2] * b[8] + a[6] * b[9] + a[10] * b[10] + a[14] * b[11], // result[10]
                a[3] * b[8] + a[7] * b[9] + a[11] * b[10] + a[15] * b[11], // result[11]
                a[0] * b[12] + a[4] * b[13] + a[8] * b[14] + a[12] * b[15], // result[12]
                a[1] * b[12] + a[5] * b[13] + a[9] * b[14] + a[13] * b[15], // result[13]
                a[2] * b[12] + a[6] * b[13] + a[10] * b[14] + a[14] * b[15], // result[14]
                a[3] * b[12] + a[7] * b[13] + a[11] * b[14] + a[15] * b[15], // result[15]
            ],
        }
    }
}

impl Mul<Vec3> for Mat4 {
    type Output = Vec3;

    // multiplies a 3D vector with an implicit 1 in the 4th dimension by a 4x4 matrix, returning a 3D vector
    fn mul(self, other: Vec3) -> Self::Output {
        let a = self.elements;
        let w = 1.0 / (a[3] * other.x + a[7] * other.y + a[11] * other.z + a[15]);

        Vec3::new(
            a[0] * other.x + a[4] * other.y + a[8] * other.z + a[12] * w,
            a[1] * other.x + a[5] * other.y + a[9] * other.z + a[13] * w,
            a[2] * other.x + a[6] * other.y + a[10] * other.z + a[14] * w,
        )
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_mat4_matrix_multiplication() {
        let a = Mat4::from_vectors(&[
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        ]);
        let b = Mat4::from_vectors(&[
            Vec4::new(16.0, 15.0, 14.0, 13.0),
            Vec4::new(12.0, 11.0, 10.0, 9.0),
            Vec4::new(8.0, 7.0, 6.0, 5.0),
            Vec4::new(4.0, 3.0, 2.0, 1.0),
        ]);
        let c = a * b;
        assert_eq!(
            c.elements,
            [
                386.0, 444.0, 502.0, 560.0, 274.0, 316.0, 358.0, 400.0, 162.0, 188.0, 214.0, 240.0,
                50.0, 60.0, 70.0, 80.0,
            ]
        );
    }

    #[test]
    fn test_mat4_vector_multiplication() {
        let a = Mat4::from_vectors(&[
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        ]);
        let b = Vec3::new(1.0, 2.0, 3.0);
        let c = a * b;

        let w = 1.0 / 72.0;
        assert_eq!(c, Vec3::new(51.0 * w, 58.0 * w, 65.0 * w));
    }
}
