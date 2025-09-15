struct Globals {
	screen_wh: vec2<f32>, 
	_pad: vec2<f32>,
};
@group(0) @binding(0) var<uniform> U: Globals;

struct VSOut {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) v_color: vec3<f32>,
	@location(1) v_center_ndc: vec2<f32>,
	@location(2) v_radius_ndc: vec2<f32>,
	@location(3) v_ndc: vec2<f32>,        
};

fn px_to_ndc(px: vec2<f32>) -> vec2<f32> {
	let half = 0.5 * U.screen_wh;

	return vec2<f32>(px.x / half.x, px.y / half.y);
}

@vertex
fn vs_main(
	@location(0) quad_pos: vec2<f32>,
	@location(1) i_pos_px: vec2<f32>,   
	@location(2) i_radius_px: f32,     
	@location(3) i_color: vec3<f32>,
) -> VSOut {
	var out: VSOut;

	let local_px = quad_pos * i_radius_px;  
	let world_px = i_pos_px + local_px;

	let ndc = px_to_ndc(world_px);
	out.clip_position = vec4<f32>(ndc, 0.0, 1.0);

	out.v_center_ndc = px_to_ndc(i_pos_px);
	out.v_radius_ndc = 2.0 * vec2<f32>(i_radius_px) / U.screen_wh; 
	out.v_ndc = ndc;

	out.v_color = i_color;
	return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
	let d = vec2<f32>(
		(in.v_ndc.x - in.v_center_ndc.x) / in.v_radius_ndc.x,
		(in.v_ndc.y - in.v_center_ndc.y) / in.v_radius_ndc.y
	);

	if (dot(d, d) > 1.0) { discard; }

	return vec4<f32>(in.v_color, 1.0);
}
