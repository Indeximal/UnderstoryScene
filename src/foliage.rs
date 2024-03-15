use crate::mesh::{ElementMeshVAO, InstancedMeshesVAO, Mesh};
use crate::renderer::Renderable;
use crate::shader::{Shader, ShaderBuilder};

use nalgebra_glm as glm;
use noise::{MultiFractal, NoiseFn};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Poisson, Uniform};
use std::rc::Rc;

#[derive(Clone)]
pub struct ShrubEntities {
    pub color: glm::Vec3,
    pub vao: Rc<InstancedMeshesVAO>,
    pub shader: Rc<Shader>,
}

impl ShrubEntities {
    pub fn from_scratch(num: usize, seed: u32, height_map: &impl NoiseFn<f64, 2>) -> Self {
        let shader = unsafe {
            ShaderBuilder::new()
                .with_shader_file("shaders/composable_instanced.vert")
                .with_shader_file("shaders/composable_shaded_color.frag")
                .link()
                .expect("Simple shader had errors. See stdout.")
        };

        let mesh = Mesh::load("models/shrub2.obj");
        let mesh_vao = ElementMeshVAO::new_from_mesh(&mesh);

        let color = glm::vec3(71. / 255., 49. / 255., 68. / 255.);
        let scale = 0.7;

        let distr = probability_distribution(num as f64 / 36.0, seed);
        let positions =
            generate_points_on_distribution(distr, (-3.0, 3.0, -3.0, 3.0), seed as u64 + 1);

        println!("Spawned {} shrubs", positions.len());

        // For some very weird ass reason do the translate & scale functions right multiply,
        // thus for scale than translate, I need to translate then scale...

        let mut rng = StdRng::seed_from_u64(seed as u64 + 2);

        let model_mats: Vec<glm::Mat4> = positions
            .into_iter()
            .map(|p| {
                // TODO: rotation based on height gradient
                let rotation_angle: f32 = rng.sample(Uniform::new(0.0, 6.28));
                // TODO: scale in a more natural distribution
                let z_scale: f32 = rng.sample(Uniform::new(0.4, 1.0));

                glm::scale(
                    &glm::rotate_z(
                        &glm::translate(
                            &glm::identity(),
                            &glm::vec3(p.x, p.y, height_map.get([p.x as f64, p.y as f64]) as f32),
                        ),
                        rotation_angle,
                    ),
                    &glm::vec3(scale, scale, scale * z_scale),
                )
            })
            .collect();

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

/// Note that the `density` might not actually be the average, since
/// this is too difficult to enforce. Just some scale approximately in the same
/// order as the average.
///
/// FIXME: more consitent shrub number
fn probability_distribution(density: f64, seed: u32) -> impl NoiseFn<f64, 2> {
    let noise = noise::Fbm::<noise::Perlin>::new(seed)
        .set_octaves(4) // Not very much detail required
        .set_frequency(0.2); // Large scale features approx 5 meters large

    // Transform from [-1, 1] to [0, density]
    let noise = noise::ScaleBias::new(noise)
        .set_bias(1.0)
        .set_scale(density / 2.0);

    // // Make it less uniform.
    // let noise = noise::Power::new(noise, noise::Constant::new(5.0));
    noise
}

/// Generates random points in a rectangle according to the given density distribution.
///
/// It does this sampling the distribution at discrete locations and then drawing
/// from a poission variable N with expected value equal to the density times the
/// area of the chunk. In this tiny chunk the N points are uniformly distributed.
///
/// Given the same seed and distribution, this function is deterministic.
///
/// The distribution is assumed to be normalized, ie the value of an integral over a
/// unit area should be the number of points in this area.
/// The unit is therefore [number of points / area].
fn generate_points_on_distribution(
    distribution: impl NoiseFn<f64, 2>,
    (x_min, x_max, y_min, y_max): (f32, f32, f32, f32),
    seed: u64,
) -> Vec<glm::Vec2> {
    let mut points = Vec::new();
    let resolution = 100;

    let dx = (x_max - x_min) / resolution as f32;
    let dy = (y_max - y_min) / resolution as f32;
    let area = dx * dy;

    let mut rng = StdRng::seed_from_u64(seed);

    for x in 0..resolution {
        for y in 0..resolution {
            let fx = x_min + dx * x as f32;
            let fy = y_min + dy * y as f32;

            let density =
                distribution.get([(fx + dx / 2.).into(), (fy + dy / 2.).into()]) as f32 * area;
            if density <= 0.0 {
                continue;
            }

            let poisson = Poisson::new(density).expect("density should be positive and a number");
            let num_points_in_chunk = poisson.sample(&mut rng);

            for _ in 0..num_points_in_chunk as usize {
                let point = glm::vec2(fx + dx * rng.gen::<f32>(), fy + dy * rng.gen::<f32>());
                points.push(point);
            }
        }
    }

    points
}
