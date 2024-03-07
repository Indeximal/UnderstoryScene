use std::marker::PhantomData;

use gl::types::GLuint;
use rand::Rng;
use rand::SeedableRng;

use crate::error::clear_gl_errors;
use crate::error::get_gl_errors;

pub struct Texture {
    id: GLuint,
    /// Mark the texture as !Send and !Sync, since OpenGL is not thread safe
    _marker: PhantomData<*const ()>,
}

impl Texture {
    pub fn new_rgb(width: i32, height: i32, data: &[f32]) -> Self {
        assert!(
            data.len() == (width * height * 3) as usize,
            "New texture data length does not match width and height"
        );

        let mut id = 0;
        clear_gl_errors();
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            // SAFETY: data is a valid pointer to a valid slice of f32 and
            // has the correct length (asserted above).
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32, // The target format
                width,
                height,
                0,
                gl::RGB,
                gl::FLOAT,
                data.as_ptr() as *const _,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        get_gl_errors().expect("Failed to create texture");
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn activate(&self, texture_unit: u32) {
        // Might not be restrictive enought, based on implementation
        assert!(texture_unit < 32, "Texture unit out of range");
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn noise(width: i32, height: i32, seed: u64) -> Self {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
        let mut data = vec![0.0; (width * height * 3) as usize];
        // Each value between 0. and 1.
        rng.fill(&mut data[..]);

        Self::new_rgb(width, height, &data)
    }
}
