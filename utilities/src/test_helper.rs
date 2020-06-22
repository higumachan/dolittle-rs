use num::Float;

pub fn nearly_equal<F: Float>(f1: F, f2: F) -> bool {
    (f1 - f2).abs() < F::epsilon()
}