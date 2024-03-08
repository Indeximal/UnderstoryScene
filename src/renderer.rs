use std::ffi::{CStr, CString};

use gl::types::GLfloat;
use glutin::display::GlDisplay;
use nalgebra_glm as glm;

use crate::terrain::{BasePlate, TerrainEntity};

pub trait Renderable {
    fn render(&self, view_proj_mat: &glm::Mat4);
}

pub struct Renderer {
    aspect_ratio: f32,
    entities: Vec<Box<dyn Renderable>>,
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

        let mut viewport: [gl::types::GLint; 4] = [0; 4];
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
        }
        let aspect_ratio = viewport[2] as f32 / viewport[3] as f32;

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
        }

        Self {
            aspect_ratio,
            entities: vec![
                Box::new(TerrainEntity::from_scratch()),
                Box::new(BasePlate::from_scratch()),
            ],
        }
    }

    pub fn draw(&self) {
        self.draw_with_clear_color(186. / 255., 219. / 255., 222. / 255., 1.0)
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
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let projection: glm::Mat4 = glm::perspective(
            self.aspect_ratio,
            65.0 * std::f32::consts::PI / 180.,
            0.1,  // 10 cm
            50.0, // 50 m
        );
        let camera_transform = glm::look_at(
            &glm::vec3(0.0, -4.0, 1.7), // Stand behind the scene with eye height 170cm
            &glm::vec3(0.0, -1.0, 0.2), // Look at the floor near the center
            &glm::Vec3::z_axis(),
        );
        let view_proj_mat = projection * camera_transform;

        for entity in &self.entities {
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
