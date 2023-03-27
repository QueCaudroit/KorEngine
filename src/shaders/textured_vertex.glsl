#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view_proj;
    vec3 camera_position;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coords_in;

layout(location = 0) out vec2 tex_coords;
layout(location = 1) out float shade;

void main() {
    vec4 world_position = ubo.model * vec4(position, 1.0);
    gl_Position = ubo.view_proj * world_position;
    shade = max(dot(normalize(ubo.camera_position), normal), 0.1);
}