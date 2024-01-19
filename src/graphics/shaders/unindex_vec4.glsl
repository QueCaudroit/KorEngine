#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Input {
    vec4 input_data[];
};

layout(set = 0, binding = 1) buffer Index {
    uint index[];
};

layout(set = 0, binding = 2) buffer Output {
    vec4 output_data[];
};

void main() {
    uint idx = gl_GlobalInvocationID.x;
    output_data[idx] = input_data[index[idx]];
}
