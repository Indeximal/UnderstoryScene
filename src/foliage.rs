use crate::assets::ImageNoiseFnWrapper;
use crate::mesh::{ElementMeshVAO, InstancedMeshesVAO, Mesh};
use crate::renderer::Renderable;
use crate::scene::SCENE_SIZE;
use crate::shader::Shader;
use crate::texture::Texture;

use nalgebra_glm as glm;
use noise::{MultiFractal, NoiseFn};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_distr::Uniform;
use std::rc::Rc;

#[derive(Clone)]
pub struct ShrubEntities {
    pub albedo: Rc<Texture>,
    pub vao: Rc<InstancedMeshesVAO>,
    pub shader: Rc<Shader>,
}

pub struct ShrubEntitiesBuilder {
    density: f64,
    num_limit: usize,
    height_map: Option<Rc<dyn NoiseFn<f64, 2>>>,
    model: Option<Rc<Mesh>>,
    z_scale_range: (f32, f32),
    scale_range: (f32, f32),
    texture: Option<Rc<Texture>>,
    shader: Option<Rc<Shader>>,
    bushiness: Option<ImageNoiseFnWrapper<1>>,
}

impl ShrubEntitiesBuilder {
    pub fn new() -> Self {
        ShrubEntitiesBuilder {
            density: 0.0,
            num_limit: usize::MAX,
            height_map: None,
            model: None,
            z_scale_range: (1.0, 1.0),
            scale_range: (1.0, 1.0),
            texture: None,
            shader: None,
            bushiness: None,
        }
    }

    pub fn load(self, seed: u32) -> ShrubEntities {
        let mut rng = StdRng::seed_from_u64(seed as u64);

        let height_map = self.height_map.expect("Height map is required");
        let model = self.model.expect("Model source file path is required");
        let texture = self.texture.expect("Texture is required");
        let shader = self.shader.expect("Shader is required");

        let mesh_vao = ElementMeshVAO::new_from_mesh(&model);

        let distr = probability_distribution(self.density, rng.gen());
        let mut positions = if let Some(bushiness) = self.bushiness {
            let bushiness = noise::Power::new(bushiness, noise::Constant::new(2.0));
            let bushiness = noise::ScaleBias::new(bushiness)
                .set_scale(2.0)
                .set_bias(0.1);

            let distr = noise::Multiply::new(distr, bushiness);
            generate_points_on_distribution(distr, (0., SCENE_SIZE, 0., SCENE_SIZE), rng.gen())
        } else {
            generate_points_on_distribution(distr, (0., SCENE_SIZE, 0., SCENE_SIZE), rng.gen())
        };

        if positions.len() > self.num_limit {
            positions.shuffle(&mut rng);
            positions.resize_with(self.num_limit, || {
                unreachable!("Len is less than the limit")
            });
        }
        let positions = positions;

        println!("Spawned {} entities", positions.len());

        // For some very weird ass reason do the translate & scale functions right multiply,
        // thus for scale than translate, I need to translate then scale...

        let model_mats: Vec<glm::Mat4> = positions
            .into_iter()
            .map(|p| {
                // TODO: rotation based on height gradient
                let rotation_angle: f32 = rng.sample(Uniform::new(0.0, 6.28));
                // TODO: scale in a more natural distribution
                let z_scale: f32 = rng.sample(Uniform::new_inclusive(
                    self.z_scale_range.0,
                    self.z_scale_range.1,
                ));
                let scale: f32 = rng.sample(Uniform::new_inclusive(
                    self.scale_range.0,
                    self.scale_range.1,
                ));

                glm::scale(
                    &glm::rotate_z(
                        &glm::translate(
                            &glm::identity(),
                            &glm::vec3(p.x, p.y, height_map.get([p.x as f64, p.y as f64]) as f32),
                        ),
                        rotation_angle,
                    ),
                    &glm::vec3(scale, scale, scale * z_scale),
                )
            })
            .collect();

        let instanced_vao = InstancedMeshesVAO::from_existing_with_models(mesh_vao, &model_mats);

        ShrubEntities {
            albedo: texture,
            vao: Rc::new(instanced_vao),
            shader,
        }
    }

    pub fn with_density(mut self, density: f64) -> Self {
        self.density = density;
        self
    }

