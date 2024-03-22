#version 410 core

in vec3 v_pos;
in vec3 v_normal;

uniform sampler2D terrain_albedo_xy;
uniform sampler2D terrain_albedo_xz;
uniform sampler2D terrain_albedo_yz;

out vec4 color;

const vec3 light_up = normalize(vec3(0.5, 0.5, 1.0));

// 2d 45 degree rotation mat
const mat2 rotation45 = mat2(0.707, -0.707, 0.707, 0.707);

void main() {
    vec3 normal = normalize(v_normal);
    vec3 weights = pow(abs(normal), vec3(8.0));
    weights /= dot(weights, vec3(1.0));

    vec4 color_xy = texture(terrain_albedo_xy, 1.6 * v_pos.xy, -1.5);
    vec4 color_xz = texture(terrain_albedo_xz, v_pos.xz, -1.5);
    vec4 color_yz = texture(terrain_albedo_yz, v_pos.yz, -1.5);
    color = color_xy * weights.z + color_xz * weights.y + color_yz * weights.x;

    // Cheap order independent transparency
    if (color.a < 0.6) {
        discard;
    }

    // Test different shadings on right/left screen half
    if (gl_FragCoord.x > 400) {

    } else {

    }

    // Shading based on normal (half ambient, half diffuse from above)
    color *= clamp(dot(light_up, normal) * 0.5 + 0.5, 0., 1.);
    // Shading based on height
    color *= clamp(0.3 + 2.0 * v_pos.z, 0., 1.);

    color.a = 1.0;
}
