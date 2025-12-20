use std::f64::consts::PI;
use crate::physics::geometry::{Geometry, Mesh, MBV, Point3, quick_hull};

pub struct Ball {
    pub radius: f64,
}


fn fibonacci_sphere_points(n: usize, r: f64) -> Vec<Point3> {
    let golden_ratio = (1.0 + 5f64.sqrt()) / 2.0;
    let mut pts = Vec::with_capacity(n);

    for i in 0..n {
        let z = 1.0 - 2.0 * ((i as f64) + 0.5) / n as f64;
        let phi = 2.0 * PI * (((i as f64) + 0.5) * golden_ratio).fract();

        let r_xy = (1.0 - z * z).sqrt();
        let x = r_xy * phi.cos();
        let y = r_xy * phi.sin();

        pts.push(Point3::new(x, y, z) * r);
    }
    pts
}

impl Ball {
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }

    fn build_mesh(vertices: Vec<Point3>) -> Mesh {
        let indices = quick_hull(&vertices);
        Mesh { vertices, indices }
    }
}

impl Geometry for Ball {
    fn get_mesh(&self) -> Mesh {
        let verts = fibonacci_sphere_points(100, self.radius);
        Self::build_mesh(verts)
    }

    fn minimal_bounding_volume(&self) -> MBV {
        let side = 2.0 * self.radius;
        MBV(side, side, side)
    }
}