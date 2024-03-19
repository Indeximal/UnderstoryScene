use nalgebra_glm as glm;
use rand::{Rng, SeedableRng};
use std::rc::Rc;

use crate::mesh::{ElementMeshVAO, Mesh};
use crate::renderer::Renderable;
use crate::shader::{Shader, ShaderBuilder};
use crate::texture::Texture;

use noise::{MultiFractal, NoiseFn, ScaleBias};

/// The side length of the centered terrain square in meters.
pub const TERRAIN_SIZE: f32 = 6.0;

#[derive(Clone)]
pub struct TerrainEntity {
    pub vao: Rc<ElementMeshVAO>,
    pub displacement: Rc<Texture>,
    pub albedo: Rc<Texture>,
    pub model: glm::Mat4,
    pub shader: Rc<Shader>,
    // A matrix that will right multiply a world coordinate into a uv coordinate
    pub world_to_uv: glm::Mat3,
}

impl TerrainEntity {
    /// This is not particularly smart to use more than once, as it
    /// does not share textures, shaders or buffers.
    pub fn from_scratch(height_fn: &(impl NoiseFn<f64, 2> + ?Sized)) -> Self {
        let terrain_shader = unsafe {
            ShaderBuilder::new()
                .with_shader_file("shaders/terrain.vert")
                .with_shader_file("shaders/terrain.frag")
                .link()
                .expect("Terrain shader had errors. See stdout.")
        };

        let quad = Mesh::quad_mesh(256);
        let quad_vao = ElementMeshVAO::new_from_mesh(&quad);
        let height_tex = Texture::from_noise(
            height_fn,
            (
                -TERRAIN_SIZE / 2.,
                TERRAIN_SIZE / 2.,
                -TERRAIN_SIZE / 2.,
                TERRAIN_SIZE / 2.,
            ),
            128,
        );
        let model = glm::scale(
            &glm::identity(),
            &glm::vec3(TERRAIN_SIZE / 2.0, TERRAIN_SIZE / 2.0, 1.0),
        );
        let uv_to_world = glm::translate2d(
            &glm::scale2d(&glm::identity(), &glm::vec2(TERRAIN_SIZE, TERRAIN_SIZE)),
            &glm::vec2(-0.5, -0.5),
        );

        let albedo =
            Texture::from_file("textures/grass1.jpeg").expect("Failed to load ground texture");
        albedo.enable_mipmap();

        TerrainEntity {
            vao: Rc::new(quad_vao),
            displacement: Rc::new(height_tex),
            albedo: Rc::new(albedo),
            model,
            world_to_uv: glm::inverse(&uv_to_world),
            shader: Rc::new(terrain_shader),
        }
    }
}

pub struct BasePlate {
    vao: Rc<ElementMeshVAO>,
    model: glm::Mat4,
    shader: Rc<Shader>,
}

impl BasePlate {
    pub fn from_scratch() -> Self {
        let shader = unsafe {
            ShaderBuilder::new()
                .with_shader_file("shaders/composable_perspective.vert")
                .with_shader_file("shaders/composable_const_color.frag")
                .link()
                .expect("Simple shader had errors. See stdout.")
        };

        let quad = Mesh::quad();
        let quad_vao = ElementMeshVAO::new_from_mesh(&quad);
        // 100 by 100 meters size
        let model = glm::translate(
            &glm::scale(&glm::identity(), &glm::vec3(100.0, 100.0, 1.0)),
            &glm::vec3(0., 0., -0.5),
        );

        BasePlate {
            vao: Rc::new(quad_vao),
            model,
            shader: Rc::new(shader),
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

            self.albedo.activate(1);
            gl::Uniform1i(self.shader.get_uniform_location("terrain_albedo"), 1);
        }

        self.vao.render();
    }
}

impl Renderable for BasePlate {
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

            gl::Uniform3fv(
                self.shader.get_uniform_location("color"),
                1,
                (&[79.0f32 / 255., 63. / 255., 45. / 255.]) as *const _,
            );
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
        // let worley_values = noise::Worley::new(seed).set_frequency(1.0);
        // let worley_values = noise::Max::new(worley_values, noise::Constant::new(0.0));
        // let worley_distance = noise::Worley::new(seed)
        //     .set_frequency(1.0)
        //     .set_return_type(noise::core::worley::ReturnType::Distance);
        // let noise = noise::Multiply::new(worley_values, worley_distance);

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

pub fn height_map(seed: u32) -> impl NoiseFn<f64, 2> + 'static {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed as u64);

    let rocks = noise::Fbm::<RockMap>::new(rng.gen())
        .set_octaves(3)
        .set_lacunarity(3.0)
        .set_persistence(0.3)
        .set_frequency(0.8);

    let rocks = ScaleBias::new(rocks).set_scale(1.1);

    let height = noise::Fbm::<noise::Value>::new(rng.gen())
        .set_octaves(6)
        .set_frequency(0.2);
    let height = ScaleBias::new(height).set_scale(0.3);

    noise::Add::new(rocks, height)
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
