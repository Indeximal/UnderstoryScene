use crate::renderer::Renderable;
use crate::terrain::{BasePlate, TerrainEntity};

use nalgebra_glm as glm;
use std::time::Instant;

pub struct Scene {
    pub entities: Vec<Box<dyn Renderable>>,
}

impl Scene {
    pub fn load() -> Self {
        println!("Loading scene...");
        let start = Instant::now();

        let timer = Instant::now();
        let terrain_entity = TerrainEntity::from_scratch();
        println!("Loading the terrain took {}ms", timer.elapsed().as_millis());

        let timer = Instant::now();
        let base_plate = BasePlate::from_scratch();
        println!(
            "Loading the base plate took {}ms",
            timer.elapsed().as_millis()
        );

        let entities: Vec<Box<dyn Renderable>> =
            vec![Box::new(terrain_entity), Box::new(base_plate)];

        println!("Total loading time: {}ms", start.elapsed().as_millis());

        Scene { entities }
    }

    pub fn background_color(&self) -> (f32, f32, f32, f32) {
        (186. / 255., 219. / 255., 222. / 255., 1.0) // A sky blue
    }

    pub fn eye_position(&self) -> glm::Vec3 {
        glm::vec3(0.0, -4.0, 1.7) // Stand behind the scene with eye height 170cm
    }

    pub fn look_at(&self) -> glm::Vec3 {
        glm::vec3(0.0, -1.0, 0.2) // Look at the floor near the center
    }
}
