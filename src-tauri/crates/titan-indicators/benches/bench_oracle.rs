use criterion::{black_box, criterion_group, criterion_main, Criterion};
use titan_indicators::oracle::{calculate, OracleWeights};
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

fn bench_oracle_200(c: &mut Criterion) {
    let candles = generate_candles(200);
    let weights = OracleWeights::default();

    c.bench_function("oracle_calculate_200", |b| {
        b.iter(|| calculate(black_box(&candles), black_box(&weights)))
    });
}

fn bench_oracle_100(c: &mut Criterion) {
    let candles = generate_candles(100);
    let weights = OracleWeights::default();

    c.bench_function("oracle_calculate_100", |b| {
        b.iter(|| calculate(black_box(&candles), black_box(&weights)))
    });
}

fn bench_oracle_50(c: &mut Criterion) {
    let candles = generate_candles(50);
    let weights = OracleWeights::default();

    c.bench_function("oracle_calculate_50", |b| {
        b.iter(|| calculate(black_box(&candles), black_box(&weights)))
    });
}

criterion_group!(benches, bench_oracle_200, bench_oracle_100, bench_oracle_50);
criterion_main!(benches);
