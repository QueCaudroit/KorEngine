#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Vertex {
    float vertex[];
};

layout(set = 0, binding = 1) buffer TexCoord {
    float tex_coord[];
};

layout(set = 0, binding = 2) buffer Normal {
    float normal[];
};

layout(set = 0, binding = 3) buffer Tangent {
    float tangent[];
};

void main() {
    uint idx = gl_GlobalInvocationID.x;
    vec3 a = vec3(vertex[9*idx], vertex[9*idx + 1], vertex[9*idx + 2]);
    vec3 b = vec3(vertex[9*idx + 3], vertex[9*idx + 4], vertex[9*idx + 5]);
    vec3 c = vec3(vertex[9*idx + 6], vertex[9*idx + 7], vertex[9*idx + 8]);
    
    vec2 uv_a = vec2(tex_coord[6*idx], tex_coord[6*idx + 1]);
    vec2 uv_b = vec2(tex_coord[6*idx + 2], tex_coord[6*idx + 3]);
    vec2 uv_c = vec2(tex_coord[6*idx + 4], tex_coord[6*idx + 5]);

    vec3 ab = b - a;
    vec3 ac = c - a;
    vec2 uv_ab = uv_b - uv_a;
    vec2 uv_ac = uv_c - uv_a;

    vec3 na = vec3(normal[9*idx], normal[9*idx + 1], normal[9*idx + 2]);
    vec3 nb = vec3(normal[9*idx + 3], normal[9*idx + 4], normal[9*idx + 5]);
    vec3 nc = vec3(normal[9*idx + 6], normal[9*idx + 7], normal[9*idx + 8]);

    float det = (uv_ab.x * uv_ac.y - uv_ac.x * uv_ab.y);
    

    vec3 tangent_a;
    vec3 tangent_b;
    vec3 tangent_c;
    if (det != 0.0) {
        det = 1 / det;
        vec3 tangent_raw = det * uv_ac.y * ab - det * uv_ab.y * ac;
        tangent_a = normalize(tangent_raw - na * dot(tangent_raw, na));
        tangent_b = normalize(tangent_raw - nb * dot(tangent_raw, nb));
        tangent_c = normalize(tangent_raw - nc * dot(tangent_raw, nc));
    } else {
        tangent_a = normalize(vec3(-na.y - na.z, na.x, na.x));
        tangent_b = normalize(vec3(-nb.y - nb.z, nb.x, nb.x));
        tangent_c = normalize(vec3(-nc.y - nc.z, nc.x, nc.x));
    }

    tangent[9*idx] = tangent_a.x;
    tangent[9*idx + 1] = tangent_a.y;
    tangent[9*idx + 2] = tangent_a.z;
    tangent[9*idx + 3] = tangent_b.x;
    tangent[9*idx + 4] = tangent_b.y;
    tangent[9*idx + 5] = tangent_b.z;
    tangent[9*idx + 6] = tangent_c.x;
    tangent[9*idx + 7] = tangent_c.y;
    tangent[9*idx + 8] = tangent_c.z;
}
