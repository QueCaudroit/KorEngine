#version 450

layout(binding = 1) uniform sampler2D tex;

layout(location = 0) in vec2 tex_coords;
layout(location = 1) in float shade;

layout(location = 0) out vec4 f_color;

void main() {
    vec4 tex_color = texture(tex, tex_coords);
    f_color = vec4(tex_color.rgb * shade, tex_color.a);
}