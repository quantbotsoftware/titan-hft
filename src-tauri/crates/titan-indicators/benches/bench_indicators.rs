use criterion::{black_box, criterion_group, criterion_main, Criterion};
use titan_indicators::indicators::{calculate_all_with_periods, IndicatorPeriods};
use titan_indicators::oracle::OracleWeights;
use titan_types::Candle;

fn generate_candles(n: usize) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(n);
    let mut price = 150.0;
    let mut time = 1_700_000_000i64;

    for i in 0..n {
        let phase = (i as f64) * 0.1;
        let trend = (phase * 0.3).sin() * 5.0;
        let noise = ((i * 7 + 13) % 100) as f64 * 0.04 - 2.0;
        price += trend * 0.1 + noise * 0.3;
        price = price.max(50.0);

        let spread = price * 0.005;
        let open = price - spread * 0.3;
        let high = price + spread;
        let low = price - spread;
        let close = price + noise * 0.1;
        let volume = 10000.0 + ((i * 31 + 7) % 500) as f64 * 100.0;

        candles.push(Candle {
            time,
            open,
            high,
            low,
            close,
            volume,
        });
        time += 60;
    }
    candles
}

fn bench_all_indicators_200(c: &mut Criterion) {
    let candles = generate_candles(200);
    let periods = IndicatorPeriods::default();
    let weights = OracleWeights::default();

    c.bench_function("indicators_all_200", |b| {
        b.iter(|| {
            calculate_all_with_periods(
                black_box(&candles),
                black_box(&periods),
                black_box(&weights),
            )
        })
    });
}

fn bench_all_indicators_100(c: &mut Criterion) {
    let candles = generate_candles(100);
    let periods = IndicatorPeriods::default();
    let weights = OracleWeights::default();

    c.bench_function("indicators_all_100", |b| {
        b.iter(|| {
            calculate_all_with_periods(
                black_box(&candles),
                black_box(&periods),
                black_box(&weights),
            )
        })
    });
}

criterion_group!(benches, bench_all_indicators_200, bench_all_indicators_100);
criterion_main!(benches);
