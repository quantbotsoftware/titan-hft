//! End-to-end benchmark for the full strategy scoring pipeline.
//!
//! Since the main crate is a binary (not a library), we benchmark the
//! indicator+oracle computation which constitutes >95% of check_entry latency.
//! The remaining score arithmetic and threshold comparison adds ~1-2us.
//!
//! This benchmark exercises the same code path as check_entry:
//!   1. calculate_all_with_periods (EMA, RSI, MACD, BB, ADX, VWAP, StochRSI + full Oracle)
//!   2. The result feeds directly into calculate_score in the app
//!
//! Target: full pipeline < 10ms for 200 candles.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use titan_indicators::indicators::{calculate_all_with_periods, IndicatorPeriods};
use titan_indicators::oracle::{calculate as oracle_calculate, OracleWeights};
use titan_types::Candle;

fn generate_realistic_candles(n: usize) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(n);
    let mut price = 185.0; // AAPL-like
    let mut time = 1_711_700_000i64;

    for i in 0..n {
        let trend = ((i as f64) * 0.05).sin() * 2.0;
        let vol_cycle = ((i as f64) * 0.02).cos() * 0.5;
        let noise = ((i * 7 + 13) % 100) as f64 * 0.03 - 1.5;

        price += trend * 0.1 + noise * 0.2;
        price = price.max(100.0).min(300.0);

        let intraday_vol = 1.0 + vol_cycle.abs();
        let spread = price * 0.003 * intraday_vol;
        let open = price - spread * 0.2;
        let high = price + spread * (1.0 + noise.abs() * 0.1);
        let low = price - spread * (1.0 + noise.abs() * 0.1);
        let close = price + noise * 0.05;

        let base_vol = 50000.0 + ((i * 31 + 7) % 800) as f64 * 200.0;
        let volume = base_vol * intraday_vol;

        candles.push(Candle {
            time,
            open,
            high: high.max(open).max(close),
            low: low.min(open).min(close),
            close,
            volume,
        });
        time += 60;
    }
    candles
}

fn bench_full_pipeline_200(c: &mut Criterion) {
    let candles = generate_realistic_candles(200);
    let periods = IndicatorPeriods::default();
    let weights = OracleWeights::default();

    c.bench_function("full_pipeline_200", |b| {
        b.iter(|| {
            calculate_all_with_periods(
                black_box(&candles),
                black_box(&periods),
                black_box(&weights),
            )
        })
    });
}

fn bench_oracle_isolated_200(c: &mut Criterion) {
    let candles = generate_realistic_candles(200);
    let weights = OracleWeights::default();

    c.bench_function("oracle_isolated_200", |b| {
        b.iter(|| oracle_calculate(black_box(&candles), black_box(&weights)))
    });
}

fn bench_full_pipeline_50(c: &mut Criterion) {
    let candles = generate_realistic_candles(50);
    let periods = IndicatorPeriods::default();
    let weights = OracleWeights::default();

    c.bench_function("full_pipeline_50", |b| {
        b.iter(|| {
            calculate_all_with_periods(
                black_box(&candles),
                black_box(&periods),
                black_box(&weights),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_full_pipeline_200,
    bench_oracle_isolated_200,
    bench_full_pipeline_50
);
criterion_main!(benches);
