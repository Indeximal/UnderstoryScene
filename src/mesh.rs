use std::marker::PhantomData;

use gl::types::GLuint;
use nalgebra_glm as glm;
use tobj;

use crate::error::{clear_gl_errors, get_gl_errors};

pub const POSITION_ATTRIB_PTR: u32 = 0;
pub const NORMAL_ATTRIB_PTR: u32 = 1;
#[allow(dead_code)]
pub const TANGENT_ATTRIB_PTR: u32 = 2;
#[allow(dead_code)]
pub const BITANGENT_ATTRIB_PTR: u32 = 3;
pub const UV_ATTRIB_PTR: u32 = 4;

pub const MODEL_MAT_ATTRIB_PTR_1: u32 = 8;
pub const MODEL_MAT_ATTRIB_PTR_2: u32 = 9;
pub const MODEL_MAT_ATTRIB_PTR_3: u32 = 10;
pub const MODEL_MAT_ATTRIB_PTR_4: u32 = 11;
pub const MODEL_NORMAL_ATTRIB_PTR_1: u32 = 12;
pub const MODEL_NORMAL_ATTRIB_PTR_2: u32 = 13;
pub const MODEL_NORMAL_ATTRIB_PTR_3: u32 = 14;

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

/// Some data stored on the GPU.
pub struct VAO {
    id: GLuint,
    vbos: Vec<GLuint>,
    /// Mark the vao as !Send and !Sync, since OpenGL is not thread safe
    _marker: PhantomData<*const ()>,
}

pub struct ElementMeshVAO {
    index_count: usize,
    vao: VAO,
}

pub struct InstancedMeshesVAO {
    index_count_per_instance: usize,
    instance_count: usize,
    vao: VAO,
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
        let (models, _materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .expect("Failed to load model");

        if models.len() > 1 || models.len() == 0 {
            panic!("Please use a model with a single mesh!")
            // You could try merging the vertices and indices
            // of the separate meshes into a single mesh.
            // I'll leave that as an optional exercise. ;)
        }

        let terrain = models[0].to_owned();
        println!(
            "Loaded {} with {} vertices and {} triangles.",
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

    /// Generates a mesh of `divisions` by `divisions` quads on the XY plane.
    ///
    /// Is 2x2 large and spans 0 to 1 in UV space.
    pub fn quad_mesh(num_quads: u32) -> Self {
        let mut positions = vec![];
        let mut normals = vec![];
        let mut uvs = vec![];
        let mut indices = vec![];

        for y in 0..=num_quads {
            for x in 0..=num_quads {
                let u = x as f32 / num_quads as f32;
                let v = y as f32 / num_quads as f32;
                let px = u * 2.0 - 1.0;
                let py = v * 2.0 - 1.0;

                // Generate the vertex
                positions.extend_from_slice(&[px, py, 0.0]);
                normals.extend_from_slice(&[0.0, 0.0, 1.0]);
                uvs.extend_from_slice(&[u, v]);

                // if not on the edge, generate two triangles aswell
                if x < num_quads && y < num_quads {
                    let this_i = x + y * (num_quads + 1);
                    indices.extend_from_slice(&[
                        this_i,
                        this_i + 1,
                        this_i + (num_quads + 1),
                        this_i + 1,
                        this_i + (num_quads + 1) + 1,
                        this_i + (num_quads + 1),
                    ]);
                }
            }
        }

        Mesh {
            positions,
            normals,
            uvs,
            indices,
        }
    }
}

impl ElementMeshVAO {
    /// Loads the mesh data onto the GPU.
    ///
    /// The VAO's attributes are configured according to the constants in this module.
    pub fn new_from_mesh(mesh: &Mesh) -> Self {
        mesh.check_consitency()
            .expect("Refusing to create VAO from inconsistent mesh.");

        clear_gl_errors();

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

        get_gl_errors().expect("Generating the mesh buffer run into errors");

        ElementMeshVAO {
            index_count: mesh.indices.len(),
            vao: VAO {
                id: vao_id,
                vbos: vec![position_vbo, normal_vbo, uvs_vbo, index_vbo],
                _marker: PhantomData,
            },
        }
    }

    pub fn render(&self) {
        // SAFETY: VAO id was created in the constructor, errors were checked,
        // and the object is on the same thread.
        unsafe {
            gl::BindVertexArray(self.vao.id);
            gl::DrawElements(
                gl::TRIANGLES,
                self.index_count as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }
}

impl InstancedMeshesVAO {
    pub fn from_existing_with_models(mut single_vao: ElementMeshVAO, models: &[glm::Mat4]) -> Self {
        // Matrices as attributes need a pointer for each column, ie 3 or 4 attribs.
        unsafe fn set_vertex_attrib_pointer(attrib_ptr: u32, offset: usize, components: usize) {
            let vec_size = components * std::mem::size_of::<f32>();
            gl::EnableVertexAttribArray(attrib_ptr);
            gl::VertexAttribPointer(
                attrib_ptr,
                components as i32,
                gl::FLOAT,
                gl::FALSE,
                (components * vec_size) as gl::types::GLint,
                (offset * vec_size) as *const _,
            );
            // Set as instance attribute
            gl::VertexAttribDivisor(attrib_ptr, 1);
        }

        // SAFETY: glm::Mat4 are represented as 16 densely packed floats
        let model_mats_vbo = unsafe {
            gl::BindVertexArray(single_vao.vao.id);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (models.len() * std::mem::size_of::<glm::Mat4>()) as gl::types::GLsizeiptr,
                models.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            set_vertex_attrib_pointer(MODEL_MAT_ATTRIB_PTR_1, 0, 4);
            set_vertex_attrib_pointer(MODEL_MAT_ATTRIB_PTR_2, 1, 4);
            set_vertex_attrib_pointer(MODEL_MAT_ATTRIB_PTR_3, 2, 4);
            set_vertex_attrib_pointer(MODEL_MAT_ATTRIB_PTR_4, 3, 4);

            vbo
        };

        // Generate and load normal transformation matrices
        // (Currently broken)
        let mut normal_mats: Vec<glm::Mat3> = Vec::with_capacity(models.len());
        for model_mat in models {
            let model_normal: glm::Mat3 =
                glm::mat4_to_mat3(&glm::transpose(&glm::inverse(&model_mat)));
            normal_mats.push(model_normal);
        }

        let normal_mats_vbo = unsafe {
            gl::BindVertexArray(single_vao.vao.id);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (normal_mats.len() * std::mem::size_of::<glm::Mat3>()) as gl::types::GLsizeiptr,
                normal_mats.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            set_vertex_attrib_pointer(MODEL_NORMAL_ATTRIB_PTR_1, 0, 3);
            set_vertex_attrib_pointer(MODEL_NORMAL_ATTRIB_PTR_2, 1, 3);
            set_vertex_attrib_pointer(MODEL_NORMAL_ATTRIB_PTR_3, 2, 3);

            vbo
        };
        single_vao.vao.vbos.push(model_mats_vbo);
        single_vao.vao.vbos.push(normal_mats_vbo);

        Self {
            index_count_per_instance: single_vao.index_count,
            instance_count: models.len(),
            vao: single_vao.vao,
        }
    }

    pub fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vao.id);
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                self.index_count_per_instance as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
                self.instance_count as i32,
            );
        }
    }
}

impl Drop for VAO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(self.vbos.len() as i32, self.vbos.as_ptr());
            gl::DeleteVertexArrays(1, &self.id);
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
