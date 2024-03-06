#version 410 core

layout(location = 0) in vec3 position;
layout(location = 4) in vec2 uv;

out vec2 v_uv;

uniform mat4 view_proj;

void main() {
    gl_Position = view_proj * vec4(position, 1.0);
    v_uv = uv;
}
