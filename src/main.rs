// use geometry::*;
// use coords::*;
// use std::f64::consts::PI;
use winit::event_loop::{EventLoop, ControlFlow};

pub mod physics;
pub mod app;

use app::app::App;


// fn main() {
//     // Создаём шар радиуса 1
//     let ball = Ball::new(1.0);

//     // Оборачиваем его в GraphicsGeometry
//     let ggeom = GraphicsGeometry::new(
//         Box::new(ball),
//         (0.0, PI / 4.0, 0.0),   // вращение вокруг оси Y на 45°
//         2.0,                      // масштаб 2×
//         Point3::new(1.0, 0.0, 0.0) // смещение по X на +1
//     );

//     let mesh = ggeom.get_surface();
//     println!("Количество вершин после трансформации: {}", mesh.vertices.len());
//     println!("Первые 5 вершины:");
//     for v in &mesh.vertices[..5] {
//         println!("{:?}", v);
//     }
// }

fn init_logging() {
    // TODO : set logging level
    env_logger::init();
}

fn main() {
    init_logging();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    // we use the fastest one, but can be used this one too
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}