#version 330 core
uniform mat4 m, v, p;
uniform sampler2D t_Color;
uniform vec3 world_light_pos;

in vec2 UV;
in vec3 world_pos;
in vec3 camera_norm;
in vec3 camera_eye_dir;
in vec3 camera_light_dir;

out vec3 color;

void main() {
  vec3 light_color = vec3(1, 1, 1);
  float light_power = 50.0f;

  vec3 mat_kd = texture2D(t_Color, UV).rgb;
  vec3 mat_ka = vec3(0.4,0.4,0.4) * mat_kd;
  vec3 mat_ks = vec3(0.3,0.3,0.3);

  float distance = length(world_light_pos - world_pos);

  vec3 n = normalize( camera_norm );
  vec3 l = normalize( camera_light_dir );
  float cosTheta = clamp( dot( n, l ), 0, 1 );

  vec3 E = normalize(camera_eye_dir);

  vec3 R = reflect(-l, n);

  float cosAlpha = clamp( dot( E, R ), 0, 1 );

  color = mat_ka +
          mat_kd * light_color * light_power * cosTheta / (distance * distance) +
          mat_ks * light_color * light_power * pow(cosAlpha, 5) / (distance * distance);
}
