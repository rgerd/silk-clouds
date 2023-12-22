struct PushConstants {
  time: f32,
  chunk_id: u32
}

var<push_constant> push: PushConstants;

@group(0) @binding(0)
var terrain: texture_storage_3d<rgba16float, write>;

fn mod289_3(x: vec3<f32>) -> vec3<f32> { return x - floor(x * (1.0 / 289.0)) * 289.0; }
fn mod289(x: vec2<f32>) -> vec2<f32> { return x - floor(x * (1.0 / 289.0)) * 289.0; }
fn permute(x: vec3<f32>) -> vec3<f32> { return mod289_3(((x*34.0)+10.0)*x); }
fn snoise(v: vec2<f32>) -> f32 {
    let C = vec4(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);
    var i  = floor(v + dot(v, C.yy) );
    var x0 = v -   i + dot(i, C.xx);
    var i1 = vec2(0.0, 1.0);
    if (x0.x > x0.y) { i1 = vec2(1.0, 0.0); }
    var x12 = x0.xyxy + C.xxzz;
    x12.x -= i1.x;
    x12.y -= i1.y;
    i = mod289(i);
    var p = permute( permute( i.y + vec3(0.0, i1.y, 1.0 )) + i.x + vec3(0.0, i1.x, 1.0 ));
    var m = 0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw));
    m.x = max(m.x, 0.0); m.y = max(m.y, 0.0); m.z = max(m.z, 0.0);
    m = m*m ;
    m = m*m ;
    var x = 2.0 * fract(p * C.www) - 1.0;
    var h = abs(x) - 0.5;
    var ox = floor(x + 0.5);
    var a0 = x - ox;
    m *= 1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h );
    var g = vec3(a0.x  * x0.x  + h.x  * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130.0 * dot(m, g);
}

fn noise(v: vec3<f32>) -> f32 {
  let cloud_time = push.time / 9.0;
  var out = 0.0;
  
  var freq = 0.5;
  out += snoise(vec2(v.x * freq, v.y * freq + cloud_time)) * snoise(vec2(v.y * freq + cloud_time, v.z * freq));

  freq = 1.0;
  out += snoise(vec2(v.x * freq, v.y * freq + cloud_time)) * snoise(vec2(v.y * freq + cloud_time, v.z * freq)) * 0.5;

  freq = 2.0;
  out += snoise(vec2(v.x * freq, v.y * freq + cloud_time)) * snoise(vec2(v.y * freq + cloud_time, v.z * freq)) * 0.2;

  // freq = 4.0;
  // out += snoise(vec2(v.x * freq, v.y * freq + cloud_time)) * snoise(vec2(v.y * freq + cloud_time, v.z * freq)) * 0.1;

  // freq = 8.0;
  // out += snoise(vec2(v.x * freq, v.y * freq + cloud_time)) * snoise(vec2(v.y * freq + cloud_time, v.z * freq)) * 0.05;

  return clamp(pow(out * 2.0, 1.2), 0.0, 1.0);
}

const GRADIENT_D: f32 = 0.0001;

@compute @workgroup_size(5, 5, 5)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = f32(global_id.x) / 65.0 + f32((push.chunk_id >> 0u) & 1u);
    let y = f32(global_id.y) / 65.0 + f32((push.chunk_id >> 1u) & 1u);
    let z = f32(global_id.z) / 65.0 + f32((push.chunk_id >> 2u) & 1u);

    var density = noise(vec3(x, y, z));
    var gradient = normalize(vec3<f32>(
      (noise(vec3(x + GRADIENT_D, y, z)) - density) / GRADIENT_D, 
      (noise(vec3(x, y + GRADIENT_D, z)) - density) / GRADIENT_D, 
      (noise(vec3(x, y, z + GRADIENT_D)) - density) / GRADIENT_D));
    textureStore(terrain, global_id, vec4<f32>(density, gradient));
}