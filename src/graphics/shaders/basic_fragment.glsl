#version 450

layout(binding = 1) uniform UniformBufferObject {
    vec4 color;
    float metalness;
    float roughness;
} ubo;

layout(location = 0) in vec3 light_direction;
layout(location = 1) in vec3 camera_direction;
layout(location = 2) in vec3 normal_direction;
layout(location = 0) out vec4 f_color;

const float lambertian_diffuse = 0.31830988618; // 1/pi
const float ambient_light = 0.1;

void main() {
    vec3 half_direction = normalize(camera_direction + light_direction);
    float NL = dot(normal_direction, light_direction);
    float NV = dot(normal_direction, camera_direction);
    float NH = dot(normal_direction, half_direction);
    float non_metalness = 1 - ubo.metalness;
    float roughness = ubo.roughness * ubo.roughness * ubo.roughness * ubo.roughness;
    float non_roughness = 1 - roughness;
    float microfacet_distribution_coeff = 1 - non_roughness * NH * NH;
    float visibility_coeff = (NL + sqrt(roughness + non_roughness * NL * NL)) * (NV + sqrt(roughness + non_roughness * NV * NV));
    float specular = max(roughness / (visibility_coeff * microfacet_distribution_coeff * microfacet_distribution_coeff), 0.0);
    float schlick_coeff = 1 - dot(half_direction, camera_direction);
    schlick_coeff = schlick_coeff * schlick_coeff * schlick_coeff * schlick_coeff * schlick_coeff;
    float fresnel_mix_coeff = 0.04 + 0.96 * schlick_coeff;
    float albedo_coeff = max(NL, 0.0);
    float white_coeff = lambertian_diffuse * albedo_coeff * (ubo.metalness * specular * schlick_coeff + non_metalness * specular * fresnel_mix_coeff);
    float colored_coeff = lambertian_diffuse * (albedo_coeff * (ubo.metalness * specular * (1 - schlick_coeff) + non_metalness * (1 - fresnel_mix_coeff)) + ambient_light);
    vec3 color_temp = ubo.color.rgb * colored_coeff + vec3(white_coeff);
    f_color = vec4(color_temp, ubo.color.a);
}