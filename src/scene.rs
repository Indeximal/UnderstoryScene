use crate::assets::Assets;
use crate::foliage::ShrubEntitiesBuilder;
use crate::renderer::Renderable;
use crate::terrain::TerrainEntity;

use nalgebra_glm as glm;
use noise::NoiseFn;
use rand::{Rng, SeedableRng};
use std::rc::Rc;
use std::time::Instant;

macro_rules! time {
    ($name:expr, $block:expr) => {{
        let start = Instant::now();
        let result = $block;
        println!("Creating {} took {}ms", $name, start.elapsed().as_millis());
        result
    }};
}

pub struct Scene {
    pub entities: Vec<Box<dyn Renderable>>,
    pub start_time: Instant,
}

impl Scene {
    pub fn create(seed: u32, assets: &Assets) -> Self {
        time!(format!("SCENE {}", seed), {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed as u64);

            let height_map: Rc<dyn NoiseFn<f64, 2>> =
                time!("height map", Rc::new(crate::terrain::height_map(rng.gen())));

            let ground_entity = time!(
                "terrain",
                TerrainEntity::from_assets(height_map.as_ref(), assets)
            );

            // Accepting that the VAO is loaded anew
            let shrubs = time!("shrubs", {
                ShrubEntitiesBuilder::new()
                    .with_density(50.)
                    .on_height_map(&height_map)
                    .with_texture(assets.shrub_tex.clone())
                    .with_model(assets.shrub_model.clone())
                    .with_shader(assets.foliage_shader.clone())
                    .with_z_scale_range(0.4, 1.2)
                    .load(rng.gen())
            });

            let bushes = time!("bushes", {
                ShrubEntitiesBuilder::new()
                    .with_density(30.)
                    .on_height_map(&height_map)
                    .with_texture(assets.bush_tex.clone())
                    .with_model(assets.bush_model.clone())
                    .with_shader(assets.foliage_shader.clone())
                    .with_z_scale_range(0.9, 1.0)
                    .load(rng.gen())
            });

            let trees = time!("trees", {
                ShrubEntitiesBuilder::new()
                    .with_density(1.)
                    .with_entitiy_limit(7)
                    .on_height_map(&height_map)
                    .with_texture(assets.bark_tex.clone())
                    .with_model(assets.tree_model.clone())
                    .with_shader(assets.foliage_shader.clone())
                    .with_scale_range(0.5, 1.0)
                    .load(rng.gen())
            });

            let entities: Vec<Box<dyn Renderable>> = vec![
                Box::new(ground_entity),
                Box::new(shrubs),
                Box::new(bushes),
                Box::new(trees),
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
