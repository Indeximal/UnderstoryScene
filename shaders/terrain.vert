#version 410 core

layout(location = 0) in vec3 position;
layout(location = 4) in vec2 uv;

out vec3 v_pos;
out vec3 v_normal;
out float v_variant;

uniform sampler2D displacement_map;
uniform sampler2D variant_map;
uniform mat4 view_proj;
// This will only transform the base shape, not the displacement
uniform mat4 model_mat;
// A matrix that will right multiply a world coordinate into a uv coordinate
uniform mat3 world_to_uv;

uniform vec3 depression;

// Min feature size is less than 5cm
const float dx = 0.05;

float sample_displacement(vec2 wcoord) {
    vec3 w_uv = world_to_uv * vec3(wcoord, 1.0);
    return texture(displacement_map, w_uv.xy).r;
}

void main() {
    vec4 world_pos = model_mat * vec4(position, 1.0);

    float z = sample_displacement(world_pos.xy);
    float zu = sample_displacement(world_pos.xy + vec2(dx, 0.0));
    float zv = sample_displacement(world_pos.xy + vec2(0.0, dx));
    float du = (z - zu) / dx;
    float dv = (z - zv) / dx;
    v_normal = normalize(vec3(du, dv, 1.0));

    v_variant = texture(variant_map, (world_to_uv * vec3(world_pos.xy, 1.0)).xy).r;

    // FIXME: this changes the normal
    vec2 depress_vec = depression.xy - world_pos.xy;
    float depress = exp(-dot(depress_vec, depress_vec)) * depression.z;

    // Doing this after the model matrix means that the direction is hardcoded
    vec4 displaced_pos = world_pos + (z - depress) * vec4(0.0, 0.0, 1.0, 0.0);
    v_pos = displaced_pos.xyz;
    gl_Position = view_proj * displaced_pos;
}
