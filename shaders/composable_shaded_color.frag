#version 410 core

in vec3 v_position;
in vec3 v_normal;

uniform vec3 color;

out vec4 out_color;

const vec3 light_up = normalize(vec3(0.5, 0.5, 1.0));

void main() {
    vec3 normal = normalize(v_normal);
    float diff = clamp(dot(normal, light_up), 0.0, 1.0);
    out_color = vec4(color * (0.5 * diff + 0.5), 1.0);
}
