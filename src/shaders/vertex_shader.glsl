#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view_proj;
    vec4 color;
    vec3 camera_position;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec4 fragColor;

void main() {
    vec4 world_position = ubo.model * vec4(position, 1.0);
    gl_Position = ubo.view_proj * world_position;
    vec3 color = vec3(0.1, 0.1, 0.1) + vec3(0.7, 0.7, 0.7) * dot(normalize(ubo.camera_position), normal);
    fragColor =  vec4(max(color, vec3(0.0, 0.0, 0.0)), 1.0);
}