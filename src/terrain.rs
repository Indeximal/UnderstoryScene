use nalgebra_glm as glm;
use std::rc::Rc;

use crate::mesh::{Mesh, VAO};
use crate::renderer::Renderable;
use crate::shader::{Shader, ShaderBuilder};
use crate::texture::{format, Texture};

use noise::{Add, ScaleBias};
use noise::{NoiseFn, ScalePoint};

pub struct TerrainEntity {
    vao: Rc<VAO>,
    displacement: Rc<Texture>,
    model: glm::Mat4,
    shader: Rc<Shader>,
}

impl TerrainEntity {
    /// This is not particularly smart to use more than once, as it
    /// does not share textures, shaders or buffers.
    pub fn from_scratch() -> Self {
        let terrain_shader = unsafe {
            ShaderBuilder::new()
                .with_shader_file("shaders/terrain.vert")
                .with_shader_file("shaders/terrain.frag")
                .link()
                .expect("Terrain shader had errors. See stdout.")
        };

        let quad = Mesh::quad_mesh(256);
        let quad_vao = VAO::new_from_mesh(&quad);
        let noise = noise_texture(128, 128, 12345678);
        // 6 by 6 meters size
        let model = glm::scale(&glm::identity(), &glm::vec3(3.0, 3.0, 1.0));

        TerrainEntity {
            vao: Rc::new(quad_vao),
            displacement: Rc::new(noise),
            model,
            shader: Rc::new(terrain_shader),
        }
    }
}

fn noise_texture(width: u32, height: u32, seed: u32) -> Texture {
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

    // Copied the internals of [`noise::utils::PlaneMapBuilder`], since I
    // want to get the vector directly.
    let mut result_map = vec![0.0f32; (width * height) as usize];

    let x_bounds = (-1.0, 1.0);
    let y_bounds = (-1.0, 1.0);
    let x_step = (x_bounds.1 - x_bounds.0) / width as f64;
    let y_step = (y_bounds.1 - y_bounds.0) / height as f64;

    for y in 0..height {
        for x in 0..width {
            let current_y = y_bounds.0 + y_step * y as f64;
            let current_x = x_bounds.0 + x_step * x as f64;

            result_map[(y * width + x) as usize] = noise.get([current_x, current_y]) as f32;
        }
    }

    Texture::new::<f32, format::GrayScale>(width, height, result_map.as_slice())
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

            self.displacement.activate(0);
            gl::Uniform1i(self.shader.get_uniform_location("displacement_map"), 0);
        }

        self.vao.render();
    }
}
