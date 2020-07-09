
pub type Float = f64;

pub struct Vector2D {
    pub x: Float,
    pub y: Float,
}

pub fn dir_vector(direction_deg: Float) -> Vector2D {
    let (s, c) = direction_deg.to_radians().sin_cos();

    Vector2D {
        x: c,
        y: s,
    }
}


#[cfg(test)]
mod tests {
    use crate::geometry::{dir_vector, Float};

    const EPS: Float = 1e-5;

    #[test]
    fn test_dir_vector() {
        let v = dir_vector(45.0f64);
        assert!((v.x - v.y).abs() < EPS);
    }
}
