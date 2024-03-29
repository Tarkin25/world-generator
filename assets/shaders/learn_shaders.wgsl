#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::pbr_types

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

struct GradientPoint {
    color: vec4<f32>,
    height: f32,
};

@group(1) @binding(0)
var<storage, read> gradient_points: array<GradientPoint>;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let lowest_gradient_point = gradient_points[0].height;
    let highest_gradient_point = gradient_points[arrayLength(&gradient_points) - 1u].height;

    var output_color: vec4<f32>;

    //custom stuff
    let height = in.world_position.y;

    if height < lowest_gradient_point {
        output_color = gradient_points[0].color;
    } else if height > highest_gradient_point {
        output_color = gradient_points[arrayLength(&gradient_points) - 1u].color;
    } else {
        for (var i = 0u; i < arrayLength(&gradient_points) - 1u; i++) {
            if gradient_points[i].height <= height && gradient_points[i + 1u].height > height {
              let alpha = (height - gradient_points[i].height) / (gradient_points[i+1u].height - gradient_points[i].height);
              output_color = mix(gradient_points[i].color, gradient_points[i+1u].color, alpha);
                //output_color = gradient_points[i].color;
            }
        }
    }

    let up = vec3<f32>(0.0, 1.0, 0.0);
    let angle = acos(dot(up, in.world_normal) / length(up) * length(in.world_normal));

    if angle >= radians(45.0) {
        output_color = vec4<f32>(0.5, 0.5, 0.5, 1.0);
    }

    //output_color = vec4<f32>(0.5, angle, 0.5, 1.0);

    var pbr_input: PbrInput;

    pbr_input.material.base_color = output_color;

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        true,
        in.is_front,
    );
    pbr_input.material.emissive = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    pbr_input.material.perceptual_roughness = 0.8;
    pbr_input.material.metallic = 0.01;

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = apply_normal_mapping(
        0u,
        pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
#ifdef VERTEX_UVS
        in.uv,
#endif
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);
    output_color = pbr(pbr_input);

#ifdef TONEMAP_IN_SHADER
    output_color = tone_mapping(output_color);
#endif
#ifdef DEBAND_DITHER
    var output_rgb = output_color.rgb;
    output_rgb = pow(output_rgb, vec3<f32>(1.0 / 2.2));
    output_rgb = output_rgb + screen_space_dither(in.frag_coord.xy);
    // This conversion back to linear space is required because our output texture format is
    // SRGB; the GPU will assume our output is linear and will apply an SRGB conversion.
    output_rgb = pow(output_rgb, vec3<f32>(2.2));
    output_color = vec4(output_rgb, output_color.a);
#endif
    return output_color;
}
