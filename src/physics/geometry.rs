use std::f64::consts::PI;

use nalgebra::{Vector3, Rotation3};
use convexhull3d::{ConvexHull3D, Vertex};
use std::collections::HashSet;

pub fn quick_hull(vertices: &Vec<Point3>) -> Vec<[usize; 3]> {
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

impl Mesh {
    pub fn get_edges_indices(&self) -> Vec<(usize, usize)> {
        let mut edges = HashSet::new();

        for inds in &self.indices {
            let (a, b, c) = (inds[0], inds[1], inds[2]);
            edges.insert((a.min(b), a.max(b)));
            edges.insert((a.min(c), a.max(c)));
            edges.insert((c.min(b), c.max(b)));
        }

        edges.into_iter().collect()
    }

    pub fn get_center(&self) -> Point3 {
        let count = self.vertices.len();
        assert_ne!(count, 0, "count of points must be > 0");
        let sum = self.vertices.iter().fold(Point3::new(0.0, 0.0, 0.0), |acc, &point| {
            Point3::new(
                acc.x + point.x,
                acc.y + point.y,
                acc.z + point.z,
            )
        });
        Point3::new(
            sum.x / count as f64,
            sum.y / count as f64,
            sum.z / count as f64,
        )
    }
}

pub trait Geometry {
    fn get_surface_mesh(&self) -> Mesh;
    fn get_edges_mesh(&self, bold: f32) -> Mesh;
    fn minimal_bounding_volume(&self) -> MBV;
}

pub fn generate_transform(aspect_ratio: f32) -> glam::Mat4 {
    let projection = glam::Mat4::perspective_rh(PI as f32 / 4.0, aspect_ratio, 1.0, 100.0);
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(1.5f32, -5.0, 3.0),
        glam::Vec3::ZERO,
        glam::Vec3::Z,
    );
    projection * view
}