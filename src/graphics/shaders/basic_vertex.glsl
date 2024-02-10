#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
    vec3 light_position;
    vec3 camera_position;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in mat4 model;

layout(location = 0) out vec3 light_direction;
layout(location = 1) out vec3 camera_direction;
layout(location = 2) out vec3 normal_direction;

void main() {
    vec4 world_position = model * vec4(position, 1.0);
    gl_Position = ubo.view_proj * world_position;
    light_direction = normalize(ubo.light_position - world_position.xyz);
    camera_direction = normalize(ubo.camera_position - world_position.xyz);
    normal_direction = normalize((model * vec4(normal, 0.0)).xyz);
}