#version 450

layout(binding = 0) uniform UniformBufferObject {
    uint transform_length;
    mat4 view_proj;
} ubo;

layout(binding = 2) buffer Transforms {
    mat4 transforms[];
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coords_in;
layout(location = 3) in vec3 camera_position;
layout(location = 4) in mat4 model;
layout(location = 8) in vec4 weights;
layout(location = 9) in uvec4 joints;

layout(location = 0) out vec2 tex_coords;
layout(location = 1) out float shade;

void main() {
    mat4 animated_transform = transforms[joints.x] * weights.x + transforms[joints.y] * weights.y + transforms[joints.z] * weights.z + transforms[joints.w] * weights.w;
    vec4 world_position = model * animated_transform * vec4(position, 1.0);
    gl_Position = ubo.view_proj * world_position;
    shade = max(dot(normalize(camera_position), normal), 0.1);
    tex_coords = tex_coords_in;
}