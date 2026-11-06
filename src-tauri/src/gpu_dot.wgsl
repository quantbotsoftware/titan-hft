struct Params {
  len: u32,
  total_threads: u32,
  _pad0: u32,
  _pad1: u32,
};

@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> out: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

const WORKGROUP_SIZE: u32 = 256u;
var<workgroup> shared_sum: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
  @builtin(global_invocation_id) gid: vec3<u32>,
  @builtin(local_invocation_id) lid: vec3<u32>,
  @builtin(workgroup_id) wid: vec3<u32>,
) {
  let len = params.len;
  let total_threads = params.total_threads;
  if (len == 0u || total_threads == 0u) {
    return;
  }

  var sum: f32 = 0.0;
  var i: u32 = gid.x;
  loop {
    if (i >= len) { break; }
    sum = sum + a[i] * b[i];
    i = i + total_threads;
  }

  shared_sum[lid.x] = sum;
  workgroupBarrier();

  var stride: u32 = WORKGROUP_SIZE / 2u;
  loop {
    if (stride == 0u) { break; }
    if (lid.x < stride) {
      shared_sum[lid.x] = shared_sum[lid.x] + shared_sum[lid.x + stride];
    }
    workgroupBarrier();
    stride = stride / 2u;
  }

  if (lid.x == 0u) {
    out[wid.x] = shared_sum[0];
  }
}
