use rust_gamephysics::{app, math::types::Vector3, physics, scene::SceneBuilder};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut scene = SceneBuilder::new();
    let cube = scene.add_shape(physics::Shape::new_cuboid(Vector3::new(1.0, 1.0, 1.0)));
    let sphere = scene.add_shape(physics::Shape::new_sphere(1.0));
    let min_sphere = scene.add_shape(physics::Shape::new_sphere(0.5));

    scene.add_instance(cube, Vector3::new(0.0, 0.0, 0.0));
    scene.add_instance(sphere, Vector3::new(2.0, 0.0, 0.0));
    scene.add_instance(min_sphere, Vector3::new(0.0, 2.0, 0.0));
    scene.add_instance(min_sphere, Vector3::new(0.0, 0.0, 2.0));
    scene.set_camera(Vector3::new(5.0, 5.0, 5.0), Vector3::new(0.0, 0.0, 0.0));

    app::ApplicationBuilder::new()
        .with_scene(scene)
        .build()?
        .run();
    Ok(())
}
