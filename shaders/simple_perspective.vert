#version 410 core

layout(location = 0) in vec3 position;

out vec3 v_position;

uniform mat4 view_proj;
uniform mat4 model_mat;

void main() {
    vec4 world_pos = model_mat * vec4(position, 1.0);
    v_position = world_pos.xyz;
    gl_Position = view_proj * world_pos;
}
