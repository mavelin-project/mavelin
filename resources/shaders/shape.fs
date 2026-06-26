#version 330

in vec2 v_local_uv;
flat in vec4 v_color;
flat in vec2 v_half_size;
flat in vec4 v_radii;
flat in uint v_mode;

out vec4 f_color;
out vec4 f_bright_color;

uniform sampler2D atlas;

float sdRoundBox(vec2 p, vec2 b, vec4 r) {
    r.xy = (p.x > 0.0) ? r.xy : r.wz;
    r.x  = (p.y > 0.0) ? r.x  : r.y;
    vec2 q = abs(p) - b + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

void main() {
    if (v_mode == 0u) {
        float d = sdRoundBox(v_local_uv, v_half_size, v_radii);
        float a = 1.0 - smoothstep(-0.5, 0.5, d);

        f_color = vec4(v_color.rgb, v_color.a * a);
        f_bright_color = vec4(0.0);
    } else if (v_mode == 1u) {
        float mask = texture(atlas, v_local_uv).r;

        f_color = vec4(v_color.rgb, v_color.a * mask);
        f_bright_color = vec4(0.0);
    } else {
        f_color = v_color; // lyon triangles, passthrough
        f_bright_color = vec4(0.0);
    }
}