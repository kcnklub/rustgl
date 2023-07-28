#version 330 core
out vec4 FragColor;

uniform sampler2D texture0;
uniform vec3 lightPos;

in vec2 TexCoord;
in vec3 Normal;
in vec3 FragPos;

void main() 
{
    vec3 ambient = 0.25 * texture(texture0, TexCoord).rgb;

    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(lightPos - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = vec3(1.0, 1.0, 1.0) * diff * texture(texture0, TexCoord).rgb;

    FragColor = vec4(ambient + diffuse, 1.0);
} 


