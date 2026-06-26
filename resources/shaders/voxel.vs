#version 140

// chunk-local position
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
// chunk XZ position on grid
uniform ivec3 chunk;
uniform vec3 camera_pos;
uniform vec3 sun_position;

// vec4 toLinear(vec4 sRGB) {
//   bvec3 cutoff = lessThan(sRGB.rgb, vec3(0.04045));
//   vec3 higher = pow((sRGB.rgb + vec3(0.055)) / vec3(1.055), vec3(2.4));
//   vec3 lower = sRGB.rgb / vec3(12.92);

//   return vec4(mix(higher, lower, cutoff), sRGB.a);
// }

float fog_spherical_distance(vec3 pos) { return length(pos); }
float fog_cylindrical_distance(vec3 pos) { return max(length(pos.xz), abs(pos.y)); }

void main() {
  float block_light = (float(light & uint(15)) + 1.0) / 16.0;
  float sun_light = (float((light >> uint(4)) & uint(15)) + 1.0) / 16.0;
  
  sun_light *= max(sun_position.y * 0.45 + 0.5, 0.02);
  
  float light_intensity = min(max(block_light, sun_light), 1.0);

  vec4 linear_color = color / 255.0;
  vec3 pos = chunk - camera_pos + position;

  gl_Position = matrix * vec4(pos, 1.0);

  v_spherical_dist = fog_spherical_distance(pos);
  v_cylindrical_dist = fog_cylindrical_distance(pos);
  v_color = linear_color;
  v_light_intensity = light_intensity;
  v_tex_coords = uv;
}
