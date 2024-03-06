use std::ffi::{CStr, CString};

use gl::types::GLfloat;
use glutin::display::GlDisplay;
use nalgebra_glm as glm;

use crate::shader::{Shader, ShaderBuilder};

pub struct Renderer {
    shader: Shader,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    aspect_ratio: f32,
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

        let vao = unsafe {
            let mut vao = std::mem::zeroed();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            vao
        };

        let vbo = unsafe {
            let mut vbo = std::mem::zeroed();
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0, // position
                2,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl::VertexAttribPointer(
                1, // color
                3,
                gl::FLOAT,
                0,
                5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
                (2 * std::mem::size_of::<f32>()) as *const () as *const _,
            );
            gl::EnableVertexAttribArray(0 as gl::types::GLuint);
            gl::EnableVertexAttribArray(1 as gl::types::GLuint);
            vbo
        };

        let mut viewport: [gl::types::GLint; 4] = [0; 4];
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
        }
        let viewport_width = viewport[2] as gl::types::GLsizei;
        let viewport_height = viewport[3] as gl::types::GLsizei;
        let aspect_ratio = viewport_width as f32 / viewport_height as f32;

        Self {
            shader,
            vao,
            vbo,
            aspect_ratio,
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

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::ClearColor(red, green, blue, alpha);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
        self.aspect_ratio = width as f32 / height as f32;
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

fn get_gl_string(variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl::GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

#[rustfmt::skip]
static VERTEX_DATA: [f32; 30] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
     0.5,  0.5,  0.0,  1.0,  0.0,
     0.5, -0.5,  0.0,  0.0,  1.0,

    -0.5, -0.5,  1.0,  0.0,  0.0,
    -0.5,  0.5,  0.0,  1.0,  0.0,
     0.5,  0.5,  0.0,  0.0,  1.0,
];
