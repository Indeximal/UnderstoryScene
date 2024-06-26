use nalgebra_glm as glm;
use rand::{Rng, SeedableRng};
use std::rc::Rc;

use crate::assets::{Assets, ImageNoiseFnWrapper};
use crate::mesh::ElementMeshVAO;
use crate::renderer::Renderable;
use crate::scene::SCENE_SIZE;
use crate::shader::Shader;
use crate::texture::Texture;

use noise::{MultiFractal, NoiseFn, ScaleBias};

#[derive(Clone)]
pub struct TerrainEntity {
    pub vao: Rc<ElementMeshVAO>,
    pub displacement: Rc<Texture>,
    pub variant: Rc<Texture>,
    pub albedo_xy1: Rc<Texture>,
    pub albedo_xy2: Rc<Texture>,
    pub albedo_xz: Rc<Texture>,
    pub albedo_yz: Rc<Texture>,
    pub shader: Rc<Shader>,
    pub model: glm::Mat4,
    /// A matrix that will right multiply a world coordinate into a uv coordinate.
    pub world_to_uv: glm::Mat3,
}

impl TerrainEntity {
    pub fn ground(
        height_fn: &(impl NoiseFn<f64, 2> + ?Sized),
        variant_fn: &(impl NoiseFn<f64, 2> + ?Sized),
        assets: &Assets,
    ) -> Self {
        let model = glm::scale(&glm::identity(), &glm::vec3(SCENE_SIZE, SCENE_SIZE, 1.0));
        let height_tex = Texture::from_noise(height_fn, (0., SCENE_SIZE, 0., SCENE_SIZE), 256);
        let variant_tex = Texture::from_noise(variant_fn, (0., SCENE_SIZE, 0., SCENE_SIZE), 256);

        TerrainEntity {
            vao: assets.terrain_quad_mesh.clone(),
            displacement: Rc::new(height_tex),
            variant: Rc::new(variant_tex),
            albedo_xy1: assets.moss_tex.clone(),
            albedo_xy2: assets.ground_tex.clone(),
            albedo_xz: assets.rock_tex.clone(),
            albedo_yz: assets.rock_tex.clone(),
            model,
            world_to_uv: glm::scale2d(
                &glm::identity(),
                &glm::vec2(1.0 / SCENE_SIZE, 1.0 / SCENE_SIZE),
            ),
            shader: assets.terrain_shader.clone(),
        }
    }

    pub fn bushes(
        height_fn: &(impl NoiseFn<f64, 2> + ?Sized),
        variant_fn: &(impl NoiseFn<f64, 2> + ?Sized),
        assets: &Assets,
    ) -> Self {
        let model = glm::scale(&glm::identity(), &glm::vec3(SCENE_SIZE, SCENE_SIZE, 1.0));
        let height_tex = Texture::from_noise(height_fn, (0., SCENE_SIZE, 0., SCENE_SIZE), 256);
        let variant_tex = Texture::from_noise(variant_fn, (0., SCENE_SIZE, 0., SCENE_SIZE), 256);

        TerrainEntity {
            vao: assets.terrain_quad_mesh.clone(),
            displacement: Rc::new(height_tex),
            variant: Rc::new(variant_tex),
            albedo_xy1: assets.transparent_tex.clone(),
            albedo_xy2: assets.bush_tex.clone(),
            albedo_xz: assets.transparent_tex.clone(),
            albedo_yz: assets.transparent_tex.clone(),
            model,
            world_to_uv: glm::scale2d(
                &glm::identity(),
                &glm::vec2(1.0 / SCENE_SIZE, 1.0 / SCENE_SIZE),
            ),
            shader: assets.terrain_shader.clone(),
        }
    }
}

impl Renderable for TerrainEntity {
    fn render(&self, view_proj_mat: &glm::Mat4) {
        unsafe {
            self.shader.activate();
            gl::UniformMatrix4fv(
                self.shader.get_uniform_location("view_proj"),
                1,
                gl::FALSE,
                view_proj_mat.as_ptr(),
            );
            gl::UniformMatrix4fv(
                self.shader.get_uniform_location("model_mat"),
                1,
                gl::FALSE,
                self.model.as_ptr(),
            );
            gl::UniformMatrix3fv(
                self.shader.get_uniform_location("world_to_uv"),
                1,
                gl::FALSE,
                self.world_to_uv.as_ptr(),
            );

            self.displacement.activate(0);
            gl::Uniform1i(self.shader.get_uniform_location("displacement_map"), 0);
            self.variant.activate(5);
            gl::Uniform1i(self.shader.get_uniform_location("variant_map"), 5);

            self.albedo_xy1.activate(4);
            gl::Uniform1i(self.shader.get_uniform_location("terrain_albedo_xy1"), 4);
            self.albedo_xy2.activate(1);
            gl::Uniform1i(self.shader.get_uniform_location("terrain_albedo_xy2"), 1);
            self.albedo_xz.activate(2);
            gl::Uniform1i(self.shader.get_uniform_location("terrain_albedo_xz"), 2);
            self.albedo_yz.activate(3);
            gl::Uniform1i(self.shader.get_uniform_location("terrain_albedo_yz"), 3);
        }

        self.vao.render();
    }
}

