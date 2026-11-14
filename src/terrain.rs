//! What the landscape layers share: fixed noise, so the land is always the
//! same land, and the farther ridge's skyline, which the railway's road
//! climbs to.

/// A fixed random number from 0 to 1 for any pair of numbers.
pub fn hash(i: f64, seed: f64) -> f64 {
    let x = (i * 127.1 + seed * 311.7).sin() * 43758.5453;
    x - x.floor()
}

/// Smooth one-dimensional value noise, from 0 to 1.
pub fn noise(x: f64, seed: f64) -> f64 {
    let i = x.floor();
    let f = x - i;
    let u = f * f * (3.0 - 2.0 * f);
    hash(i, seed) * (1.0 - u) + hash(i + 1.0, seed) * u
}

/// Ridged noise has sharp peaks and broad valleys, like a mountain range.
pub fn ridged(x: f64, seed: f64) -> f64 {
    let v = 1.0 - (2.0 * noise(x, seed) - 1.0).abs();
    v * v
}

/// The farther ridge's height above the horizon, in pixels, at fraction `u`
/// of the way across a window `height` pixels high. A shoulder of hill rises
/// behind the level crossing, so the road beyond the rails has a slope to
/// climb into, and a saddle dips in the ridge to the left of it: the pass
/// the road goes over.
pub fn mid_ridge(u: f64, height: f64) -> f64 {
    let shoulder = (u - 0.6) / 0.12;
    let pass = (u - 0.25) / 0.04;
    let west = (u - 0.14) / 0.07;
    height * (0.035 + 0.15 * noise(u * 2.6 + 5.0, 4.0))
        + height * 0.025 * noise(u * 8.2, 5.0)
        + height * 0.07 * (-shoulder * shoulder).exp()
        - height * 0.04 * (-pass * pass).exp()
        + height * 0.03 * (-west * west).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_land_is_always_the_same_land() {
        assert_eq!(noise(3.7, 4.0), noise(3.7, 4.0));
        for i in 0..200 {
            let x = f64::from(i) * 0.37;
            assert!((0.0..=1.0).contains(&hash(x, 9.0)));
            assert!((0.0..=1.0).contains(&noise(x, 9.0)));
            assert!((0.0..=1.0).contains(&ridged(x, 9.0)));
        }
        // Noise passes through the fixed values at whole numbers.
        assert_eq!(noise(5.0, 2.0), hash(5.0, 2.0));
    }

    #[test]
    fn the_ridge_dips_at_the_pass() {
        let height = 1000.0;
        assert!(mid_ridge(0.25, height) < mid_ridge(0.2, height));
        assert!(mid_ridge(0.25, height) < mid_ridge(0.3, height));
        // It stays a ridge all the way across.
        for i in 0..=100 {
            assert!(mid_ridge(f64::from(i) / 100.0, height) > 0.0);
        }
    }
}
