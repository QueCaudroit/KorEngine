#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Input {
    float input_data[];
};

layout(set = 0, binding = 1) buffer Output {
    float output_data[];
};

void main() {
    uint idx = gl_GlobalInvocationID.x;
    vec3 a = vec3(input_data[9*idx], input_data[9*idx + 1], input_data[9*idx + 2]);
    vec3 b = vec3(input_data[9*idx + 3], input_data[9*idx + 4], input_data[9*idx + 5]);
    vec3 c = vec3(input_data[9*idx + 6], input_data[9*idx + 7], input_data[9*idx + 8]);
    vec3 result = normalize(cross(b-a, c-a));
    output_data[9*idx] = result.x;
    output_data[9*idx + 1] = result.y;
    output_data[9*idx + 2] = result.z;
    output_data[9*idx + 3] = result.x;
    output_data[9*idx + 4] = result.y;
    output_data[9*idx + 5] = result.z;
    output_data[9*idx + 6] = result.x;
    output_data[9*idx + 7] = result.y;
    output_data[9*idx + 8] = result.z;
}
