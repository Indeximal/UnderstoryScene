use std::marker::PhantomData;

use gl::types::GLuint;
use noise::{Add, ScaleBias};
use noise::{NoiseFn, ScalePoint};

use crate::error::clear_gl_errors;
use crate::error::get_gl_errors;

pub struct Texture {
    id: GLuint,
    /// Mark the texture as !Send and !Sync, since OpenGL is not thread safe
    _marker: PhantomData<*const ()>,
}

impl Texture {
    /// Create a new texture from a slice of data.
    ///
    /// Specify the format using the generics, for example:
    /// ```no_run
    /// let data = vec![0.0, 0.0, 0.0, 1.0];
    /// let texture = Texture::new::<f32, RGBA>(1, 1, data.as_slice());
    /// ```
    pub fn new<T: format::TextureDataValue, F: format::TextureFormat>(
        width: u32,
        height: u32,
        data: &[T],
    ) -> Self {
        assert!(
            data.len() == (width * height) as usize * F::num_components(),
            "Texture data length does not match width, height and format"
        );

        let mut id = 0;
        clear_gl_errors();
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            // SAFETY: data is a valid pointer to a valid slice of T and
            // has the correct length (asserted above).
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                F::to_glenum() as i32, // The target format
                width as i32,
                height as i32,
                0,
                F::to_glenum(),
                T::to_glenum(),
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
        // Might not be restrictive enought, based on implementation ?
        assert!(texture_unit < 32, "Texture unit out of range");
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + texture_unit);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn noise(width: u32, height: u32, seed: u32) -> Self {
        let octave0 = ScaleBias::new(ScalePoint::new(noise::Value::new(seed + 1)).set_scale(2.))
            .set_scale(0.5)
            .set_bias(0.5);
        let octave1 = ScaleBias::new(ScalePoint::new(noise::Value::new(seed + 2)).set_scale(4.))
            .set_scale(0.25)
            .set_bias(0.25);
        let octave2 = ScaleBias::new(ScalePoint::new(noise::Value::new(seed + 3)).set_scale(8.))
            .set_scale(0.125)
            .set_bias(0.125);

        let noise = Add::new(Add::new(octave0, octave1), octave2);

        // Copied the internals of [`noise::utils::PlaneMapBuilder`], since I
        // want to get the vector directly.
        let mut result_map = vec![0.0f32; (width * height) as usize];

        let x_bounds = (-1.0, 1.0);
        let y_bounds = (-1.0, 1.0);
        let x_step = (x_bounds.1 - x_bounds.0) / width as f64;
        let y_step = (y_bounds.1 - y_bounds.0) / height as f64;

        for y in 0..height {
            for x in 0..width {
                let current_y = y_bounds.0 + y_step * y as f64;
                let current_x = x_bounds.0 + x_step * x as f64;

                result_map[(y * width + x) as usize] = noise.get([current_x, current_y]) as f32;
            }
        }

        Self::new::<f32, format::GrayScale>(width, height, result_map.as_slice())
    }
}

/// Weird magic I had fun creating, to leverage the Rust type system to create
/// overloading of the `Texture::new` function.
mod format {
    trait Sealed {}

    /// Use either [`f32`] or [`u8`].
    #[allow(private_bounds)]
    pub trait TextureDataValue: Sealed {
        fn to_glenum() -> gl::types::GLenum;
    }

    /// Use either [`GrayScale`] or [`RGBA`].
    #[allow(private_bounds)]
    pub trait TextureFormat: Sealed {
        fn num_components() -> usize;
        fn to_glenum() -> gl::types::GLenum;
    }

    pub struct GrayScale;
    impl Sealed for GrayScale {}
    impl TextureFormat for GrayScale {
        fn num_components() -> usize {
            1
        }
        fn to_glenum() -> gl::types::GLenum {
            gl::RED
        }
    }
    pub struct RGBA;
    impl Sealed for RGBA {}
    impl TextureFormat for RGBA {
        fn num_components() -> usize {
            4
        }
        fn to_glenum() -> gl::types::GLenum {
            gl::RGBA
        }
    }

    impl Sealed for f32 {}
    impl TextureDataValue for f32 {
        fn to_glenum() -> gl::types::GLenum {
            gl::FLOAT
        }
    }

    impl Sealed for u8 {}
    impl TextureDataValue for u8 {
        fn to_glenum() -> gl::types::GLenum {
            gl::UNSIGNED_BYTE
        }
    }
}
