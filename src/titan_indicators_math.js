function ema(values, period) {
    const k = 2 / (period + 1);
    const out = [];
    let prev = null;
    for (const v of values) {
        if (prev === null) prev = v;
        else prev = (v - prev) * k + prev;
        out.push(prev);
    }
    return out;
}

function bbands(values, period, mult) {
    const upper = [];
    const mid = [];
    const lower = [];
    for (let i = 0; i < values.length; i++) {
        if (i < period - 1) {
            upper.push(null); mid.push(null); lower.push(null);
            continue;
        }
        const slice = values.slice(i + 1 - period, i + 1);
        const mean = slice.reduce((a, b) => a + b, 0) / period;
        const variance = slice.reduce((a, b) => a + (b - mean) ** 2, 0) / (period - 1);
        const std = Math.sqrt(variance);
        mid.push(mean);
        upper.push(mean + mult * std);
        lower.push(mean - mult * std);
    }
    return { upper, mid, lower };
}

function rsi(values, period) {
    if (values.length < period + 1) return values.map(() => null);
    const gains = [];
    const losses = [];
    for (let i = 1; i < values.length; i++) {
        const diff = values[i] - values[i - 1];
        gains.push(diff > 0 ? diff : 0);
        losses.push(diff < 0 ? -diff : 0);
    }
    let avgGain = gains.slice(0, period).reduce((a, b) => a + b, 0) / period;
    let avgLoss = losses.slice(0, period).reduce((a, b) => a + b, 0) / period;
    const out = Array(period).fill(null);
    let rs = avgLoss === 0 ? 100 : avgGain / avgLoss;
    out.push(100 - (100 / (1 + rs)));
    for (let i = period; i < gains.length; i++) {
        avgGain = ((avgGain * (period - 1)) + gains[i]) / period;
        avgLoss = ((avgLoss * (period - 1)) + losses[i]) / period;
        rs = avgLoss === 0 ? 100 : avgGain / avgLoss;
        out.push(100 - (100 / (1 + rs)));
    }
    return out;
}

function macd(values, fast, slow, signal) {
    const emaFast = ema(values, fast);
    const emaSlow = ema(values, slow);
    const macdLine = values.map((_, i) => emaFast[i] - emaSlow[i]);
    const signalLine = ema(macdLine, signal);
    const hist = macdLine.map((v, i) => v - signalLine[i]);
    return { macdLine, signalLine, hist };
}

function adx(data, period) {
    const len = data.length;
    const out = Array(len).fill(null);
    if (len <= period * 2) return out;
    const tr = Array(len).fill(0);
    const plusDM = Array(len).fill(0);
    const minusDM = Array(len).fill(0);
    for (let i = 1; i < len; i++) {
        const cur = data[i];
        const prev = data[i - 1];
        const high = Number(cur.high);
        const low = Number(cur.low);
        const prevClose = Number(prev.close);
        const prevHigh = Number(prev.high);
        const prevLow = Number(prev.low);
        const range1 = high - low;
        const range2 = Math.abs(high - prevClose);
        const range3 = Math.abs(low - prevClose);
        tr[i] = Math.max(range1, range2, range3);
        const upMove = high - prevHigh;
        const downMove = prevLow - low;
        plusDM[i] = (upMove > downMove && upMove > 0) ? upMove : 0;
        minusDM[i] = (downMove > upMove && downMove > 0) ? downMove : 0;
    }
    let trSum = 0;
    let plusSum = 0;
    let minusSum = 0;
    for (let i = 1; i <= period; i++) {
        trSum += tr[i];
        plusSum += plusDM[i];
        minusSum += minusDM[i];
    }
    const dx = Array(len).fill(null);
    for (let i = period + 1; i < len; i++) {
        trSum = trSum - (trSum / period) + tr[i];
        plusSum = plusSum - (plusSum / period) + plusDM[i];
        minusSum = minusSum - (minusSum / period) + minusDM[i];
        const diPlus = trSum === 0 ? 0 : (100 * plusSum / trSum);
        const diMinus = trSum === 0 ? 0 : (100 * minusSum / trSum);
        const denom = diPlus + diMinus;
        dx[i] = denom === 0 ? 0 : (100 * Math.abs(diPlus - diMinus) / denom);
    }
    let adxVal = 0;
    let count = 0;
    for (let i = period + 1; i <= period * 2; i++) {
        if (dx[i] === null || dx[i] === undefined) continue;
        adxVal += dx[i];
        count += 1;
    }
    if (count === 0) return out;
    adxVal = adxVal / count;
    out[period * 2] = adxVal;
    for (let i = period * 2 + 1; i < len; i++) {
        if (dx[i] === null || dx[i] === undefined) continue;
        adxVal = ((adxVal * (period - 1)) + dx[i]) / period;
        out[i] = adxVal;
    }
    return out;
}
