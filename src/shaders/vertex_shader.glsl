#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view_proj;
    vec4 color;
    vec3 camera_position;
} ubo;

layout(location = 0) in vec3 position;
//layout(location = 1) in vec3 normal;

layout(location = 0) out vec4 fragColor;

void main() {
    vec4 world_position = ubo.model * vec4(position, 1.0);
    gl_Position = ubo.view_proj * world_position;
    fragColor =  vec4(0.8, 0.8, 0.8, 1.0);
}