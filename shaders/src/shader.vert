#version 460 core
#define VULKAN 100

layout(location=0) in vec3 pos;
layout(location=1) in vec3 norm;
layout(location=2) in vec4 tang;
layout(location=3) in vec4 color;
layout(location=4) in vec2 tex;

layout(push_constant) uniform Transforms {
    mat4 camera;
    mat4 world;
} transforms;

layout(location=0) out VS_OUT {
    vec3 norm;
    vec4 tang;
    vec4 color;
    vec2 tex;
} vs_out;

void main() {
    vs_out.norm = norm;
    vs_out.tang = tang;
    vs_out.color = color;
    vs_out.tex = tex;
    gl_Position = transforms.camera * transforms.world * vec4(pos, 1.0);
}