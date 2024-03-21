use crate::mesh::{ElementMeshVAO, Mesh};
use crate::shader::{Shader, ShaderBuilder};
use crate::texture::Texture;

use std::rc::Rc;
use std::time::Instant;

macro_rules! time {
    ($name:expr, $block:expr) => {{
        let start = Instant::now();
        let result = $block;
        println!("Loading {} took {}ms", $name, start.elapsed().as_millis());
        result
    }};
}

/// These are all the loaded things that are shared between generated scenes,
/// ie textures, models and shaders that do not depend on the seed.
pub struct Assets {
    pub moss_tex: Rc<Texture>,
    pub rock_tex: Rc<Texture>,
    pub bush_tex: Rc<Texture>,
    pub shrub_tex: Rc<Texture>,
    pub bark_tex: Rc<Texture>,

    // These could technically also share the VAO, but since the instance data
    // is dynamic, this would be weird.
    pub shrub_model: Rc<Mesh>,
    pub bush_model: Rc<Mesh>,
    pub tree_model: Rc<Mesh>,
    pub terrain_quad_mesh: Rc<ElementMeshVAO>,

    pub terrain_shader: Rc<Shader>,
    pub foliage_shader: Rc<Shader>,
}

impl Assets {
    pub fn load() -> Self {
        time!(
            "ASSETS",
            Assets {
                moss_tex: time!("moss texture", {
                    let tex = Texture::from_file("textures/moss1.jpeg")
                        .expect("Loading moss texture failed");
                    tex.enable_mipmap();
                    Rc::new(tex)
                }),
                rock_tex: time!("rock texture", {
                    let tex = Texture::from_file("textures/rock1.jpeg")
                        .expect("Loading rock texture failed");
                    tex.enable_mipmap();
                    Rc::new(tex)
                }),
                bush_tex: time!("bush texture", {
                    let tex = Texture::from_file("textures/bush_masked1.png")
                        .expect("Loading bush texture failed");
                    tex.enable_mipmap();
                    Rc::new(tex)
                }),
                shrub_tex: time!("shrub texture", {
                    let tex = Texture::new::<f32, crate::texture::format::RGBA>(
                        1,
                        1,
                        &[71. / 255., 49. / 255., 68. / 255., 1.0],
                    );
                    tex.enable_mipmap();
                    Rc::new(tex)
                }),
                bark_tex: time!("bark texture", {
                    let tex = Texture::from_file("textures/bark1.jpeg")
                        .expect("Loading bark texture failed");
                    tex.enable_mipmap();
                    Rc::new(tex)
                }),

                shrub_model: time!("shrub model", {
                    let model = Mesh::load("models/shrub2.obj");
                    Rc::new(model)
                }),
                bush_model: time!("bush model", {
                    let model = Mesh::load("models/bush1.obj");
                    Rc::new(model)
                }),
                tree_model: time!("tree model", {
                    let model = Mesh::load("models/tree1.obj");
                    Rc::new(model)
                }),
                terrain_quad_mesh: time!("terrain mesh", {
                    let quad = Mesh::quad_mesh(256);
                    let quad_vao = ElementMeshVAO::new_from_mesh(&quad);
                    Rc::new(quad_vao)
                }),

                terrain_shader: time!("terrain shader", {
                    let shader = ShaderBuilder::new()
                        .with_shader_file("shaders/terrain.vert")
                        .with_shader_file("shaders/terrain.frag")
                        .link()
                        .expect("Terrain shader had errors. See stdout.");
                    Rc::new(shader)
                }),
                foliage_shader: time!("foliage shader", {
                    let shader = ShaderBuilder::new()
                        .with_shader_file("shaders/composable_instanced.vert")
                        .with_shader_file("shaders/composable_shaded_texture.frag")
                        .link()
                        .expect("Foliage shader had errors. See stdout.");
                    Rc::new(shader)
                }),
            }
        )
    }
}
