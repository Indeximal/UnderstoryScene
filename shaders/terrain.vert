#version 410 core

layout(location = 0) in vec3 position;
layout(location = 4) in vec2 uv;

out vec2 v_uv;
out vec3 v_pos;

uniform sampler2D displacement_map;
uniform mat4 view_proj;
uniform mat4 model_mat;

void main() {
    float z = texture(displacement_map, uv).r * 0.1;
    vec3 model_pos = position + vec3(0.0, 0.0, 1.0) * z;
    vec4 world_pos = model_mat * vec4(model_pos, 1.0);
    v_pos = world_pos.xyz;
    gl_Position = view_proj * world_pos;
    v_uv = uv;
}
