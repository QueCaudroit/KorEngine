#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
    vec3 light_position;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coords_in;
layout(location = 3) in mat4 model;

layout(location = 0) out vec2 tex_coords;
layout(location = 1) out float shade;

const float lambertian_diffuse = 0.31830988618; // 1/pi

void main() {
    vec4 world_position = model * vec4(position, 1.0);
    vec3 world_normal = normalize((model * vec4(normal, 0.0)).xyz);
    gl_Position = ubo.view_proj * world_position;
    vec3 light_direction = normalize(ubo.light_position - world_position.xyz);
    shade = (lambertian_diffuse * max(dot(light_direction, world_normal), 0.1));
    tex_coords = tex_coords_in;
}