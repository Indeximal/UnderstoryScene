#version 410 core

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;

out vec3 v_color;

uniform mat4 view_proj;

void main() {
    vec4 homog = vec4(position, 0.0, 1.0);
    gl_Position = view_proj * homog;
    v_color = color;
}
