
/////////////////////////////////////// wgpu 0.12

// struct VertexInput {
// 	[[location(0)]] position: vec2<f32>;
// };

// struct VertexOutput {
// 	[[builtin(position)]] clip_position: vec4<f32>;
// 	[[location(1)]] background_uv: vec2<f32>;
// };


// [[group(0), binding(0)]]
// var t_background: texture_2d<f32>;

// [[group(0), binding(1)]]
// var s_background: sampler;


// struct Uniforms {
// 	top_left: vec2<f32>;
// 	bottom_right: vec2<f32>;
// };

// [[group(1), binding(0)]]
// var<uniform> uniforms: Uniforms;



// [[stage(vertex)]]
// fn vs_main(model: VertexInput) -> VertexOutput {
// 	var out: VertexOutput;

// 	out.clip_position = vec4<f32>(model.position.x, model.position.y, 0.0, 1.0);
// 	out.background_uv = mix(
// 		uniforms.top_left,
// 		uniforms.bottom_right,
// 		model.position * 0.5 + 0.5
// 	);
// 	out.background_uv.y = uniforms.top_left.y + uniforms.bottom_right.y - out.background_uv.y;

// 	return out;
// }



// [[stage(fragment)]]
// fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
// 	return textureSample(t_background, s_background, in.background_uv);
// }




/////////////////////////////////////// wgpu master


struct VertexInput {
	@location(0) position: vec2<f32>
};

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(1) background_uv: vec2<f32>
};


@group(0)
@binding(0)
var t_background: texture_2d<f32>;

@group(0)
@binding(1)
var s_background: sampler;


struct Uniforms {
	top_left: vec2<f32>,
	bottom_right: vec2<f32>
};

@group(1)
@binding(0)
var<uniform> uniforms: Uniforms;



@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
	var out: VertexOutput;

	out.clip_position = vec4<f32>(model.position.x, model.position.y, 0.0, 1.0);
	out.background_uv = mix(
		uniforms.top_left,
		uniforms.bottom_right,
		model.position * 0.5 + 0.5
	);
	out.background_uv.y = uniforms.top_left.y + uniforms.bottom_right.y - out.background_uv.y;

	return out;
}



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(t_background, s_background, in.background_uv);
}


