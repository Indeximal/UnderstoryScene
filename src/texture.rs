use std::marker::PhantomData;

use gl::types::GLuint;
use image::GenericImageView;

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
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::MIRRORED_REPEAT as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::MIRRORED_REPEAT as i32,
            );
        }
        get_gl_errors().expect("Failed to create texture");
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn from_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use image::io::Reader as ImageReader;

        let img = ImageReader::open(path)?.decode()?;
        let (width, height) = img.dimensions();

        img.as_rgba8()
            .map(|img| {
                Ok(Self::new::<u8, format::RGBA>(
                    width,
                    height,
                    img.as_raw().as_slice(),
                ))
            })
            .or_else(|| {
                img.as_rgb8().map(|img| {
                    Ok(Self::new::<u8, format::RGB>(
                        width,
                        height,
                        img.as_raw().as_slice(),
                    ))
                })
            })
            .or_else(|| {
                img.as_luma8().map(|img| {
                    Ok(Self::new::<u8, format::GrayScale>(
                        width,
                        height,
                        img.as_raw().as_slice(),
                    ))
                })
            })
            .unwrap_or_else(|| Err("Unsupported image format".into()))
    }

    pub fn enable_mipmap(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
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
}

/// Weird magic I had fun creating, to leverage the Rust type system to create
/// overloading of the `Texture::new` function.
pub mod format {
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

    pub struct RGB;
    impl Sealed for RGB {}
    impl TextureFormat for RGB {
        fn num_components() -> usize {
            3
        }
        fn to_glenum() -> gl::types::GLenum {
            gl::RGB
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
