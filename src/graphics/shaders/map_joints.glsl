#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Input {
    uvec4 input_data[];
};

layout(set = 0, binding = 1) buffer Mapping {
    uint mapping[];
};

void main() {
    uint idx = gl_GlobalInvocationID.x;
    uvec4 temp = input_data[idx];
    input_data[idx] = uvec4(mapping[temp.x], mapping[temp.y], mapping[temp.z], mapping[temp.w]);
}
