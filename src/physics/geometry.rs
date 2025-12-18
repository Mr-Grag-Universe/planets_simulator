use std::f64::consts::PI;

use nalgebra::{Vector3, Rotation3};
use convexhull3d::{ConvexHull3D, Vertex};

fn quick_hull(vertices: &Vec<Point3>) -> Vec<[usize; 3]> {
    let vertices: Vec<Vertex> = vertices.iter()
        .map(|v| Vertex::new(v.x, v.y, v.z))
        .collect();

    match ConvexHull3D::build(&vertices) {
        Ok(hull) => {
            let mut indices = Vec::new();
            
            for face in hull.faces() {
                indices.push([
                    face.v0,
                    face.v1,
                    face.v2,
                ]);
            }
            indices
        }
        Err(_) => vec![],
    }
}

pub type Point3 = Vector3<f64>;

#[derive(Debug, Clone, Copy)]
pub struct MBV(pub f64, pub f64, pub f64);

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Point3>,
    pub indices: Vec<[usize; 3]>,
}

pub trait Geometry {
    fn get_surface(&self) -> Mesh;
    fn minimal_bounding_volume(&self) -> MBV;
}

pub struct Ball {
    pub radius: f64,
}

impl Ball {
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }

    fn fibonacci_sphere_points(n: usize, r: f64) -> Vec<Point3> {
        let golden_ratio = (1.0 + 5f64.sqrt()) / 2.0;
        let mut pts = Vec::with_capacity(n);

        for i in 0..n {
            let z = 1.0 - 2.0 * ((i as f64) + 0.5) / n as f64;
            let theta = z.acos();
            let phi = 2.0 * PI * (((i as f64) + 0.5) * golden_ratio).fract();

            let r_xy = (1.0 - z * z).sqrt();
            let x = r_xy * phi.cos();
            let y = r_xy * phi.sin();

            pts.push(Point3::new(x, y, z) * r);
        }
        pts
    }

    fn build_mesh(vertices: Vec<Point3>) -> Mesh {
        let indices = quick_hull(&vertices);
        Mesh { vertices, indices }
    }
}

impl Geometry for Ball {
    fn get_surface(&self) -> Mesh {
        let verts = Self::fibonacci_sphere_points(100, self.radius);
        Self::build_mesh(verts)
    }

    fn minimal_bounding_volume(&self) -> MBV {
        let side = 2.0 * self.radius;
        MBV(side, side, side)
    }
}


pub struct GraphicsGeometry {
    pub geometry: Box<dyn Geometry>,
    pub rotation: Rotation3<f64>,
    pub scale: f64,
    pub center: Point3,
}

impl GraphicsGeometry {
    pub fn new(
        geometry: Box<dyn Geometry>,
        rotation_euler: (f64, f64, f64), // (roll, pitch, yaw) in radians
        scale: f64,
        center: Point3,
    ) -> Self {
        let rotation = Rotation3::from_euler_angles(rotation_euler.0, rotation_euler.1, rotation_euler.2);
        Self { geometry, rotation, scale, center }
    }

    pub fn get_surface(&self) -> Mesh {
        let base = self.geometry.get_surface();

        let transformed_vertices: Vec<Point3> = base.vertices.iter().map(|v| {
            let scaled = *v * self.scale;
            let rotated = self.rotation * scaled;
            rotated + self.center
        }).collect();

        Mesh {
            vertices: transformed_vertices,
            indices: base.indices.clone(),
        }
    }

    pub fn minimal_bounding_volume(&self) -> MBV {
        self.geometry.minimal_bounding_volume()
    }
}
