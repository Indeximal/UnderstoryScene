use nalgebra_glm as glm;
use std::rc::Rc;

use crate::mesh::{ElementMeshVAO, Mesh};
use crate::renderer::Renderable;
use crate::shader::{Shader, ShaderBuilder};
use crate::texture::Texture;

use noise::{Add, ScaleBias};
use noise::{NoiseFn, ScalePoint};

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
            &glm::vec3(0., 0., -0.01),
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

pub fn height_map(seed: u32) -> impl NoiseFn<f64, 2> + 'static {
    let octave0 = ScaleBias::new(ScalePoint::new(noise::Value::new(seed + 1)).set_scale(2.))
        .set_scale(0.5)
        .set_bias(0.5);
    let octave1 = ScaleBias::new(ScalePoint::new(noise::Value::new(seed + 2)).set_scale(4.))
        .set_scale(0.25)
        .set_bias(0.25);
    let octave2 = ScaleBias::new(ScalePoint::new(noise::Value::new(seed + 3)).set_scale(8.))
        .set_scale(0.125)
        .set_bias(0.125);

    let noise = Add::new(Add::new(octave0, octave1), octave2);
    let noise = ScaleBias::new(noise).set_scale(0.2);

    noise
}
