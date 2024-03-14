#version 410 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

out vec3 v_position;
out vec3 v_normal;

uniform mat4 view_proj;
uniform mat4 model_mat;
uniform mat3 normal_mat;

void main() {
    vec4 world_pos = model_mat * vec4(position, 1.0);
    v_position = world_pos.xyz;
    v_normal = normal_mat * normal;
    gl_Position = view_proj * world_pos;
}
