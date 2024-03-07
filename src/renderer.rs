use std::ffi::{CStr, CString};

use gl::types::GLfloat;
use glutin::display::GlDisplay;
use nalgebra_glm as glm;

use crate::mesh::{Mesh, VAO};
use crate::shader::{Shader, ShaderBuilder};
use crate::texture::Texture;

pub struct Renderer {
    aspect_ratio: f32,
    shader: Shader,
    quad: VAO,
    noise: Texture,
}

impl Renderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
        // Haha that seems like the only truely unsafe thing here, yet it is the only
        // one not marked as unsafe xD
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        if let Some(renderer) = get_gl_string(gl::RENDERER) {
            println!("Running on {}", renderer.to_string_lossy());
        }
        if let Some(version) = get_gl_string(gl::VERSION) {
            println!("OpenGL Version {}", version.to_string_lossy());
        }
        if let Some(shaders_version) = get_gl_string(gl::SHADING_LANGUAGE_VERSION) {
            println!("Shaders version on {}", shaders_version.to_string_lossy());
        }

        let shader = unsafe {
            ShaderBuilder::new()
                .with_shader_file("shaders/simple.vert")
                .with_shader_file("shaders/simple.frag")
                .link()
                .expect("Shader had errors. See stdout.")
        };

        let quad = Mesh::quad_mesh(32);
        let quad_vao = VAO::new_from_mesh(&quad);

        let mut viewport: [gl::types::GLint; 4] = [0; 4];
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
        }
        let aspect_ratio = viewport[2] as f32 / viewport[3] as f32;

        let noise = Texture::noise(256, 256, 12345678);

        Self {
            aspect_ratio,
            shader,
            quad: quad_vao,
            noise,
        }
    }

    pub fn draw(&self) {
        self.draw_with_clear_color(0.1, 0.1, 0.1, 1.0)
    }

    pub fn draw_with_clear_color(
        &self,
        red: GLfloat,
        green: GLfloat,
        blue: GLfloat,
        alpha: GLfloat,
    ) {
        unsafe {
            gl::ClearColor(red, green, blue, alpha);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let projection: glm::Mat4 = glm::perspective(
            self.aspect_ratio,
            80.0 * std::f32::consts::PI / 180.,
            0.1,
            350.0,
        );
        let camera_transform = glm::look_at(
            &glm::vec3(0.0, -2.0, 1.6),
            &glm::vec3(0.0, 0.0, 0.5),
            &glm::Vec3::z_axis(),
        );

        let view_proj_mat = projection * camera_transform;

        unsafe {
            self.shader.activate();
            gl::UniformMatrix4fv(
                self.shader.get_uniform_location("view_proj"),
                1,
                gl::FALSE,
                view_proj_mat.as_ptr(),
            );

            self.noise.activate(0);
            gl::Uniform1i(self.shader.get_uniform_location("noise_texture"), 0);
        }

        self.quad.render();
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
        self.aspect_ratio = width as f32 / height as f32;
    }
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}
