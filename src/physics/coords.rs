use std::ops::{Add, Sub, Mul};

#[derive(Debug, Clone)]
pub struct Coord {
    // cartesian
    pub x: f64,
    pub y: f64,
    pub z: f64,

    // spherical
    pub r: f64,
    pub azimuth: f64,
    pub elevation: f64,
}

impl Coord {
    pub fn new_cartesian(x: f64, y: f64, z: f64) -> Self {
        let r = (x * x + y * y + z * z).sqrt();
        let azimuth = if x == 0.0 && y == 0.0 { 0.0 } else { y.atan2(x) };
        let elevation = if r == 0.0 { 0.0 } else { (z / r).asin() };

        Self { x, y, z, r, azimuth, elevation }
    }

    pub fn new_spherical(r: f64, azimuth: f64, elevation: f64) -> Self {
        let cos_elev = elevation.cos();
        let sin_elev = elevation.sin();
        let cos_azim = azimuth.cos();
        let sin_azim = azimuth.sin();

        let x = r * cos_elev * cos_azim;
        let y = r * cos_elev * sin_azim;
        let z = r * sin_elev;

        Self { x, y, z, r, azimuth, elevation }
    }

    pub fn set_cartesian(&mut self, x: f64, y: f64, z: f64) {
        self.x = x;
        self.y = y;
        self.z = z;

        self.r = (x * x + y * y + z * z).sqrt();
        self.azimuth = if x == 0.0 && y == 0.0 { 0.0 } else { y.atan2(x) };
        self.elevation = if self.r == 0.0 { 0.0 } else { (z / self.r).asin() };
    }

    pub fn set_spherical(&mut self, r: f64, azimuth: f64, elevation: f64) {
        self.r = r;
        self.azimuth = azimuth;
        self.elevation = elevation;

        let cos_elev = elevation.cos();
        let sin_elev = elevation.sin();
        let cos_azim = azimuth.cos();
        let sin_azim = azimuth.sin();

        self.x = r * cos_elev * cos_azim;
        self.y = r * cos_elev * sin_azim;
        self.z = r * sin_elev;
    }

    /* ---------- Утилиты ---------------- */

    pub fn cartesian(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }

    pub fn spherical(&self) -> (f64, f64, f64) {
        (self.r, self.azimuth, self.elevation)
    }
}

/* ---------- Операции над векторами ---------------- */

impl Add for Coord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let x = self.x + rhs.x;
        let y = self.y + rhs.y;
        let z = self.z + rhs.z;
        Self::new_cartesian(x, y, z)
    }
}

impl Sub for Coord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let x = self.x - rhs.x;
        let y = self.y - rhs.y;
        let z = self.z - rhs.z;
        Self::new_cartesian(x, y, z)
    }
}

impl Mul<f64> for Coord {
    type Output = Coord;

    fn mul(self, rhs: f64) -> Self::Output {
        let x = self.x * rhs;
        let y = self.y * rhs;
        let z = self.z * rhs;
        Coord::new_cartesian(x, y, z)
    }
}

use std::fmt;

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cartesian: ({:.3}, {:.3}, {:.3})\nSpherical: (r={:.3}, azimuth={:.3} rad, elevation={:.3} rad)",
            self.x, self.y, self.z, self.r, self.azimuth, self.elevation
        )
    }
}
