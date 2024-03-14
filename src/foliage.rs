use crate::mesh::{ElementMeshVAO, InstancedMeshesVAO, Mesh};
use crate::renderer::Renderable;
use crate::shader::{Shader, ShaderBuilder};

use nalgebra_glm as glm;
use std::rc::Rc;

#[derive(Clone)]
pub struct ShrubEntities {
    pub color: glm::Vec3,
    pub vao: Rc<InstancedMeshesVAO>,
    pub shader: Rc<Shader>,
}

impl ShrubEntities {
    pub fn from_scratch(num: usize) -> Self {
        let shader = unsafe {
            ShaderBuilder::new()
                .with_shader_file("shaders/composable_instanced.vert")
                .with_shader_file("shaders/composable_shaded_color.frag")
                .link()
                .expect("Simple shader had errors. See stdout.")
        };

        let mesh = Mesh::load("models/shrub1.obj");
        let mesh_vao = ElementMeshVAO::new_from_mesh(&mesh);

        let color = glm::vec3(71. / 255., 49. / 255., 68. / 255.);
        let scale = 0.7;

        let mut model_mats = Vec::with_capacity(num);
        for i in 0..num {
            let f = i as f32 / num as f32;

            let pos = glm::vec3(6. * f - 3., -2., 0.0);
            // For some very weird ass reason, do the translate & scale functions right multiply,
            // thus for scale than translate, I need to translate then scale...
            let model = glm::scale(
                &glm::rotate_z(&glm::translate(&glm::identity(), &pos), f * 6.28),
                &glm::vec3(scale, scale, scale),
            );

            model_mats.push(model);
        }

        let instanced_vao = InstancedMeshesVAO::from_existing_with_models(mesh_vao, &model_mats);

        ShrubEntities {
            color,
            vao: Rc::new(instanced_vao),
            shader: Rc::new(shader),
        }
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
            gl::Uniform3fv(
                self.shader.get_uniform_location("color"),
                1,
                self.color.as_ptr(),
            );
        }

        self.vao.render();
    }
}