    pub fn with_entitiy_limit(mut self, num_limit: usize) -> Self {
        self.num_limit = num_limit;
        self
    }

    pub fn on_height_map(mut self, height_map: &Rc<dyn NoiseFn<f64, 2>>) -> Self {
        self.height_map = Some(height_map.clone());
        self
    }

    pub fn with_bushiness(mut self, bushiness: ImageNoiseFnWrapper<1>) -> Self {
        self.bushiness = Some(bushiness);
        self
    }

    pub fn with_model(mut self, model: Rc<Mesh>) -> Self {
        self.model = Some(model);
        self
    }

    /// Streches the models in z direction (height)
    pub fn with_z_scale_range(mut self, min_z_scale: f32, max_z_scale: f32) -> Self {
        self.z_scale_range = (min_z_scale, max_z_scale);
        self
    }

    /// Streches the models in all directions
    pub fn with_scale_range(mut self, min_scale: f32, max_scale: f32) -> Self {
        self.scale_range = (min_scale, max_scale);
        self
    }

    pub fn with_texture(mut self, texture: Rc<Texture>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_shader(mut self, shader: Rc<Shader>) -> Self {
        self.shader = Some(shader);
        self
    }
}

impl Renderable for ShrubEntities {
    fn render(&self, view_proj_mat: &glm::Mat4) {
        // SAFETY: fine, if the matrix/vector types match.
        unsafe {
            self.shader.activate();
            gl::UniformMatrix4fv(
                self.shader.get_uniform_location("view_proj"),
                1,
                gl::FALSE,
                view_proj_mat.as_ptr(),
            );

            self.albedo.activate(0);
            gl::Uniform1i(self.shader.get_uniform_location("albedo"), 0);
        }

        self.vao.render();
    }
}

/// Note that the `density` might not actually be the average, since
/// this is too difficult to enforce. Just some scale approximately in the same
/// order as the average.
///
/// FIXME: more consitent shrub number. Large scale randomness has too big influence.
fn probability_distribution(density: f64, seed: u32) -> impl NoiseFn<f64, 2> {
    let noise = noise::Fbm::<noise::Perlin>::new(seed)
        .set_octaves(4) // Not very much detail required
        .set_frequency(0.2); // Large scale features approx 5 meters large

    // Transform from [-1, 1] to [0, density]
    let noise = noise::ScaleBias::new(noise)
        .set_bias(1.0)
        .set_scale(density / 2.0);

    // // Make it less uniform.
    // let noise = noise::Power::new(noise, noise::Constant::new(5.0));
    noise
}

/// Generates random points in a rectangle according to the given density distribution.
///
/// It does this sampling the distribution at discrete locations and then drawing
/// from a poission variable N with expected value equal to the density times the
/// area of the chunk. In this tiny chunk the N points are uniformly distributed.
///
/// Given the same seed and distribution, this function is deterministic.
///
/// The distribution is assumed to be normalized, ie the value of an integral over a
/// unit area should be the number of points in this area.
/// The unit is therefore [number of points / area].
fn generate_points_on_distribution(
    distribution: impl NoiseFn<f64, 2>,
    (x_min, x_max, y_min, y_max): (f32, f32, f32, f32),
    seed: u64,
) -> Vec<glm::Vec2> {
    let mut points = Vec::new();
    let resolution = 100;

    let dx = (x_max - x_min) / resolution as f32;
    let dy = (y_max - y_min) / resolution as f32;
    let area = dx * dy;

    let mut rng = StdRng::seed_from_u64(seed);

    // Y is going front to back. Potentially reducing double drawing.
    for x in 0..resolution {
        for y in 0..resolution {
            let fx = x_min + dx * x as f32;
            let fy = y_min + dy * y as f32;

            let density =
                distribution.get([(fx + dx / 2.).into(), (fy + dy / 2.).into()]) as f32 * area;
            if density <= 0.0 {
                continue;
            }

            let num_points_in_chunk = (density + rng.gen::<f32>()).floor() as usize;

            for _ in 0..num_points_in_chunk as usize {
                let point = glm::vec2(fx + dx * rng.gen::<f32>(), fy + dy * rng.gen::<f32>());
                points.push(point);
            }
        }
    }

    points
}
