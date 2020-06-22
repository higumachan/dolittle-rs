use num::Float;

pub fn nearly_equal<F: Float>(f1: F, f2: F) -> bool {
    nearly_equal_with_eps(f1, f2, F::epsilon())
}

pub fn nearly_equal_with_eps<F: Float>(f1: F, f2: F, epsilon: F) -> bool {
    (f1 - f2).abs() < epsilon
}