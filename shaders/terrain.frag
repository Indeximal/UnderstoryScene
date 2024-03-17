#version 410 core

in vec3 v_pos;
in vec3 v_normal;

uniform sampler2D terrain_albedo;

out vec4 color;

const vec3 light_up = normalize(vec3(0.5, 0.5, 1.0));

// 2d 45 degree rotation mat
const mat2 rotation45 = mat2(0.707, -0.707, 0.707, 0.707);

void main() {
    vec4 color1 = texture(terrain_albedo, 1.5 * v_pos.xy + vec2(0.5), -1.5);
    vec4 color2 = texture(terrain_albedo, 1.6 * rotation45 * v_pos.xy, -1.5);
    vec3 normal = normalize(v_normal);

    // Cheap transparency
    color = (color1 + color2) / 2.;
    if (color.a < 0.6) {
        discard;
    }

    // Shading based on normal (half ambient, half diffuse from above)
    color *= clamp(dot(light_up, normal) * 0.5 + 0.5, 0., 1.);
    // Shading based on height
    color *= clamp(0.3 + 2.0 * v_pos.z, 0., 1.);

    // Test different shadings on right/left screen half
    if (gl_FragCoord.x > 400) {
        
    } else {
        
    }

    color.a = 1.0;
}
