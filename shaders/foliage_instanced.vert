#version 410 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 4) in vec2 texcord;
layout(location = 8) in mat4 model_mat;
layout(location = 12) in mat3 normal_mat;

out vec3 v_position;
out vec3 v_normal;
out vec2 v_texcord;

uniform mat4 view_proj;

void main() {
    vec4 world_pos = model_mat * vec4(position, 1.0);
    v_position = world_pos.xyz;
    v_normal = normal_mat * normal;
    v_texcord = texcord;
    gl_Position = view_proj * world_pos;
}
