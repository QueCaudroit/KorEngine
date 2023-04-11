#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
    vec4 color;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 camera_position;
layout(location = 3) in mat4 model;

layout(location = 0) out vec4 fragColor;

void main() {
    vec4 world_position = model * vec4(position, 1.0);
    gl_Position = ubo.view_proj * world_position;
    vec3 color_temp = ubo.color.rgb * max(dot(normalize(camera_position), normal), 0.1);
    fragColor =  vec4(color_temp, ubo.color.a);
}