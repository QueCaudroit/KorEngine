#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view_proj;
    vec3 light_position;
    vec3 camera_position;
    uint transform_length;
} ubo;

layout(binding = 2) buffer Transforms {
    mat4 transforms[];
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in mat4 model;
layout(location = 6) in vec4 weights;
layout(location = 7) in uvec4 joints;
layout(location = 8) in vec2 tex_coords_in;
layout(location = 9) in vec2 tex_metal_coords_in;
layout(location = 10) in vec2 tex_normal_coords_in;
layout(location = 11) in vec3 tangent;


layout(location = 0) out vec3 light_direction;
layout(location = 1) out vec3 camera_direction;
layout(location = 2) out vec3 normal_direction;
layout(location = 3) out vec2 tex_coords;
layout(location = 4) out vec2 tex_metal_coords;
layout(location = 5) out vec2 tex_normal_coords;
layout(location = 6) out vec3 tangent_direction;

const float lambertian_diffuse = 0.31830988618; // 1/pi

void main() {
    mat4 animated_transform = transforms[joints.x + ubo.transform_length * gl_InstanceIndex] * weights.x
        + transforms[joints.y + ubo.transform_length * gl_InstanceIndex] * weights.y
        + transforms[joints.z + ubo.transform_length * gl_InstanceIndex] * weights.z
        + transforms[joints.w + ubo.transform_length * gl_InstanceIndex] * weights.w;
    mat4 world_transform = model * animated_transform;
    vec4 world_position = world_transform * vec4(position, 1.0);
    gl_Position = ubo.view_proj * world_position;
    light_direction = normalize(ubo.light_position - world_position.xyz);
    camera_direction = normalize(ubo.camera_position - world_position.xyz);
    normal_direction = normalize((world_transform * vec4(normal, 0.0)).xyz);
    tangent_direction = normalize((world_transform * vec4(tangent, 0.0)).xyz);
    tex_coords = tex_coords_in;
    tex_metal_coords = tex_metal_coords_in;
    tex_normal_coords = tex_normal_coords_in;
}