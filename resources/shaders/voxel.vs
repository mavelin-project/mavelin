#version 140

in vec3 position;
in uint light;
in highp vec2 uv;
in uvec4 color;

out highp vec2 v_tex_coords;
out float v_light_intensity;
out vec4 v_color;

// fog
out float v_spherical_dist;
out float v_cylindrical_dist;

uniform mat4 matrix;
uniform vec3 sun_position;

vec4 toLinear(vec4 sRGB) {
  bvec3 cutoff = lessThan(sRGB.rgb, vec3(0.04045));
  vec3 higher = pow((sRGB.rgb + vec3(0.055)) / vec3(1.055), vec3(2.4));
  vec3 lower = sRGB.rgb / vec3(12.92);

  return vec4(mix(higher, lower, cutoff), sRGB.a);
}

float fog_spherical_distance(vec3 pos) { return length(pos); }
float fog_cylindrical_distance(vec3 pos) { return max(length(pos.xz), abs(pos.y)); }

void main() {
  float block_light = (float(light & uint(15)) + 1.0) / 16.0;
  float sun_light = (float((light >> uint(4)) & uint(15)) + 1.0) / 16.0;
  float sunlight = sun_light * max(sun_position.y * 0.45 + 0.5, 0.02);
  float light_intensity = min(max(block_light, sunlight), 1.0);

  vec4 linear_color = toLinear(color / 255.0);
  vec4 pos = matrix * vec4(position, 1.0);

  gl_Position = pos;

  v_spherical_dist = fog_spherical_distance(pos.xyz);
  v_cylindrical_dist = fog_cylindrical_distance(pos.xyz);
  v_color = linear_color;
  v_light_intensity = light_intensity;
  v_tex_coords = uv;
}
