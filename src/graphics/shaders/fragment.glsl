#version 450

layout(binding = 1) uniform UniformBufferObject {
    vec4 color;
    float metalness;
    float roughness;
} ubo;
layout(binding = 3) uniform sampler2D tex;
layout(binding = 4) uniform sampler2D tex_metal;
layout(binding = 5) uniform sampler2D tex_normal;


layout(location = 0) in vec3 light_direction;
layout(location = 1) in vec3 camera_direction;
layout(location = 2) in vec3 normal_direction;
layout(location = 3) in vec2 tex_coords;
layout(location = 4) in vec2 tex_metal_coords;
layout(location = 5) in vec2 tex_normal_coords;
layout(location = 6) in vec3 tangent_direction;

layout(location = 0) out vec4 f_color;

const float lambertian_diffuse = 0.31830988618; // 1/pi
const float ambient_light = 0.01;

void main() {
    vec3 bitangent = cross(normal_direction, tangent_direction);
    vec4 tex_normal = texture(tex_normal, tex_normal_coords) * 2.0 - 1.0;
    vec3 normal = normalize(tex_normal.r * tangent_direction + tex_normal.g * bitangent + tex_normal.b * normal_direction);
    //normal = normal_direction;
    vec4 tex_color = texture(tex, tex_coords) * ubo.color;
    vec4 tex_metal = texture(tex_metal, tex_metal_coords);
    float metalness = ubo.metalness * tex_metal.x;
    float roughness = ubo.roughness * tex_metal.y;
    vec3 half_direction = normalize(camera_direction + light_direction);
    float NL = dot(normal, light_direction);
    float NV = dot(normal, camera_direction);
    float NH = dot(normal, half_direction);
    float non_metalness = 1 - metalness;
    roughness = roughness * roughness * roughness * roughness;
    float non_roughness = 1 - roughness;
    float microfacet_distribution_coeff = 1 - non_roughness * NH * NH;
    float visibility_coeff = (abs(NL) + sqrt(roughness + non_roughness * NL * NL)) * (abs(NV) + sqrt(roughness + non_roughness * NV * NV));
    float specular;
    if (NH <= 0.0) {
        specular = 0.0;
    } else {
        specular = max(roughness / (visibility_coeff * microfacet_distribution_coeff * microfacet_distribution_coeff), 0.0);
    }
    float schlick_coeff = 1 - dot(half_direction, camera_direction);
    schlick_coeff = schlick_coeff * schlick_coeff * schlick_coeff * schlick_coeff * schlick_coeff;
    float fresnel_mix_coeff = 0.04 + 0.96 * schlick_coeff;
    float albedo_coeff = max(NL, 0.0);
    float white_coeff = lambertian_diffuse * albedo_coeff * (metalness * specular * schlick_coeff + non_metalness * specular * fresnel_mix_coeff);
    float colored_coeff = lambertian_diffuse * (albedo_coeff * (metalness * specular * (1 - schlick_coeff) + non_metalness * (1 - fresnel_mix_coeff)) + ambient_light);
    vec3 color_temp = tex_color.rgb * colored_coeff + vec3(white_coeff);
    f_color = vec4(color_temp, tex_color.a);
}