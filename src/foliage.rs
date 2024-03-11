use crate::mesh::{Mesh, VAO};
use crate::renderer::Renderable;
use crate::shader::{Shader, ShaderBuilder};

use nalgebra_glm as glm;
use std::rc::Rc;

#[derive(Clone)]
pub struct ShrubEntity {
    pub color: glm::Vec3,
    pub model: glm::Mat4,
    pub model_normal: glm::Mat3,
    pub vao: Rc<VAO>,
    pub shader: Rc<Shader>,
}

impl ShrubEntity {
    pub fn from_scratch() -> Self {
        let shader = unsafe {
            ShaderBuilder::new()
                .with_shader_file("shaders/simple_perspective.vert")
                .with_shader_file("shaders/simple_const_color.frag")
                .link()
                .expect("Simple shader had errors. See stdout.")
        };

        let mesh = Mesh::load("models/shrub1.obj");
        let mesh_vao = VAO::new_from_mesh(&mesh);

        let color = glm::vec3(71. / 255., 49. / 255., 68. / 255.);
        let scale = 0.3;
        let pos = glm::vec3(1.0, -2.0, 0.0);
        // For some very weird ass reason, do the translate & scale functions right multiply,
        // thus for scale than translate, I need to translate then scale...
        let model = glm::scale(
            &glm::translate(&glm::identity(), &pos),
            &glm::vec3(scale, scale, scale),
        );
        let model_normal = glm::mat4_to_mat3(&glm::transpose(&glm::inverse(&model)));

        ShrubEntity {
            color,
            model,
            model_normal,
            vao: Rc::new(mesh_vao),
            shader: Rc::new(shader),
        }
    }
}

impl Renderable for ShrubEntity {
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
            gl::UniformMatrix4fv(
                self.shader.get_uniform_location("model_mat"),
                1,
                gl::FALSE,
                self.model.as_ptr(),
            );
            gl::UniformMatrix3fv(
                self.shader.get_uniform_location("normal_mat"),
                1,
                gl::FALSE,
                self.model_normal.as_ptr(),
            );
            gl::Uniform3fv(
                self.shader.get_uniform_location("color"),
                1,
                self.color.as_ptr(),
            );
        }

        self.vao.render();
    }
}
