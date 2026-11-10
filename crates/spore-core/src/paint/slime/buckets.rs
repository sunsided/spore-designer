//! Spatial lookup for slime balls: a grid of buckets, so a pixel only sums the balls that
//! can reach it.

use std::collections::HashMap;

use super::Ball;

/// Balls sorted into square buckets `bucket_size` reference pixels wide. Each bucket keeps
/// the input order, so every window gathers a pixel's balls in the same order and sums them
/// to bit-identical results (which keeps endless-field chunks seamless).
pub(super) struct BallBuckets {
    bucket_size: f32,
    buckets: HashMap<(i32, i32), Vec<Ball>>,
}

impl BallBuckets {
    pub fn new(balls: &[Ball], bucket_size: f32) -> Self {
        let mut buckets: HashMap<(i32, i32), Vec<Ball>> = HashMap::new();
        for ball in balls {
            let key = (
                (ball.x / bucket_size).floor() as i32,
                (ball.y / bucket_size).floor() as i32,
            );
            buckets.entry(key).or_default().push(*ball);
        }
        Self {
            bucket_size,
            buckets,
        }
    }

    /// Replace `out` with the balls of the 3x3 buckets around reference position `(x, y)`,
    /// in a fixed order (rows, then columns).
    pub fn gather(&self, x: f32, y: f32, out: &mut Vec<Ball>) {
        let (bucket_x, bucket_y) = (
            (x / self.bucket_size).floor() as i32,
            (y / self.bucket_size).floor() as i32,
        );
        out.clear();
        for row in bucket_y - 1..=bucket_y + 1 {
            for column in bucket_x - 1..=bucket_x + 1 {
                if let Some(bucket) = self.buckets.get(&(column, row)) {
                    out.extend_from_slice(bucket);
                }
            }
        }
    }
}
