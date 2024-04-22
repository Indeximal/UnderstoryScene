use crate::assets::{Assets, ImageNoiseFnWrapper};
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

/// The side length of the scene square in meters.
///
/// One corner of the scene is at (0, 0), the opposite at (+SCENE_SIZE, +SCENE_SIZE).
pub const SCENE_SIZE: f32 = 15.0;

pub struct Scene {
    pub entities: Vec<Box<dyn Renderable>>,
    pub start_time: Instant,
}

impl Scene {
    pub fn create(seed: u32, assets: &Assets) -> Self {
        time!(format!("SCENE {}", seed), {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed as u64);

            let height_map: Rc<dyn NoiseFn<f64, 2>> = time!(
                "height map",
                Rc::new(crate::terrain::height_map(
                    assets.base_map.clone(),
                    rng.gen()
                ))
            );

            let variant_map: Rc<dyn NoiseFn<f64, 2>> = time!(
                "variant map",
                Rc::new(crate::terrain::variant_map(
                    assets.base_map.clone(),
                    rng.gen()
                ))
            );

            let ground_entity = time!(
                "terrain",
                TerrainEntity::ground(height_map.as_ref(), variant_map.as_ref(), assets)
            );

            let blueberry_bushes = time!(
                "blueberry terrain",
                TerrainEntity::bushes(
                    &noise::Add::new(height_map.as_ref(), crate::terrain::bush_heights(rng.gen())),
                    variant_map.as_ref(),
                    assets
                )
            );

            // Accepting that the VAO is loaded anew
            let saplings = time!("saplings", {
                ShrubEntitiesBuilder::new()
                    .with_density(50.)
                    .on_height_map(&height_map)
                    .with_bushiness(ImageNoiseFnWrapper::new_green(assets.base_map.clone()))
                    .with_texture(assets.sapling_tex.clone())
                    .with_model(assets.sapling_model.clone())
                    .with_shader(assets.foliage_shader.clone())
                    .with_z_scale_range(0.4, 1.2)
                    .load(rng.gen())
            });

            let bushes = time!("bushes", {
                ShrubEntitiesBuilder::new()
                    .with_density(30.)
                    .on_height_map(&height_map)
                    .with_texture(assets.bush_tex.clone())
                    .with_model(assets.bush1_model.clone())
                    .with_shader(assets.foliage_shader.clone())
                    .with_z_scale_range(0.9, 1.0)
                    .load(rng.gen())
            });

            let shrubs = time!("shrubs", {
                ShrubEntitiesBuilder::new()
                    .with_density(5.)
                    .on_height_map(&height_map)
                    .with_texture(assets.shrub_side_tex.clone())
                    .with_model(assets.shrub_model.clone())
                    .with_shader(assets.foliage_shader.clone())
                    .with_bushiness(ImageNoiseFnWrapper::new_green(assets.base_map.clone()))
                    .with_z_scale_range(0.7, 1.0)
                    .with_scale_range(1.5, 3.0)
                    .load(rng.gen())
            });

            let trees = time!("trees", {
                ShrubEntitiesBuilder::new()
                    .with_density(1.)
                    .with_entitiy_limit(60)
                    .on_height_map(&height_map)
                    .with_bounds(0., 1.5 * SCENE_SIZE, 0., 1.5 * SCENE_SIZE)
                    .with_texture(assets.bark_tex.clone())
                    .with_model(assets.tree_model.clone())
                    .with_shader(assets.foliage_shader.clone())
                    .with_bushiness(ImageNoiseFnWrapper::new_blue(assets.base_map.clone()))
                    .with_scale_range(0.5, 1.0)
                    .load(rng.gen())
            });

            let entities: Vec<Box<dyn Renderable>> = vec![
                Box::new(trees),
                Box::new(saplings),
                Box::new(bushes),
                Box::new(shrubs),
                Box::new(blueberry_bushes),
                Box::new(ground_entity),
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
        let bob = glm::vec3(
            0.1 * (t * 0.32 + 1.).sin(),
            0.03 * (t * 0.33 + 2.).sin(),
            0.05 * (t * 0.34 + 3.).sin(),
        );
        // Stand in a corner of the scene the scene somewhat above the ground
        let base = glm::vec3(1.0, 1.0, 2.0);
        base
    }

    pub fn look_at(&self) -> glm::Vec3 {
        // Look at the floor in the direction of the opposite corner
        glm::vec3(3.5, 3.5, 0.3)
    }
}
