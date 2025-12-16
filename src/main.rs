use geometry::*;
use coords::*;
use std::f64::consts::PI;

pub mod coords;
pub mod geometry;


fn main() {
    // Создаём шар радиуса 1
    let ball = Ball::new(1.0);

    // Оборачиваем его в GraphicsGeometry
    let ggeom = GraphicsGeometry::new(
        Box::new(ball),
        (0.0, PI / 4.0, 0.0),   // вращение вокруг оси Y на 45°
        2.0,                      // масштаб 2×
        Point3::new(1.0, 0.0, 0.0) // смещение по X на +1
    );

    let mesh = ggeom.get_surface();
    println!("Количество вершин после трансформации: {}", mesh.vertices.len());
    println!("Первые 5 вершины:");
    for v in &mesh.vertices[..5] {
        println!("{:?}", v);
    }
}