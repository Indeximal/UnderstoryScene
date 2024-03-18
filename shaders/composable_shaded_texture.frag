#version 410 core

in vec3 v_position;
in vec3 v_normal;
in vec2 v_texcord;

uniform sampler2D albedo;

out vec4 out_color;

const vec3 light_up = normalize(vec3(0.5, 0.5, 1.0));

void main() {
    vec4 color = texture(albedo, v_texcord, -1.5);

    // Cheap order independent transparency
    if (color.a <= 0.5) {
        discard;
    }

    vec3 normal = normalize(v_normal);
    float diff = clamp(dot(normal, light_up), 0.0, 1.0);
    out_color = vec4(color.rgb * (0.5 * diff + 0.5), 1.0);
}