/// Nameable type for the height noise fn.
///
/// Sadly requires indirection, as to implement Seedable, you have to able to
/// name the type as it seems.
pub struct RockMap {
    f: Box<dyn NoiseFn<f64, 2> + 'static>,
    seed: u32,
}

impl RockMap {
    pub fn new(seed: u32) -> RockMap {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed as u64);

        // Large scale features, but not very much detail
        let rockyness = noise::Fbm::<noise::Value>::new(rng.gen())
            .set_octaves(4)
            .set_frequency(0.5);
        // Flatten out a lot of the values.
        let rockyness_threshold = 0.5;
        let rockyness = noise::ScaleBias::new(rockyness).set_bias(-rockyness_threshold);
        let rockyness = noise::Max::new(rockyness, noise::Constant::new(0.0));
        let rockyness = noise::ScaleBias::new(rockyness).set_scale(3.0);
        let rockyness = noise::Min::new(rockyness, noise::Constant::new(1.0));

        // Manhattan distances to create hard ridges
        let ridges = noise::Worley::new(rng.gen())
            .set_frequency(1.0)
            .set_distance_function(&noise::core::worley::distance_functions::manhattan)
            .set_return_type(noise::core::worley::ReturnType::Distance);
        let ridges = Slice4D { func_4d: ridges };

        let rocks = noise::Multiply::new(rockyness, ridges);
        let rocks = ScaleBias::new(rocks).set_scale(0.2);

        let noise = rocks;

        RockMap {
            f: Box::new(noise),
            seed,
        }
    }
}

impl Default for RockMap {
    fn default() -> Self {
        RockMap::new(0)
    }
}

impl NoiseFn<f64, 2> for RockMap {
    fn get(&self, point: [f64; 2]) -> f64 {
        self.f.get(point)
    }
}

impl noise::Seedable for RockMap {
    fn set_seed(self, seed: u32) -> Self {
        RockMap::new(seed)
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}

pub fn height_map(base: Rc<image::RgbaImage>, seed: u32) -> impl NoiseFn<f64, 2> + 'static {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed as u64);

    let rocks = noise::Fbm::<RockMap>::new(rng.gen())
        .set_octaves(3)
        .set_lacunarity(3.0)
        .set_persistence(0.3)
        .set_frequency(0.8);

    let rocks = ScaleBias::new(rocks).set_scale(0.8);

    let height = noise::Fbm::<noise::Value>::new(rng.gen())
        .set_octaves(6)
        .set_frequency(0.2);
    let height = ScaleBias::new(height).set_scale(0.3).set_bias(0.3);

    let base_height = noise::Power::new(
        ImageNoiseFnWrapper::new_red(base),
        noise::Constant::new(2.0),
    );
    let base_height = noise::ScaleBias::new(base_height).set_scale(2.0);

    noise::Add::new(base_height, noise::Add::new(rocks, height))
}

pub fn variant_map(base: Rc<image::RgbaImage>, seed: u32) -> impl NoiseFn<f64, 2> + 'static {
    let noise = noise::Fbm::<noise::Value>::new(seed)
        .set_octaves(6)
        .set_frequency(0.2);
    let noise = ScaleBias::new(noise).set_scale(0.5).set_bias(0.5);

    let bushiness = noise::Power::new(
        ImageNoiseFnWrapper::new_green(base),
        noise::Constant::new(2.0),
    );
    let bushiness = noise::ScaleBias::new(bushiness).set_scale(0.3);

    let total = noise::Add::new(bushiness, noise);
    let total = noise::Clamp::new(total).set_bounds(0.0, 1.0);

    noise::Power::new(total, noise::Constant::new(2.0))
}

pub fn bush_heights(seed: u32) -> impl NoiseFn<f64, 2> + 'static {
    const MIN_HEIGHT: f64 = 0.02;
    const MAX_HEIGHT: f64 = 0.1;
    let noise = noise::Fbm::<noise::Value>::new(seed)
        .set_octaves(6)
        .set_frequency(0.8);
    // Move from [-1, 1] to interval [MIN_HEIGHT, MAX_HEIGHT]
    let noise = ScaleBias::new(noise).set_scale(0.5).set_bias(0.5);
    let noise = ScaleBias::new(noise)
        .set_scale(MAX_HEIGHT - MIN_HEIGHT)
        .set_bias(MIN_HEIGHT);

    noise
}

struct Slice4D<F: NoiseFn<f64, 4>> {
    func_4d: F,
}

impl<F> NoiseFn<f64, 2> for Slice4D<F>
where
    F: NoiseFn<f64, 4>,
{
    fn get(&self, point: [f64; 2]) -> f64 {
        self.func_4d
            .get([0.5 * point[0] + point[1], point[1], point[0], 2. * point[1]])
    }
}
