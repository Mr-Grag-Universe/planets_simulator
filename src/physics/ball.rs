use std::{f64::consts::PI, ops::Index};
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

    fn build_surface_mesh(&self) -> Mesh {
        let vertices = fibonacci_sphere_points(100, self.radius);
        let indices = quick_hull(&vertices);
        Mesh { vertices, indices }
    }
}

fn shift_point_from(x: &Point3, base: &Point3, shift_range: f32) -> Point3 {
    let dist = x - base;
    let e = dist / dist.magnitude();
    x + shift_range as f64 * e
}

impl Geometry for Ball {
    fn get_surface_mesh(&self) -> Mesh {
        self.build_surface_mesh()
    }
    
    fn get_edges_mesh(&self, bold: f32) -> Mesh {
        let vertices = fibonacci_sphere_points(100, self.radius);
        let surface_indices = quick_hull(&vertices);
        let surface_mesh = Mesh { vertices: vertices.clone(), indices: surface_indices };
        let indices = surface_mesh.get_edges_indices();

        let mut edges_vertices = Vec::new();
        let mut edges_indices = Vec::new();
        let center_point = surface_mesh.get_center();
        for (ind1, ind2) in &indices {
            let (v1, v2) = (&vertices[*ind1], &vertices[*ind2]);
            let v1_edge = shift_point_from(&v1, &center_point, bold);
            let v2_edge = shift_point_from(&v2, &center_point, bold);

            let ind11_edge = edges_vertices.iter().position(|v| v == v1).unwrap_or_else(|| {
                edges_vertices.push(v1.clone());
                edges_vertices.len() - 1
            });

            let ind12_edge = edges_vertices.iter().position(|v| *v == v1_edge).unwrap_or_else(|| {
                edges_vertices.push(v1_edge.clone());
                edges_vertices.len() - 1
            });

            // Теперь получаем индексы для v2 и v2_edge
            let ind21_edge = edges_vertices.iter().position(|v| v == v2).unwrap_or_else(|| {
                edges_vertices.push(v2.clone());
                edges_vertices.len() - 1
            });

            let ind22_edge = edges_vertices.iter().position(|v| *v == v2_edge).unwrap_or_else(|| {
                edges_vertices.push(v2_edge.clone());
                edges_vertices.len() - 1
            });

            edges_indices.push([ind11_edge, ind12_edge, ind21_edge]);
            edges_indices.push([ind12_edge, ind22_edge, ind21_edge]);
        }

        Mesh { vertices: edges_vertices, indices: edges_indices }
    }

    fn minimal_bounding_volume(&self) -> MBV {
        let side = 2.0 * self.radius;
        MBV(side, side, side)
    }
}