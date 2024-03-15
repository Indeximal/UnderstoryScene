#version 410 core

layout(location = 0) in vec3 position;
layout(location = 4) in vec2 uv;

out vec3 v_pos;
out vec3 v_normal;

uniform sampler2D displacement_map;
uniform mat4 view_proj;
uniform mat4 model_mat;

// Max feature size is less than 1%
const float dx = 0.01;

float sample_displacement(vec2 uv) {
    return texture(displacement_map, uv).r;
}

void main() {
    float z = sample_displacement(uv);
    float zu = sample_displacement(uv + vec2(dx, 0.0));
    float zv = sample_displacement(uv + vec2(0.0, dx));
    float du = (zu - z) / dx;
    float dv = (zv - z) / dx;

    // Assume UV is aligned to xy plane, but scaled down.
    // Not actually accurate, I think should be 6.0, but artistic freedom.
    v_normal = normalize(vec3(du / 3.0, dv / 3.0, 1.0));

    vec3 model_pos = position + z * vec3(0.0, 0.0, 1.0);
    vec4 world_pos = model_mat * vec4(model_pos, 1.0);
    gl_Position = view_proj * world_pos;
    v_pos = world_pos.xyz;
}
