use nalgebra::Rotation3;
use crate::physics::geometry::{Geometry, Point3, Mesh, MBV};

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
        center: Point3
    ) -> Self {
        let rotation = Rotation3::from_euler_angles(rotation_euler.0, rotation_euler.1, rotation_euler.2);
        Self { geometry, rotation, scale, center }
    }

    pub fn get_surface(&self) -> Mesh {
        let base = self.geometry.get_surface_mesh();

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

    pub fn get_edges(&self, bold: f32) -> Mesh {
        let base = self.geometry.get_edges_mesh(bold);

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