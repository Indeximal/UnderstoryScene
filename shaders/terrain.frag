#version 410 core

in vec2 v_uv;
in vec3 v_pos;

uniform sampler2D terrain_albedo;

out vec4 color;

// 2d 45 degree rotation mat
const mat2 rotation45 = mat2(0.707, -0.707, 0.707, 0.707);

void main() {
    vec4 color1 = texture(terrain_albedo, 1.5 * v_pos.xy + vec2(0.5), -1.5);
    vec4 color2 = texture(terrain_albedo, 1.6 * rotation45 * v_pos.xy, -1.5);

    // Cheap transparency
    color = (color1 + color2) / 2.;
    if (color.a < 0.6) {
        discard;
    } else {
        color.a = 1.0;
    }

    if (v_pos.x < 0.0) {
        color *= clamp(0.1 + 2.5 * v_pos.z, 0., 1.);
    } else {
    }
}
