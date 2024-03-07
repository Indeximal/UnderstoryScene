pub fn clear_gl_errors() {
    loop {
        let error = unsafe { gl::GetError() };
        if error == gl::NO_ERROR {
            break;
        }
        // Ignore the error
    }
}

pub fn get_gl_errors() -> Result<(), Vec<&'static str>> {
    let mut errors = Vec::new();

    loop {
        let error = unsafe { gl::GetError() };
        let error_str = match error {
            gl::NO_ERROR => break,
            gl::INVALID_ENUM => "GL_INVALID_ENUM",
            gl::INVALID_VALUE => "GL_INVALID_VALUE",
            gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
            gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
            gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
            gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
            _ => "Unknown error",
        };

        errors.push(error_str);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
