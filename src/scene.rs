use crate::foliage::ShrubEntitiesBuilder;
use crate::mesh::{ElementMeshVAO, Mesh};
use crate::renderer::Renderable;
use crate::shader::{Shader, ShaderBuilder};
use crate::terrain::TerrainEntity;
use crate::texture::Texture;
use nalgebra_glm as glm;
use noise::NoiseFn;
use std::rc::Rc;
use std::time::Instant;

pub struct Scene {
    pub entities: Vec<Box<dyn Renderable>>,
    pub start_time: Instant,
}

macro_rules! time {
    ($name:expr, $block:expr) => {{
        let start = Instant::now();
        let result = $block;
        println!("Loading {} took {}ms", $name, start.elapsed().as_millis());
        result
    }};
}

impl Scene {
    pub fn load() -> Self {
        time!("scene", {
            let height_map: Rc<dyn NoiseFn<f64, 2>> =
                time!("height map", Rc::new(crate::terrain::height_map(424242)));

            let ground_entity = time!("terrain", TerrainEntity::from_scratch(height_map.as_ref()));

            // TODO: additional noise, not just offset.
            let _leaves_entity = time!("leaves texture", {
                let tex = Texture::from_file("textures/leaves_masked1.png")
                    .expect("Loading leaves texture failed");
                tex.enable_mipmap();
                TerrainEntity {
                    albedo_xy: Rc::new(tex),
                    model: glm::translate(&ground_entity.model, &glm::vec3(0.0, 0.0, 0.03)),
                    ..ground_entity.clone()
                }
            });

            let base_plate = time!("base plate", BasePlate::from_scratch());

            let shrubs = time!("shrubs", {
                ShrubEntitiesBuilder::new()
                    .with_approx_number_of_entities(500)
                    .on_height_map(&height_map)
                    .with_color(&[71. / 255., 49. / 255., 68. / 255., 1.0])
                    .with_model_file("models/shrub2.obj")
                    .with_z_scale_range(0.4, 1.0)
                    .load(424247)
            });

            let bushes = time!("bushes", {
                ShrubEntitiesBuilder::new()
                    .with_approx_number_of_entities(2000)
                    .on_height_map(&height_map)
                    .with_texture("textures/bush_masked1.png")
                    .with_model_file("models/bush1.obj")
                    .with_z_scale_range(0.9, 1.0)
                    .load(777781)
            });

            let entities: Vec<Box<dyn Renderable>> = vec![
                Box::new(ground_entity),
                // Box::new(leaves_entity),
                Box::new(base_plate),
                Box::new(shrubs),
                Box::new(bushes),
            ];

            Scene {
                entities,
                start_time: Instant::now(),
            }
        })
    }

    pub fn background_color(&self) -> (f32, f32, f32, f32) {
        (186. / 255., 219. / 255., 222. / 255., 1.0) // A sky blue
    }

    pub fn eye_position(&self) -> glm::Vec3 {
        let t = self.start_time.elapsed().as_secs_f32();
        // Different phase and frequency for random looking movement
        glm::vec3(
            0.1 * (t * 0.32 + 1.).sin(),
            0.03 * (t * 0.33 + 2.).sin(),
            0.05 * (t * 0.34 + 3.).sin(),
        ) + glm::vec3(0.0, -4.0, 1.7) // Stand behind the scene with eye height 170cm
    }

    pub fn look_at(&self) -> glm::Vec3 {
        glm::vec3(0.0, -1.0, 0.2) // Look at the floor near the center
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
