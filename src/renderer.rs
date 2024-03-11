use std::ffi::{CStr, CString};

use glutin::display::GlDisplay;
use nalgebra_glm as glm;

pub trait Renderable {
    fn render(&self, view_proj_mat: &glm::Mat4);
}

pub struct Renderer {
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
        let mut max_texture_size: gl::types::GLint = 0;
        unsafe {
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_texture_size);
        }
        println!("Max texture size: {max_texture_size}px");

        let mut viewport: [gl::types::GLint; 4] = [0; 4];
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
        }
        let aspect_ratio = viewport[2] as f32 / viewport[3] as f32;

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }

        Self { aspect_ratio }
    }

    pub fn draw(&mut self, scene: &crate::scene::Scene) {
        let (red, green, blue, alpha) = scene.background_color();
        unsafe {
            gl::ClearColor(red, green, blue, alpha);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let projection: glm::Mat4 = glm::perspective(
            self.aspect_ratio,
            65.0 * std::f32::consts::PI / 180.,
            0.1,  // 10 cm
            50.0, // 50 m
        );
        let camera_transform = glm::look_at(
            &scene.eye_position(),
            &scene.look_at(),
            &glm::Vec3::z_axis(),
        );
        let view_proj_mat = projection * camera_transform;

        for entity in &scene.entities {
            entity.render(&view_proj_mat);
        }
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
