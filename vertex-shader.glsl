#version 330
layout(location = 0) in vec3 u_pos;
layout(location = 1) in vec3 u_normal;
layout(location = 2) in vec2 u_uv;

out vec2 UV;
out vec3 world_pos;
out vec3 camera_norm;
out vec3 camera_eye_dir;
out vec3 camera_light_dir;

uniform mat4 u_Model, u_View, u_Proj;
uniform vec3 world_light_pos;

void main() {
  vec4 h_pos = vec4(u_pos, 1.0);
  gl_Position = u_Proj * u_View * u_Model * h_pos;

  world_pos = (u_Model * h_pos).xyz;

  camera_eye_dir = vec3(0,0,0) - ( u_View * u_Model * h_pos ).xyz;

  camera_light_dir = (u_View * vec4(world_light_pos, 1.0)).xyz + camera_eye_dir;

  camera_norm = ( u_View * u_Model * vec4(u_normal,0)).xyz;

  UV = u_uv;
}
