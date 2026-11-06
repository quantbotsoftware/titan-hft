struct Params {
    len: u32,
    ema_fast: u32,
    ema_slow: u32,
    rsi_period: u32,
}

@group(0) @binding(0) var<storage, read> closes: array<f32>;
@group(0) @binding(1) var<storage, read_write> ema_fast_out: array<f32>;
@group(0) @binding(2) var<storage, read_write> ema_slow_out: array<f32>;
@group(0) @binding(3) var<storage, read_write> rsi_out: array<f32>;
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x > 0u) {
        return;
    }
    let len = params.len;
    if (len == 0u) {
        return;
    }

    // EMA fast
    let mult_fast = 2.0 / (f32(params.ema_fast) + 1.0);
    var prev_fast = closes[0];
    ema_fast_out[0] = prev_fast;
    var i: u32 = 1u;
    loop {
        if (i >= len) { break; }
        let v = closes[i];
        let next = (v - prev_fast) * mult_fast + prev_fast;
        prev_fast = next;
        ema_fast_out[i] = prev_fast;
        i = i + 1u;
    }

    // EMA slow
    let mult_slow = 2.0 / (f32(params.ema_slow) + 1.0);
    var prev_slow = closes[0];
    ema_slow_out[0] = prev_slow;
    i = 1u;
    loop {
        if (i >= len) { break; }
        let v = closes[i];
        let next = (v - prev_slow) * mult_slow + prev_slow;
        prev_slow = next;
        ema_slow_out[i] = prev_slow;
        i = i + 1u;
    }

    // RSI
    let period = params.rsi_period;
    if (len <= period) {
        i = 0u;
        loop {
            if (i >= len) { break; }
            rsi_out[i] = -1.0;
            i = i + 1u;
        }
        return;
    }

    // Fill initial None values
    i = 0u;
    loop {
        if (i < period) {
            rsi_out[i] = -1.0;
            i = i + 1u;
            continue;
        }
        break;
    }

    var avg_gain: f32 = 0.0;
    var avg_loss: f32 = 0.0;
    i = 1u;
    loop {
        if (i > period) { break; }
        let diff = closes[i] - closes[i - 1u];
        if (diff > 0.0) {
            avg_gain = avg_gain + diff;
        } else {
            avg_loss = avg_loss + (-diff);
        }
        i = i + 1u;
    }
    avg_gain = avg_gain / f32(period);
    avg_loss = avg_loss / f32(period);

    var rs: f32 = 0.0;
    var rsi_val: f32 = 100.0;
    if (avg_loss > 0.0) {
        rs = avg_gain / avg_loss;
        rsi_val = 100.0 - (100.0 / (1.0 + rs));
    }
    rsi_out[period] = rsi_val;

    i = period + 1u;
    loop {
        if (i >= len) { break; }
        let diff = closes[i] - closes[i - 1u];
        var gain: f32 = 0.0;
        var loss: f32 = 0.0;
        if (diff > 0.0) {
            gain = diff;
        } else {
            loss = -diff;
        }
        avg_gain = ((avg_gain * (f32(period) - 1.0)) + gain) / f32(period);
        avg_loss = ((avg_loss * (f32(period) - 1.0)) + loss) / f32(period);
        if (avg_loss > 0.0) {
            rs = avg_gain / avg_loss;
            rsi_val = 100.0 - (100.0 / (1.0 + rs));
        } else {
            rsi_val = 100.0;
        }
        rsi_out[i] = rsi_val;
        i = i + 1u;
    }
}
