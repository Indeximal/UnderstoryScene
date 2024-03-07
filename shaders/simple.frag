#version 410 core

in vec2 v_uv;

uniform sampler2D noise_texture;

out vec4 color;

void main() {
    vec4 noise = texture(noise_texture, v_uv);
    color = vec4(noise.rrr, 1.0);
}
