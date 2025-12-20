use std::f64::consts::PI;
use crate::physics::geometry::{Geometry, Mesh, MBV, Point3};

pub struct Cube {
    side_len: f64,
}

fn vertex(pos: [i8; 3]) -> Point3 {
    Point3::new(pos[0] as f64, pos[1] as f64, pos[2] as f64)
}

impl Cube {
    pub fn new(side_len: f64) -> Self {
        Self { side_len }
    }
    
    fn build_mesh(&self) -> Mesh {
        let scale = self.side_len / 2.0;
        let vertices = [
            // bottom
            scale * vertex([-1, -1, -1]),
            scale * vertex([1, -1, -1]),
            scale * vertex([-1, 1, -1]),
            scale * vertex([1, 1, -1]),
            // top
            scale * vertex([-1, -1, 1]),
            scale * vertex([1, -1, 1]),
            scale * vertex([-1, 1, 1]),
            scale * vertex([1, 1, 1]),
        ];
        
        let indices: &[[usize; 3]] = &[
            [0, 1, 3], [0, 2, 3], // bottom (xy-plane)
            [4, 5, 7], [4, 6, 7], // top (xy-plane + 1)
            [0, 1, 5], [0, 4, 5], // (xz-plane)
            [2, 3, 7], [2, 6, 7], // (xz-plane + 1)
            [0, 2, 6], [0, 4, 6], // (yz-plane)
            [1, 3, 7], [1, 5, 7], // (yz-plane + 1)
        ];
        
        Mesh { vertices: vertices.to_vec(), indices: indices.to_vec() }
    }
}

impl Geometry for Cube {
    fn get_mesh(&self) -> Mesh {
        self.build_mesh()
    }

    fn minimal_bounding_volume(&self) -> MBV {
        let side = self.side_len;
        MBV(side, side, side)
    }
} 