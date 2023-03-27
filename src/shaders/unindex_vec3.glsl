#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Input {
    float input_data[];
};

layout(set = 0, binding = 1) buffer Index {
    uint index[];
};

layout(set = 0, binding = 2) buffer Output {
    float output_data[];
};

void main() {
    uint idx = gl_GlobalInvocationID.x;
    output_data[3*idx] = input_data[3*index[idx]];
    output_data[3*idx + 1] = input_data[3*index[idx] + 1];
    output_data[3*idx + 2] = input_data[3*index[idx] + 2];
}
