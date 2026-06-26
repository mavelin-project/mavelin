#version 330

in vec2 position;
in vec2 local_uv;
in uint shape_id;

out vec2 v_local_uv;
flat out vec4 v_color;
flat out vec2 v_half_size;
flat out vec4 v_radii;
flat out uint v_mode;

uniform samplerBuffer shape_data;
uniform mat4 matrix;

void main() {
  int base = int(shape_id) * 3;  // 3 texels per shape

  vec4 t0 = texelFetch(shape_data, base + 0);  // color
  vec4 t1 = texelFetch(shape_data, base + 1);  // half_size.xy, radii.xy (tl,tr)
  vec4 t2 = texelFetch(shape_data, base + 2);  // radii.zw (br,bl), mode, _

  v_color     = t0;
  v_half_size = t1.xy;
  v_radii     = vec4(t1.zw, t2.xy);
  v_mode      = floatBitsToUint(t2.z);
  v_local_uv  = local_uv;

  gl_Position = matrix * vec4(position, 1.0, 1.0);
}
