use gl::types::GLuint;
use tobj;

pub const POSITION_ATTRIB_PTR: u32 = 0;
pub const NORMAL_ATTRIB_PTR: u32 = 1;
pub const TANGENT_ATTRIB_PTR: u32 = 2;
pub const BITANGENT_ATTRIB_PTR: u32 = 3;
pub const UV_ATTRIB_PTR: u32 = 4;

pub struct Mesh {
    /// Cyclic X, Y, Z components
    pub positions: Vec<f32>,
    /// Cyclic X, Y, Z components
    pub normals: Vec<f32>,
    /// Cyclic U, V components
    pub uvs: Vec<f32>,

    /// Cyclic first, second, thrid vertex index.
    pub indices: Vec<u32>,
}

/// A mesh stored on the GPU.
pub struct VAO {
    index_count: usize,
    vao: GLuint,
    vbos: Vec<GLuint>,
}

impl Mesh {
    pub fn from(mesh: tobj::Mesh) -> Self {
        Mesh {
            positions: mesh.positions,
            normals: mesh.normals,
            uvs: mesh.texcoords,

            indices: mesh.indices,
        }
    }

    pub fn load(path: &str) -> Self {
        println!("Loading model...");
        let before = std::time::Instant::now();
        let (models, _materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Failed to load model");
        let after = std::time::Instant::now();
        println!(
            "Done in {:.3}ms.",
            after.duration_since(before).as_micros() as f32 / 1e3
        );

        if models.len() > 1 || models.len() == 0 {
            panic!("Please use a model with a single mesh!")
            // You could try merging the vertices and indices
            // of the separate meshes into a single mesh.
            // I'll leave that as an optional exercise. ;)
        }

        let terrain = models[0].to_owned();
        println!(
            "Loaded {} with {} points and {} triangles.",
            terrain.name,
            terrain.mesh.positions.len() / 3,
            terrain.mesh.indices.len() / 3,
        );

        Mesh::from(terrain.mesh)
    }

    /// Checks that the mesh has same size positions, normals and uvs as well as
    /// proper stride and indices.
    pub fn check_consitency(&self) -> Result<(), &'static str> {
        if self.positions.len() % 3 != 0 {
            return Err("Positions length is not a multiple of 3 (X, Y, Z).");
        }
        if self.normals.len() % 3 != 0 {
            return Err("Normals length is not a multiple of 3 (dX, dY, dZ).");
        }
        if self.normals.len() != 0 && self.normals.len() != self.positions.len() {
            return Err("Not as many normals as vertices.");
        }
        if self.uvs.len() % 2 != 0 {
            return Err("UVs length is not a multiple of 2 (U, V).");
        }
        if self.uvs.len() != 0 && self.uvs.len() != self.positions.len() / 3 * 2 {
            return Err("Not as many UVs as vertices.");
        }
        if self.indices.len() % 3 != 0 {
            return Err("Indices length is not a multiple of 3 (Vertex 1, 2, 3).");
        }
        if *self.indices.iter().max().ok_or("No indices.")? as usize >= (self.positions.len() / 3) {
            return Err("Indices point to non-existent vertices.");
        }
        Ok(())
    }

    /// A simple 2 by 2 quad on the XY plane.
    pub fn quad() -> Self {
        Mesh {
            positions: vec![
                -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0,
            ],
            normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            uvs: vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }
}

impl VAO {
    /// Loads the mesh data onto the GPU.
    ///
    /// The VAO's attributes are configured according to the constants in this module.
    pub fn new_from_mesh(mesh: &Mesh) -> Self {
        mesh.check_consitency()
            .expect("Refusing to create VAO from inconsistent mesh.");

        let vao_id = unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            vao
        };

        let position_vbo = load_attribute(&mesh.positions, POSITION_ATTRIB_PTR, 3);
        let normal_vbo = load_attribute(&mesh.normals, NORMAL_ATTRIB_PTR, 3);
        let uvs_vbo = load_attribute(&mesh.uvs, UV_ATTRIB_PTR, 2);

        let index_vbo = unsafe {
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mesh.indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                mesh.indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            vbo
        };

        VAO {
            vao: vao_id,
            index_count: mesh.indices.len(),
            vbos: vec![position_vbo, normal_vbo, uvs_vbo, index_vbo],
        }
    }

    pub fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }
}

impl Drop for VAO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(self.vbos.len() as i32, self.vbos.as_ptr());
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

fn load_attribute(data: &[f32], attrib_ptr: u32, components: usize) -> GLuint {
    unsafe {
        let mut vbo = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            data.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            attrib_ptr,
            components as gl::types::GLint,
            gl::FLOAT,
            gl::FALSE,
            (components * std::mem::size_of::<f32>()) as gl::types::GLint,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(attrib_ptr);
        vbo
    }
}
