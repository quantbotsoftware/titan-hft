struct Params {
  series_count: u32,
  ema_fast: u32,
  ema_slow: u32,
  rsi_period: u32,
}

@group(0) @binding(0) var<storage, read> closes: array<f32>;
@group(0) @binding(1) var<storage, read> offsets: array<u32>;
@group(0) @binding(2) var<storage, read> lengths: array<u32>;
@group(0) @binding(3) var<storage, read_write> ema_fast_out: array<f32>;
@group(0) @binding(4) var<storage, read_write> ema_slow_out: array<f32>;
@group(0) @binding(5) var<storage, read_write> rsi_out: array<f32>;
@group(0) @binding(6) var<uniform> params: Params;

const WORKGROUP_SIZE: u32 = 256u;
var<workgroup> shared_a: array<f32, 256>;
var<workgroup> shared_b: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
  @builtin(local_invocation_id) lid: vec3<u32>,
  @builtin(workgroup_id) wid: vec3<u32>,
) {
  let series_idx = wid.x;
  if (series_idx >= params.series_count) {
    return;
  }

  let start = offsets[series_idx];
  let len = lengths[series_idx];
  if (len == 0u) {
    return;
  }

  // If series is too long, fall back to sequential in lane 0.
  if (len > WORKGROUP_SIZE) {
    if (lid.x == 0u) {
      let mult_fast = 2.0 / (f32(params.ema_fast) + 1.0);
      var prev_fast = closes[start];
      ema_fast_out[start] = prev_fast;
      for (var i: u32 = 1u; i < len; i = i + 1u) {
        let pos = start + i;
        let v = closes[pos];
        let next = (v - prev_fast) * mult_fast + prev_fast;
        prev_fast = next;
        ema_fast_out[pos] = prev_fast;
      }

      let mult_slow = 2.0 / (f32(params.ema_slow) + 1.0);
      var prev_slow = closes[start];
      ema_slow_out[start] = prev_slow;
      for (var i: u32 = 1u; i < len; i = i + 1u) {
        let pos = start + i;
        let v = closes[pos];
        let next = (v - prev_slow) * mult_slow + prev_slow;
        prev_slow = next;
        ema_slow_out[pos] = prev_slow;
      }
    }
  } else {
    let x0 = closes[start];

    // EMA fast prefix-scan
    let alpha_fast = 2.0 / (f32(params.ema_fast) + 1.0);
    if (lid.x < len) {
      if (lid.x == 0u) {
        shared_a[lid.x] = 1.0;
        shared_b[lid.x] = 0.0;
      } else {
        shared_a[lid.x] = 1.0 - alpha_fast;
        shared_b[lid.x] = alpha_fast * closes[start + lid.x];
      }
    } else {
      shared_a[lid.x] = 1.0;
      shared_b[lid.x] = 0.0;
    }
    workgroupBarrier();

    var offset: u32 = 1u;
    loop {
      if (offset >= WORKGROUP_SIZE) { break; }
      var new_a = shared_a[lid.x];
      var new_b = shared_b[lid.x];
      if (lid.x >= offset) {
        let a_left = shared_a[lid.x - offset];
        let b_left = shared_b[lid.x - offset];
        let a_right = new_a;
        let b_right = new_b;
        new_a = a_right * a_left;
        new_b = b_right + a_right * b_left;
      }
      workgroupBarrier();
      shared_a[lid.x] = new_a;
      shared_b[lid.x] = new_b;
      workgroupBarrier();
      offset = offset * 2u;
    }

    if (lid.x < len) {
      ema_fast_out[start + lid.x] = shared_a[lid.x] * x0 + shared_b[lid.x];
    }

    // EMA slow prefix-scan
    let alpha_slow = 2.0 / (f32(params.ema_slow) + 1.0);
    if (lid.x < len) {
      if (lid.x == 0u) {
        shared_a[lid.x] = 1.0;
        shared_b[lid.x] = 0.0;
      } else {
        shared_a[lid.x] = 1.0 - alpha_slow;
        shared_b[lid.x] = alpha_slow * closes[start + lid.x];
      }
    } else {
      shared_a[lid.x] = 1.0;
      shared_b[lid.x] = 0.0;
    }
    workgroupBarrier();

    offset = 1u;
    loop {
      if (offset >= WORKGROUP_SIZE) { break; }
      var new_a2 = shared_a[lid.x];
      var new_b2 = shared_b[lid.x];
      if (lid.x >= offset) {
        let a_left2 = shared_a[lid.x - offset];
        let b_left2 = shared_b[lid.x - offset];
        let a_right2 = new_a2;
        let b_right2 = new_b2;
        new_a2 = a_right2 * a_left2;
        new_b2 = b_right2 + a_right2 * b_left2;
      }
      workgroupBarrier();
      shared_a[lid.x] = new_a2;
      shared_b[lid.x] = new_b2;
      workgroupBarrier();
      offset = offset * 2u;
    }

    if (lid.x < len) {
      ema_slow_out[start + lid.x] = shared_a[lid.x] * x0 + shared_b[lid.x];
    }
  }

  // RSI (Wilder) - multi-pass across blocks, prefix-scan within each block
  let period = params.rsi_period;
  if (len <= period) {
    if (lid.x < len) {
      rsi_out[start + lid.x] = -1.0;
    }
    return;
  }

  // Compute initial avg_gain/avg_loss in lane 0
  if (lid.x == 0u) {
    var g0: f32 = 0.0;
    var l0: f32 = 0.0;
    for (var i: u32 = 1u; i <= period; i = i + 1u) {
      let pos = start + i;
      let diff = closes[pos] - closes[pos - 1u];
      if (diff > 0.0) {
        g0 = g0 + diff;
      } else {
        l0 = l0 + (-diff);
      }
    }
    shared_a[0] = g0 / f32(period);
    shared_b[0] = l0 / f32(period);
  }
  workgroupBarrier();

  var start_avg_gain: f32 = shared_a[0];
  var start_avg_loss: f32 = shared_b[0];
  let period_f = f32(period);
  let smooth_a = (period_f - 1.0) / period_f;

  let num_blocks = (len + WORKGROUP_SIZE - 1u) / WORKGROUP_SIZE;
  var block: u32 = 0u;
  loop {
    if (block >= num_blocks) { break; }
    let base = block * WORKGROUP_SIZE;
    let idx = base + lid.x;

    // Gain prefix-scan
    if (idx < len) {
      let pos = start + idx;
      if (idx <= period) {
        shared_a[lid.x] = 1.0;
        shared_b[lid.x] = 0.0;
      } else {
        let diff = closes[pos] - closes[pos - 1u];
        let gain = select(0.0, diff, diff > 0.0);
        shared_a[lid.x] = smooth_a;
        shared_b[lid.x] = gain / period_f;
      }
    } else {
      shared_a[lid.x] = 1.0;
      shared_b[lid.x] = 0.0;
    }
    workgroupBarrier();

    var offset2: u32 = 1u;
    loop {
      if (offset2 >= WORKGROUP_SIZE) { break; }
      var new_a = shared_a[lid.x];
      var new_b = shared_b[lid.x];
      if (lid.x >= offset2) {
        let a_left = shared_a[lid.x - offset2];
        let b_left = shared_b[lid.x - offset2];
        let a_right = new_a;
        let b_right = new_b;
        new_a = a_right * a_left;
        new_b = b_right + a_right * b_left;
      }
      workgroupBarrier();
      shared_a[lid.x] = new_a;
      shared_b[lid.x] = new_b;
      workgroupBarrier();
      offset2 = offset2 * 2u;
    }

    var avg_gain: f32 = start_avg_gain;
    if (idx < len) {
      let pos = start + idx;
      avg_gain = shared_a[lid.x] * start_avg_gain + shared_b[lid.x];
      rsi_out[pos] = avg_gain; // temp storage
      if (idx < period) {
        rsi_out[pos] = -1.0;
      }
    }
    workgroupBarrier();

    // Loss prefix-scan
    if (idx < len) {
      let pos = start + idx;
      if (idx <= period) {
        shared_a[lid.x] = 1.0;
        shared_b[lid.x] = 0.0;
      } else {
        let diff = closes[pos] - closes[pos - 1u];
        let loss = select(0.0, -diff, diff < 0.0);
        shared_a[lid.x] = smooth_a;
        shared_b[lid.x] = loss / period_f;
      }
    } else {
      shared_a[lid.x] = 1.0;
      shared_b[lid.x] = 0.0;
    }
    workgroupBarrier();

    offset2 = 1u;
    loop {
      if (offset2 >= WORKGROUP_SIZE) { break; }
      var new_a2 = shared_a[lid.x];
      var new_b2 = shared_b[lid.x];
      if (lid.x >= offset2) {
        let a_left2 = shared_a[lid.x - offset2];
        let b_left2 = shared_b[lid.x - offset2];
        let a_right2 = new_a2;
        let b_right2 = new_b2;
        new_a2 = a_right2 * a_left2;
        new_b2 = b_right2 + a_right2 * b_left2;
      }
      workgroupBarrier();
      shared_a[lid.x] = new_a2;
      shared_b[lid.x] = new_b2;
      workgroupBarrier();
      offset2 = offset2 * 2u;
    }

    var avg_loss: f32 = start_avg_loss;
    if (idx < len) {
      let pos = start + idx;
      avg_loss = shared_a[lid.x] * start_avg_loss + shared_b[lid.x];
      if (idx < period) {
        rsi_out[pos] = -1.0;
      } else {
        let avg_gain_val = rsi_out[pos];
        var rsi_val: f32 = 100.0;
        if (avg_loss > 0.0) {
          let rs = avg_gain_val / avg_loss;
          rsi_val = 100.0 - (100.0 / (1.0 + rs));
        }
        rsi_out[pos] = rsi_val;
      }
    }
    workgroupBarrier();

    // Update start averages for next block
    let last_idx = min(base + WORKGROUP_SIZE - 1u, len - 1u);
    let last_tid = last_idx - base;
    if (lid.x == last_tid) {
      shared_a[0] = avg_gain;
      shared_b[0] = avg_loss;
    }
    workgroupBarrier();
    if (lid.x == 0u) {
      start_avg_gain = shared_a[0];
      start_avg_loss = shared_b[0];
    }
    workgroupBarrier();

    block = block + 1u;
  }
}
