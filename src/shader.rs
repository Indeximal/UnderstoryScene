//! Shader abstraction from Gloom-rs.
//!
//! Can load and compile a shader from file.

use gl::types::GLuint;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::{ffi::CString, path::Path};
use std::{ptr, str};

pub struct Shader {
    program_id: GLuint,

    /// Mark the vao as !Send and !Sync, since OpenGL is not thread safe
    _marker: PhantomData<*const ()>,
}

pub struct ShaderBuilder {
    program_id: GLuint,
    shaders: Vec<GLuint>,

    /// Mark the vao as !Send and !Sync, since OpenGL is not thread safe
    _marker: PhantomData<*const ()>,
}

#[allow(dead_code)]
pub enum ShaderType {
    Vertex,
    Fragment,
    TessellationControl,
    TessellationEvaluation,
    Geometry,
}

impl Shader {
    /// Make sure the shader is active before calling this
    pub fn get_uniform_location(&self, name: &str) -> i32 {
        let name_cstr = CString::new(name).expect("CString::new failed");
        let id = unsafe { gl::GetUniformLocation(self.program_id, name_cstr.as_ptr()) };
        if id == -1 {
            panic!("get_uniform_location: Uniform `{}` not found.", name);
        }
        id
    }

    pub fn activate(&self) {
        unsafe { gl::UseProgram(self.program_id) };
    }
}

impl Into<gl::types::GLenum> for ShaderType {
    fn into(self) -> gl::types::GLenum {
        match self {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
            ShaderType::TessellationControl => gl::TESS_CONTROL_SHADER,
            ShaderType::TessellationEvaluation => gl::TESS_EVALUATION_SHADER,
            ShaderType::Geometry => gl::GEOMETRY_SHADER,
        }
    }
}

impl ShaderType {
    fn from_ext(ext: &std::ffi::OsStr) -> Result<ShaderType, String> {
        match ext.to_str().expect("Failed to read extension") {
            "vert" => Ok(ShaderType::Vertex),
            "frag" => Ok(ShaderType::Fragment),
            "tcs" => Ok(ShaderType::TessellationControl),
            "tes" => Ok(ShaderType::TessellationEvaluation),
            "geom" => Ok(ShaderType::Geometry),
            e => Err(e.to_string()),
        }
    }
}

impl ShaderBuilder {
    pub fn new() -> ShaderBuilder {
        ShaderBuilder {
            // Works if OpenGL has been properly set up.
            program_id: unsafe { gl::CreateProgram() },
            shaders: vec![],
            _marker: PhantomData,
        }
    }

    pub fn with_shader_file(self, shader_path: &str) -> ShaderBuilder {
        let path = Path::new(shader_path);
        if let Some(extension) = path.extension() {
            let shader_type =
                ShaderType::from_ext(extension).expect("Failed to parse file extension.");
            let shader_src = std::fs::read_to_string(path)
                .expect(&format!("Failed to read shader source `{}`", shader_path));
            self.with_shader(&shader_src, shader_type)
                .expect(&format!("Failed to compile shader `{}`", shader_path))
        } else {
            panic!(
                "Failed to read extension of file with path: {}",
                shader_path
            );
        }
    }

    fn with_shader(
        mut self,
        shader_src: &str,
        shader_type: ShaderType,
    ) -> Result<ShaderBuilder, ()> {
        let shader = unsafe {
            let shader = gl::CreateShader(shader_type.into());
            let c_str_shader = CString::new(shader_src.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str_shader.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            if !self.check_shader_errors(shader) {
                return Err(());
            }
            shader
        };

        self.shaders.push(shader);

        Ok(self)
    }

    unsafe fn check_shader_errors(&self, shader_id: u32) -> bool {
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetShaderInfoLog(
                shader_id,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!("ERROR:: Shader Compilation Failed!");
            println!(
                "{}",
                CStr::from_bytes_until_nul(&info_log)
                    .expect("Shader error was too long")
                    .to_string_lossy()
            );
            return false;
        }
        true
    }

    unsafe fn check_linker_errors(&self) -> bool {
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetProgramiv(self.program_id, gl::LINK_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetProgramInfoLog(
                self.program_id,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!(
                "ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}",
                String::from_utf8_lossy(&info_log)
            );
            return false;
        }
        true
    }

    #[must_use = "The shader program is useless if not stored in a variable."]
    pub fn link(self) -> Result<Shader, ()> {
        unsafe {
            for &shader in &self.shaders {
                gl::AttachShader(self.program_id, shader);
            }
            gl::LinkProgram(self.program_id);

            if !self.check_linker_errors() {
                return Err(());
            }

            for &shader in &self.shaders {
                gl::DeleteShader(shader);
            }
        }

        Ok(Shader {
            program_id: self.program_id,
            _marker: PhantomData,
        })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program_id);
        }
    }
}
// Shouldn't implement Drop for ShaderBuilder, as otherwise the
// construction of Shader causes the deletion of the program...
// Might have taken me 30' to figure out.
