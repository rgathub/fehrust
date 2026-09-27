use super::*;

#[test]
fn multiply_identity() {
    let id = windows_numerics::Matrix3x2::identity();
    let result = multiply_matrix3x2(id, id);
    assert_eq!(result.M11, 1.0);
    assert_eq!(result.M12, 0.0);
    assert_eq!(result.M21, 0.0);
    assert_eq!(result.M22, 1.0);
    assert_eq!(result.M31, 0.0);
    assert_eq!(result.M32, 0.0);
}

#[test]
fn multiply_with_identity() {
    let a = windows_numerics::Matrix3x2 {
        M11: 2.0,
        M12: 3.0,
        M21: 4.0,
        M22: 5.0,
        M31: 6.0,
        M32: 7.0,
    };
    let id = windows_numerics::Matrix3x2::identity();
    let result = multiply_matrix3x2(a, id);
    assert_eq!(result.M11, 2.0);
    assert_eq!(result.M12, 3.0);
    assert_eq!(result.M21, 4.0);
    assert_eq!(result.M22, 5.0);
    assert_eq!(result.M31, 6.0);
    assert_eq!(result.M32, 7.0);
}

#[test]
fn multiply_known_values() {
    let a = windows_numerics::Matrix3x2 {
        M11: 1.0,
        M12: 2.0,
        M21: 3.0,
        M22: 4.0,
        M31: 5.0,
        M32: 6.0,
    };
    let b = windows_numerics::Matrix3x2 {
        M11: 7.0,
        M12: 8.0,
        M21: 9.0,
        M22: 10.0,
        M31: 11.0,
        M32: 12.0,
    };
    let result = multiply_matrix3x2(a, b);
    assert_eq!(result.M11, 25.0);
    assert_eq!(result.M12, 28.0);
    assert_eq!(result.M21, 57.0);
    assert_eq!(result.M22, 64.0);
    assert_eq!(result.M31, 100.0);
    assert_eq!(result.M32, 112.0);
}
