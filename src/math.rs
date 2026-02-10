use std::ops::AddAssign;

use rug::{Complex, Float};

const PRECISION: u32 = 128;
pub const MAX_ITER: u32 = 20000;

pub struct HighPrecisionState {
    pub reference: Complex, // The anchor
    pub camera: Complex,    // Where user is looking
    pub zoom: Float,
}

impl HighPrecisionState {
    pub fn new() -> Self {
        Self {
            reference: Complex::new(PRECISION),
            camera: Complex::new(PRECISION),
            zoom: Float::with_val(PRECISION, 1.0),
        }
    }

    /// Calculates how many iterations it takes to survive or escape
    pub fn get_escape_time(&self, point: &Complex, max_checks: u32) -> u32 {
        let mut z = Complex::with_val(PRECISION, (0.0, 0.0));
        let c = point; // c is the candidate point

        for i in 0..max_checks {
            z.square_mut();
            z += c;

            // Optimization: Check norm only periodically or use a rough check first
            if z.real().to_f32().abs() > 2.0 || z.imag().to_f32().abs() > 2.0 {
                let norm = Float::with_val(24, z.norm_ref());
                if norm.to_f32() > 4.0 {
                    return i;
                }
            }
        }
        max_checks // Survived!
    }

    /// Searches for a better reference point near the target center.
    /// Returns the best point found (longest survival time).
    pub fn find_best_reference(
        &self,
        center: &Complex,
        zoom: &Float,
        max_iter: u32,
    ) -> (Complex, u32) {
        // 1. Check the center first
        let center_score = self.get_escape_time(center, max_iter);
        if center_score == max_iter {
            return (center.clone(), center_score);
        }

        // If center fails, sample a pattern around it.
        // We look for a point that is "deeper" in the set (lasts longer).
        let mut best_point = center.clone();
        let mut best_score = center_score;

        // Create a search radius (e.g., 0.5 * screen width approx)
        // Since we don't have aspect ratio here, just assume square or use 1.0/zoom
        let one = Float::with_val(PRECISION, 1.0);
        let radius = one / zoom;

        // Simple 4-corner + center search pattern (you can increase this for better stability)
        let offsets = [
            (0.0, 0.0), // Check center (Camera) first
            (0.1, 0.1),
            (0.1, -0.1),
            (-0.1, 0.1),
            (-0.1, -0.1), // Corners
            (0.2, 0.0),
            (-0.2, 0.0),
            (0.0, 0.2),
            (0.0, -0.2), // Cardinals
        ];

        for (ox, oy) in offsets {
            let mut candidate = center.clone();
            let dx = Float::with_val(PRECISION, ox) * &radius;
            let dy = Float::with_val(PRECISION, oy) * &radius;

            candidate.mut_real().add_assign(&dx);
            candidate.mut_imag().add_assign(&dy);

            let score = self.get_escape_time(&candidate, max_iter);

            if score > best_score {
                best_score = score;
                best_point = candidate;
                // If we found a survivor, stop immediately!
                if best_score == max_iter {
                    break;
                }
            }
        }

        (best_point, best_score)
    }

    /// Calculates the "Reference Orbit" for the center point.
    /// Returns a list of points (Z values) that the GPU will use.
    pub fn calculate_orbit(&self, max_iter: u32) -> (Vec<[f32; 2]>, u32) {
        let mut orbit = Vec::with_capacity(max_iter as usize);
        let mut z = Complex::with_val(PRECISION, (0.0, 0.0));
        let c = &self.reference;

        let mut valid_count = 0; // Track valid iterations

        for _ in 0..max_iter {
            let re = z.real().to_f32();
            let im = z.imag().to_f32();
            orbit.push([re, im]);
            valid_count += 1; // We added a valid point

            z.square_mut();
            z += c;

            let norm = Float::with_val(24, z.norm_ref());
            if norm.to_f32() > 4.0 {
                break;
            }
        }

        // Fill the rest with zeros (Padding)
        while orbit.len() < max_iter as usize {
            orbit.push([0.0, 0.0]);
        }

        (orbit, valid_count)
    }
}
