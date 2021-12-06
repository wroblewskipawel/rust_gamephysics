#version 460 core
#define VULKAN 100

layout(location=0) in VS_OUT {
    vec3 norm;
    vec4 tang;
    vec4 color;
    vec2 tex;
} fs_in;

layout(location=0) out vec4 frag_color;

void main() {
    frag_color = fs_in.color;
}