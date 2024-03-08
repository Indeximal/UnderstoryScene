#version 410 core

in vec2 v_uv;
in vec3 v_pos;

out vec4 color;

void main() {
    color = vec4(0.5 + v_pos.zzz, 1.0);
}
