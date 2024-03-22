use noise::NoiseFn;

use crate::mesh::{ElementMeshVAO, Mesh};
use crate::scene::SCENE_SIZE;
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
    pub ground_tex: Rc<Texture>,
    pub rock_tex: Rc<Texture>,
    pub bush_tex: Rc<Texture>,
    pub bush_side_tex: Rc<Texture>,
    pub shrub_tex: Rc<Texture>,
    pub transparent_tex: Rc<Texture>,
    pub bark_tex: Rc<Texture>,

    /// A map of the terrain, created by an artist, which controls:
    ///
    /// - The base height in the R channel
    /// - The base bushyness in the G channel
    /// - The tree locations in the B channel
    pub base_map: Rc<image::RgbaImage>,

    // These could technically also share the VAO, but since the instance data
    // is dynamic, this would be weird.
    pub shrub_model: Rc<Mesh>,
    pub bush1_model: Rc<Mesh>,
    pub bush2_model: Rc<Mesh>,
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
                // Compile shaders
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

                // Load Textures
                moss_tex: time!("moss texture", {
                    let tex = Texture::from_file("textures/moss1.jpeg")
                        .expect("Loading moss texture failed");
                    tex.enable_mipmap();
                    Rc::new(tex)
                }),
                ground_tex: time!("ground texture", {
                    let tex = Texture::from_file("textures/ground1.jpeg")
                        .expect("Loading ground texture failed");
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
                bush_side_tex: time!("bush side texture", {
                    let tex = Texture::from_file("textures/bush_masked2.png")
                        .expect("Loading bush side texture failed");
                    tex.enable_mipmap();
                    Rc::new(tex)
                }),
                shrub_tex: time!("shrub texture", {
                    let tex = Texture::new::<f32, crate::texture::format::RGBA>(
                        1,
                        1,
                        &[71. / 255., 49. / 255., 68. / 255., 1.0],
                    );
                    Rc::new(tex)
                }),
                transparent_tex: time!("transparent texture", {
                    let tex = Texture::new::<f32, crate::texture::format::RGBA>(
                        1,
                        1,
                        &[85. / 255., 92. / 255., 42. / 255., 0.0],
                    );
                    Rc::new(tex)
                }),
                bark_tex: time!("bark texture", {
                    let tex = Texture::from_file("textures/bark1.jpeg")
                        .expect("Loading bark texture failed");
                    tex.enable_mipmap();
                    Rc::new(tex)
                }),

                // Load base map
                base_map: time!("base map", {
                    let img = image::open("textures/map.png")
                        .expect("Loading base map failed")
                        .into_rgba8();
                    Rc::new(img)
                }),

                // Load obj models
                shrub_model: time!("shrub model", {
                    let model = Mesh::load("models/shrub2.obj");
                    Rc::new(model)
                }),
                bush1_model: time!("bush model", {
                    let model = Mesh::load("models/bush1.obj");
                    Rc::new(model)
                }),
                bush2_model: time!("bush model", {
                    let model = Mesh::load("models/bush2.obj");
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
            }
        )
    }
}

pub struct ImageNoiseFnWrapper<const CHANNEL: usize> {
    image: Rc<image::RgbaImage>,
}

impl ImageNoiseFnWrapper<0> {
    pub fn new_red(image: Rc<image::RgbaImage>) -> Self {
        ImageNoiseFnWrapper { image }
    }
}

impl ImageNoiseFnWrapper<1> {
    pub fn new_green(image: Rc<image::RgbaImage>) -> Self {
        ImageNoiseFnWrapper { image }
    }
}

impl ImageNoiseFnWrapper<2> {
    pub fn new_blue(image: Rc<image::RgbaImage>) -> Self {
        ImageNoiseFnWrapper { image }
    }
}

impl<const CHANNEL: usize> NoiseFn<f64, 2> for ImageNoiseFnWrapper<CHANNEL> {
    fn get(&self, point: [f64; 2]) -> f64 {
        let x = (point[0] / SCENE_SIZE as f64 * self.image.width() as f64) as u32;
        let y = (point[1] / SCENE_SIZE as f64 * self.image.height() as f64) as u32;
        // clamp x & y
        let x = x.clamp(0, self.image.width() - 1);
        let y = y.clamp(0, self.image.height() - 1);
        // Intentionally switching x & y to line up with my coordinate system
        let pixel = self.image.get_pixel(y, x);
        // Red channel is height
        let value: f64 = pixel[CHANNEL].into();
        value / 255.0
    }
}
