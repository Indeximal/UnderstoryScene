use crate::foliage::ShrubEntity;
use crate::renderer::Renderable;
use crate::terrain::{BasePlate, TerrainEntity};
use crate::texture::Texture;

use nalgebra_glm as glm;
use std::rc::Rc;
use std::time::Instant;

pub struct Scene {
    pub entities: Vec<Box<dyn Renderable>>,
    pub start_time: Instant,
}

macro_rules! time {
    ($name:expr, $block:expr) => {{
        let start = Instant::now();
        let result = $block;
        println!("Loading {} took {}ms", $name, start.elapsed().as_millis());
        result
    }};
}

impl Scene {
    pub fn load() -> Self {
        time!("scene", {
            let ground_entity = time!("terrain", TerrainEntity::from_scratch());

            let leaves_entity = time!("leaves texture", {
                let tex = Texture::from_file("textures/leaves_masked1.png")
                    .expect("Loading leaves texture failed");
                tex.enable_mipmap();
                // TODO: additional noise, not just offset.
                TerrainEntity {
                    albedo: Rc::new(tex),
                    model: glm::translate(&ground_entity.model, &glm::vec3(0.0, 0.0, 0.05)),
                    ..ground_entity.clone()
                }
            });

            let base_plate = time!("base plate", BasePlate::from_scratch());

            let shrub = time!("shrub", ShrubEntity::from_scratch());

            let entities: Vec<Box<dyn Renderable>> = vec![
                Box::new(ground_entity),
                Box::new(leaves_entity),
                Box::new(base_plate),
                Box::new(shrub),
            ];

            Scene {
                entities,
                start_time: Instant::now(),
            }
        })
    }

    pub fn background_color(&self) -> (f32, f32, f32, f32) {
        (186. / 255., 219. / 255., 222. / 255., 1.0) // A sky blue
    }

    pub fn eye_position(&self) -> glm::Vec3 {
        let t = self.start_time.elapsed().as_secs_f32();
        // Different phase and frequency for random looking movement
        glm::vec3(
            0.1 * (t * 0.32 + 1.).sin(),
            0.03 * (t * 0.33 + 2.).sin(),
            0.05 * (t * 0.34 + 3.).sin(),
        ) + glm::vec3(0.0, -4.0, 1.7) // Stand behind the scene with eye height 170cm
    }

    pub fn look_at(&self) -> glm::Vec3 {
        glm::vec3(0.0, -1.0, 0.2) // Look at the floor near the center
    }
}
