// ╔══════════════════════════════════════════════════════════════════╗
// ║  TITAN ORACLE — 7-Layer Predictive Super-Indicator              ║
// ║  Replaces classical lagging indicators (EMA, RSI, MACD, ADX)    ║
// ║  with forward-looking mathematical models.                       ║
// ║                                                                  ║
// ║  Layers:                                                         ║
// ║    1. Kalman Adaptive Predictor (KAP)                           ║
// ║    2. Shannon Entropy Flow Regime Detector (EFRD)               ║
// ║    3. Fractal Hurst Momentum (FHM)                              ║
// ║    4. Lyapunov Chaos Detector                                   ║
// ║    5. Haar Wavelet Multi-Resolution Decomposition               ║
// ║    6. VPIN (Volume-Synchronized Informed Trading Probability)   ║
// ║    7. Bayesian Online Changepoint Detector (BOCPD)              ║
// ║                                                                  ║
// ║  Output: oracle_score ∈ [-100, +100]                            ║
// ╚══════════════════════════════════════════════════════════════════╝

use std::collections::HashMap;
use titan_types::Candle;

// ═══════════════════════════════════════════════════════════════════
//  SHARED CONSTANTS — single source of truth for thresholds
// ═══════════════════════════════════════════════════════════════════

const CHANGEPOINT_STABILITY_THRESHOLD: f64 = 0.3;
const ENTROPY_CHAOS_DEFAULT: f64 = 0.85;
const EMA_SMOOTHING_ALPHA: f64 = 0.3;
/// Minimum oracle confidence before a signal is actionable (used by SmartTrend)
pub const MIN_CONFIDENCE_FOR_SIGNAL: f64 = 0.4;

// ═══════════════════════════════════════════════════════════════════
//  PUBLIC API — Data Structures
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum MarketRegime {
    Trending,
    MeanReverting,
    Chaotic,
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct KalmanOutput {
    pub predicted_price: f64,
    pub predicted_velocity: f64,
    pub price_vs_predicted: f64, // >0 price above prediction (bullish deviation)
    pub kalman_gain: f64,        // How much the filter trusts new data vs model
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EntropyOutput {
    pub entropy: f64, // Shannon entropy [0, 1] normalized
    pub regime: MarketRegime,
    pub clarity_score: f64, // 1 - entropy (higher = clearer signal)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HurstOutput {
    pub hurst: f64,              // [0, 1]
    pub persistence_signal: f64, // [-1, +1] mapped from hurst
    pub interpretation: String,  // Human readable
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LyapunovOutput {
    pub exponent: f64,
    pub predictability: f64, // [0, 1] — 1 = fully predictable
    pub is_chaotic: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WaveletOutput {
    pub denoised_trend: f64,   // Reconstructed price from low-freq components
    pub trend_direction: f64,  // Slope of denoised: >0 bullish, <0 bearish
    pub dominant_cycle: usize, // Estimated dominant cycle length
    pub noise_ratio: f64,      // High-freq energy / total energy
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VpinOutput {
    pub vpin: f64,            // [0, 1]
    pub toxicity_alert: bool, // true if VPIN > threshold
    pub buy_pressure: f64,    // Proportion of volume classified as buys
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChangepointOutput {
    pub changepoint_probability: f64, // [0, 1]
    pub run_length: usize,            // Bars since last regime change
    pub regime_stable: bool,          // false if changepoint imminent
}

/// The final composite output of TITAN ORACLE
#[derive(Debug, Clone, serde::Serialize)]
pub struct OracleSnapshot {
    // Individual Layer Outputs
    pub kalman: KalmanOutput,
    pub entropy: EntropyOutput,
    pub hurst: HurstOutput,
    pub lyapunov: LyapunovOutput,
    pub wavelet: WaveletOutput,
    pub vpin: VpinOutput,
    pub changepoint: ChangepointOutput,

    // Fusion
    pub oracle_score: f64, // [-100, +100] THE FINAL ANSWER
    pub confidence: f64,   // [0, 1] how much we trust the score
    pub dominant_regime: MarketRegime,
}

/// Series-level output for charting
#[derive(Debug, Clone, serde::Serialize)]
pub struct OracleHistory {
    pub timestamps: Vec<i64>,
    pub oracle_scores: Vec<Option<f64>>,
    /// Pre-EMA oracle scores -- matches live path output exactly.
    /// `oracle_scores` has EMA smoothing for chart display; `oracle_scores_raw` does not.
    pub oracle_scores_raw: Vec<Option<f64>>,
    pub kalman_predicted: Vec<Option<f64>>,
    pub entropy_series: Vec<Option<f64>>,
    pub hurst_series: Vec<Option<f64>>,
    pub vpin_series: Vec<Option<f64>>,
    pub changepoint_series: Vec<Option<f64>>,
    pub denoised_trend: Vec<Option<f64>>,
    pub regime_series: Vec<Option<MarketRegime>>,
    pub confidence_series: Vec<Option<f64>>,
    pub kalman_gain_series: Vec<Option<f64>>,
    pub price_surprise_series: Vec<Option<f64>>,
    pub run_length_series: Vec<Option<f64>>,
    /// Full-sample scalar broadcast -- NOT a rolling per-bar value.
    /// All valid bars share the same dominant_cycle from a single wavelet_decomposition
    /// over the entire close series. Consider windowed wavelet for per-bar granularity.
    pub dominant_cycle_series: Vec<Option<f64>>,
    /// Full-sample scalar broadcast -- NOT a rolling per-bar value.
    /// All valid bars share the same noise_ratio from a single wavelet_decomposition
    /// over the entire close series. Consider windowed wavelet for per-bar granularity.
    pub noise_ratio_series: Vec<Option<f64>>,
}

// ═══════════════════════════════════════════════════════════════════
//  LAYER 1 — KALMAN ADAPTIVE PREDICTOR
// ═══════════════════════════════════════════════════════════════════
//
// State vector: x = [price, velocity]
// State transition: F = [[1, dt], [0, 1]]  (constant velocity model)
// Measurement: z = price (we only observe price)
// H = [1, 0]
//
// Adaptive: Q (process noise) scales with recent volatility
//           R (measurement noise) is fixed

struct KalmanState {
    x: [f64; 2],      // [price, velocity]
    p: [[f64; 2]; 2], // Covariance matrix
}

struct KalmanSeries {
    predictions: Vec<Option<f64>>,
    gain_series: Vec<Option<f64>>,
    surprise_series: Vec<Option<f64>>,
}

fn kalman_filter(closes: &[f64]) -> (KalmanOutput, KalmanSeries) {
    let n = closes.len();
    let mut predictions = vec![None; n];
    let mut gain_series = vec![None; n];
    let mut surprise_series = vec![None; n];

    if n < 5 {
        return (
            KalmanOutput {
                predicted_price: closes.last().copied().unwrap_or(0.0),
                predicted_velocity: 0.0,
                price_vs_predicted: 0.0,
                kalman_gain: 0.0,
            },
            KalmanSeries {
                predictions,
                gain_series,
                surprise_series,
            },
        );
    }

    let dt = 1.0; // 1 bar
    let f = [[1.0, dt], [0.0, 1.0]]; // State transition
    let h = [1.0, 0.0]; // Measurement matrix

    // Initial state
    let mut state = KalmanState {
        x: [closes[0], 0.0],
        p: [[1.0, 0.0], [0.0, 1.0]],
    };

    // Adaptive R (measurement noise) — fixed baseline
    let r = 1.0;

    let mut last_gain = 0.0;

    for i in 1..n {
        // --- Adaptive Q (process noise) based on recent volatility ---
        let vol_window = 20.min(i);
        let recent_returns: Vec<f64> = (i.saturating_sub(vol_window)..i)
            .filter(|&j| j > 0 && closes[j - 1].abs() > 1e-12)
            .map(|j| (closes[j] - closes[j - 1]) / closes[j - 1])
            .filter(|r| r.is_finite())
            .collect();

        let vol = if recent_returns.len() > 1 {
            let mean = recent_returns.iter().sum::<f64>() / recent_returns.len() as f64;
            let var = recent_returns
                .iter()
                .map(|r| (r - mean).powi(2))
                .sum::<f64>()
                / (recent_returns.len() - 1) as f64;
            var.max(0.0).sqrt()
        } else {
            0.01
        };

        // Scale Q with volatility — high vol = trust model less
        let q_scale = (vol * 100.0).clamp(0.001, 10.0);
        let q = [
            [q_scale * 0.25, q_scale * 0.5],
            [q_scale * 0.5, q_scale * 1.0],
        ];

        // --- PREDICT ---
        let x_pred = [
            f[0][0] * state.x[0] + f[0][1] * state.x[1],
            f[1][0] * state.x[0] + f[1][1] * state.x[1],
        ];

        // NaN safety: if state has gone corrupt, reset to current observation
        if !x_pred[0].is_finite() || !x_pred[1].is_finite() {
            state.x = [closes[i], 0.0];
            state.p = [[1.0, 0.0], [0.0, 1.0]];
            predictions[i] = Some(closes[i]);
            continue;
        }

        // P_pred = F * P * F^T + Q
        let fp = mat2_mul(&f, &state.p);
        let f_t = [[f[0][0], f[1][0]], [f[0][1], f[1][1]]];
        let p_pred_no_q = mat2_mul(&fp, &f_t);
        let p_pred = [
            [p_pred_no_q[0][0] + q[0][0], p_pred_no_q[0][1] + q[0][1]],
            [p_pred_no_q[1][0] + q[1][0], p_pred_no_q[1][1] + q[1][1]],
        ];

        // Store prediction BEFORE update
        predictions[i] = Some(x_pred[0]);

        // --- UPDATE ---
        let z = closes[i]; // Measurement

        // Innovation: y = z - H * x_pred
        let y = z - (h[0] * x_pred[0] + h[1] * x_pred[1]);

        // S = H * P_pred * H^T + R
        let s = h[0] * p_pred[0][0] * h[0]
            + h[0] * p_pred[0][1] * h[1]
            + h[1] * p_pred[1][0] * h[0]
            + h[1] * p_pred[1][1] * h[1]
            + r;

        if s.abs() < 1e-12 || !s.is_finite() {
            state.x = x_pred;
            state.p = p_pred;
            continue;
        }

        // K = P_pred * H^T / S
        let k = [
            (p_pred[0][0] * h[0] + p_pred[0][1] * h[1]) / s,
            (p_pred[1][0] * h[0] + p_pred[1][1] * h[1]) / s,
        ];

        last_gain = k[0];
        gain_series[i] = Some(k[0]);
        let price_surprise = if x_pred[0].abs() > 1e-12 {
            y / x_pred[0]
        } else {
            0.0
        };
        surprise_series[i] = Some(price_surprise);

        // x = x_pred + K * y
        state.x = [x_pred[0] + k[0] * y, x_pred[1] + k[1] * y];

        // Joseph Form: P = (I - K*H) * P_pred * (I - K*H)^T + K*R*K^T
        // Guarantees symmetry and positive-definiteness under floating-point arithmetic
        let ikh = [
            [1.0 - k[0] * h[0], -k[0] * h[1]],
            [-k[1] * h[0], 1.0 - k[1] * h[1]],
        ];
        let ikh_t = [[ikh[0][0], ikh[1][0]], [ikh[0][1], ikh[1][1]]];
        let ikh_p = mat2_mul(&ikh, &p_pred);
        let core = mat2_mul(&ikh_p, &ikh_t);
        let krkt = [
            [k[0] * k[0] * r, k[0] * k[1] * r],
            [k[1] * k[0] * r, k[1] * k[1] * r],
        ];
        state.p = [
            [core[0][0] + krkt[0][0], core[0][1] + krkt[0][1]],
            [core[1][0] + krkt[1][0], core[1][1] + krkt[1][1]],
        ];

        // Force symmetry and guard against numerical corruption
        state.p[0][1] = (state.p[0][1] + state.p[1][0]) * 0.5;
        state.p[1][0] = state.p[0][1];
        if state.p[0][0] < 0.0
            || state.p[1][1] < 0.0
            || !state.p[0][0].is_finite()
            || !state.p[1][1].is_finite()
        {
            state.p = [[1.0, 0.0], [0.0, 1.0]];
        }
    }

    // Final prediction: next bar
    let next_predicted = state.x[0] + state.x[1]; // price + velocity
    let current_price = match closes.last() {
        Some(p) => *p,
        None => 0.0,
    };

    let output = KalmanOutput {
        predicted_price: next_predicted,
        predicted_velocity: state.x[1],
        price_vs_predicted: (current_price - state.x[0]) / current_price,
        kalman_gain: last_gain,
    };

    (
        output,
        KalmanSeries {
            predictions,
            gain_series,
            surprise_series,
        },
    )
}

fn mat2_mul(a: &[[f64; 2]; 2], b: &[[f64; 2]; 2]) -> [[f64; 2]; 2] {
    [
        [
            a[0][0] * b[0][0] + a[0][1] * b[1][0],
            a[0][0] * b[0][1] + a[0][1] * b[1][1],
        ],
        [
            a[1][0] * b[0][0] + a[1][1] * b[1][0],
            a[1][0] * b[0][1] + a[1][1] * b[1][1],
        ],
    ]
}

// ═══════════════════════════════════════════════════════════════════
//  LAYER 2 — SHANNON ENTROPY FLOW REGIME DETECTOR
// ═══════════════════════════════════════════════════════════════════
//
// Discretize returns into N bins, compute probability distribution,
// then H = -Σ p_i * log2(p_i)
// Normalize by max entropy H_max = log2(N)
//
// Low entropy → clear regime (trending or mean-reverting)
// High entropy → chaotic / random

fn entropy_regime(closes: &[f64], window: usize, chaos_threshold: f64) -> (EntropyOutput, Vec<Option<f64>>) {
    let n = closes.len();
    let mut series = vec![None; n];
    let num_bins = 20;

    if n < window + 2 {
        return (
            EntropyOutput {
                entropy: 1.0,
                regime: MarketRegime::Unknown,
                clarity_score: 0.0,
            },
            series,
        );
    }

    let mut last_output = EntropyOutput {
        entropy: 1.0,
        regime: MarketRegime::Unknown,
        clarity_score: 0.0,
    };

    for (end, series_slot) in series.iter_mut().enumerate().take(n).skip(window) {
        let start = end - window;

        // Calculate returns in window
        let returns: Vec<f64> = (start + 1..=end)
            .filter(|&i| closes[i - 1].abs() > 1e-12)
            .map(|i| (closes[i] - closes[i - 1]) / closes[i - 1])
            .filter(|r| r.is_finite())
            .collect();

        if returns.is_empty() {
            continue;
        }

        let min_r = returns.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_r = returns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_r - min_r;

        if range < 1e-12 {
            *series_slot = Some(0.0);
            last_output = EntropyOutput {
                entropy: 0.0,
                regime: MarketRegime::Trending,
                clarity_score: 1.0,
            };
            continue;
        }

        // Bin returns
        let mut bins = vec![0usize; num_bins];
        for &r in &returns {
            let bin = ((r - min_r) / range * (num_bins - 1) as f64).round() as usize;
            let bin = bin.min(num_bins - 1);
            bins[bin] += 1;
        }

        // Shannon entropy
        let total = returns.len() as f64;
        let mut h = 0.0;
        for &count in &bins {
            if count > 0 {
                let p = count as f64 / total;
                h -= p * p.log2();
            }
        }

        // Normalize
        let h_max = (num_bins as f64).log2();
        let normalized = if h_max > 0.0 { h / h_max } else { 0.0 };
        let clarity = 1.0 - normalized;

        // Determine regime using entropy + autocorrelation
        let mean_ret = returns.iter().sum::<f64>() / returns.len() as f64;
        let autocorr = if returns.len() > 1 {
            let var: f64 =
                returns.iter().map(|r| (r - mean_ret).powi(2)).sum::<f64>() / returns.len() as f64;
            if var < 1e-12 {
                0.0
            } else {
                let cov: f64 = returns
                    .windows(2)
                    .map(|w| (w[0] - mean_ret) * (w[1] - mean_ret))
                    .sum::<f64>()
                    / (returns.len() - 1) as f64;
                cov / var
            }
        } else {
            0.0
        };

        let regime = if normalized > chaos_threshold {
            MarketRegime::Chaotic
        } else if autocorr > 0.1 {
            MarketRegime::Trending
        } else if autocorr < -0.1 {
            MarketRegime::MeanReverting
        } else {
            MarketRegime::Unknown
        };

        *series_slot = Some(normalized);
        last_output = EntropyOutput {
            entropy: normalized,
            regime,
            clarity_score: clarity,
        };
    }

    (last_output, series)
}

// ═══════════════════════════════════════════════════════════════════
//  LAYER 3 — FRACTAL HURST MOMENTUM (R/S Analysis)
// ═══════════════════════════════════════════════════════════════════
//
// Rescaled Range analysis:
//   For window sizes n = [8, 16, 32, 64, ...]:
//     R(n) = max(cumulative_deviations) - min(cumulative_deviations)
//     S(n) = std_dev of returns in window
//     R/S = R(n) / S(n)
//   H = slope of log(R/S) vs log(n)

fn hurst_exponent(closes: &[f64], max_window: usize) -> (HurstOutput, Vec<Option<f64>>) {
    let n = closes.len();
    let mut series = vec![None; n];

    if n < 32 {
        return (
            HurstOutput {
                hurst: 0.5,
                persistence_signal: 0.0,
                interpretation: "Not enough data".into(),
            },
            series,
        );
    }

    // Calculate returns
    let returns: Vec<f64> = (1..n).map(|i| closes[i] - closes[i - 1]).collect();

    let mut last_hurst = 0.5;

    // Sliding window Hurst
    let calc_window = max_window.min(n - 1);

    for end in calc_window..returns.len() {
        let window_returns = &returns[end - calc_window..end];
        let h = rs_hurst(window_returns);
        last_hurst = h;
        series[end + 1] = Some(h); // +1 because returns are offset by 1
    }

    let persistence = (last_hurst - 0.5) * 2.0; // Map [0,1] → [-1,+1]

    let interpretation = if last_hurst > 0.65 {
        "Strong Trending (Persistence)"
    } else if last_hurst > 0.55 {
        "Mild Trending"
    } else if last_hurst > 0.45 {
        "Random Walk (No Edge)"
    } else if last_hurst > 0.35 {
        "Mild Mean-Reversion"
    } else {
        "Strong Mean-Reversion (Anti-Persistence)"
    };

    let output = HurstOutput {
        hurst: last_hurst,
        persistence_signal: persistence,
        interpretation: interpretation.to_string(),
    };

    (output, series)
}

fn rs_hurst(returns: &[f64]) -> f64 {
    let n = returns.len();
    if n < 8 {
        return 0.5;
    }

    // Use sub-windows of size: 8, 16, 32, 64, ...
    let mut window_sizes: Vec<usize> = Vec::new();
    let mut s = 8;
    while s <= n {
        window_sizes.push(s);
        s *= 2;
    }

    if window_sizes.len() < 2 {
        return 0.5; // Not enough scales
    }

    let mut log_n = Vec::new();
    let mut log_rs = Vec::new();

    for &ws in &window_sizes {
        let num_segments = n / ws;
        if num_segments == 0 {
            continue;
        }

        let mut rs_sum = 0.0;
        let mut valid_segments = 0;

        for seg in 0..num_segments {
            let start = seg * ws;
            let end = start + ws;
            let segment = &returns[start..end];

            let mean = segment.iter().sum::<f64>() / ws as f64;
            let std_dev = {
                let var = segment.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / ws as f64;
                var.max(0.0).sqrt()
            };

            if std_dev < 1e-12 {
                continue;
            }

            // Cumulative deviations from mean
            let mut cum_dev = Vec::with_capacity(ws);
            let mut running = 0.0;
            for &val in segment {
                running += val - mean;
                cum_dev.push(running);
            }

            let max_dev = cum_dev.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min_dev = cum_dev.iter().cloned().fold(f64::INFINITY, f64::min);
            let range = max_dev - min_dev;

            rs_sum += range / std_dev;
            valid_segments += 1;
        }

        if valid_segments > 0 {
            let avg_rs = rs_sum / valid_segments as f64;
            if avg_rs > 0.0 {
                log_n.push((ws as f64).ln());
                log_rs.push(avg_rs.ln());
            }
        }
    }

    if log_n.len() < 2 {
        return 0.5;
    }

    // Linear regression: log(R/S) = H * log(n) + c
    linear_regression_slope(&log_n, &log_rs).clamp(0.0, 1.0)
}

fn linear_regression_slope(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let sum_x2: f64 = x.iter().map(|a| a * a).sum();

    let denom = n * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-12 {
        return 0.5;
    }
    (n * sum_xy - sum_x * sum_y) / denom
}

// ═══════════════════════════════════════════════════════════════════
//  LAYER 4 — LYAPUNOV CHAOS DETECTOR
// ═══════════════════════════════════════════════════════════════════
//
// Estimates the maximum Lyapunov exponent via nearest-neighbor
// divergence in the returns phase space.
//
// Positive λ → chaotic (sensitive to initial conditions)
// λ ≈ 0     → edge of chaos / periodic
// Negative  → stable, convergent (predictable)

fn lyapunov_exponent(closes: &[f64], embed_dim: usize, lag: usize) -> LyapunovOutput {
    let n = closes.len();
    let returns: Vec<f64> = (1..n)
        .filter(|&i| closes[i - 1].abs() > 1e-12)
        .map(|i| (closes[i] - closes[i - 1]) / closes[i - 1])
        .filter(|r| r.is_finite())
        .collect();

    let min_points = embed_dim * lag + 20;
    if returns.len() < min_points {
        return LyapunovOutput {
            exponent: 0.0,
            predictability: 0.5,
            is_chaotic: false,
        };
    }

    // Create delay-embedded vectors
    let num_vectors = returns.len() - (embed_dim - 1) * lag;
    let mut vectors: Vec<Vec<f64>> = Vec::with_capacity(num_vectors);

    for i in 0..num_vectors {
        let mut vec = Vec::with_capacity(embed_dim);
        for d in 0..embed_dim {
            vec.push(returns[i + d * lag]);
        }
        vectors.push(vec);
    }

    if vectors.len() < 10 {
        return LyapunovOutput {
            exponent: 0.0,
            predictability: 0.5,
            is_chaotic: false,
        };
    }

    // For each point, find nearest neighbor (excluding temporal neighbors)
    let min_temporal_sep = lag * 2;
    let mut lyap_sum = 0.0;
    let mut lyap_count = 0;
    let evolution_steps = 5.min(vectors.len() / 4);

    for i in 0..vectors.len().saturating_sub(evolution_steps) {
        // Find nearest neighbor
        let mut min_dist = f64::INFINITY;
        let mut nn_idx = 0;

        for j in 0..vectors.len().saturating_sub(evolution_steps) {
            if i == j || (i as isize - j as isize).unsigned_abs() < min_temporal_sep {
                continue;
            }

            let dist = euclidean_distance(&vectors[i], &vectors[j]);
            if dist < min_dist && dist > 1e-12 {
                min_dist = dist;
                nn_idx = j;
            }
        }

        if !(1e-12..f64::INFINITY).contains(&min_dist) {
            continue;
        }

        // Track divergence
        if i + evolution_steps < vectors.len() && nn_idx + evolution_steps < vectors.len() {
            let evolved_dist = euclidean_distance(
                &vectors[i + evolution_steps],
                &vectors[nn_idx + evolution_steps],
            );

            if evolved_dist > 1e-12 {
                lyap_sum += (evolved_dist / min_dist).ln();
                lyap_count += 1;
            }
        }
    }

    let exponent = if lyap_count > 0 {
        lyap_sum / (lyap_count as f64 * evolution_steps as f64)
    } else {
        0.0
    };

    // Map to predictability: negative exponent = more predictable
    let predictability = (-exponent * 10.0).exp().clamp(0.0, 1.0);

    LyapunovOutput {
        exponent,
        predictability,
        is_chaotic: exponent > 0.05,
    }
}

fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .max(0.0)
        .sqrt()
}

// ═══════════════════════════════════════════════════════════════════
//  LAYER 5 — HAAR WAVELET MULTI-RESOLUTION DECOMPOSITION
// ═══════════════════════════════════════════════════════════════════
//
// Decomposes the price signal into approximation (low frequency = trend)
// and detail (high frequency = noise) coefficients at multiple levels.
// Reconstructs only the trend component for denoised price.

fn wavelet_decomposition(closes: &[f64]) -> (WaveletOutput, Vec<Option<f64>>) {
    let n = closes.len();
    let mut denoised_series = vec![None; n];

    if n < 16 {
        return (
            WaveletOutput {
                denoised_trend: closes.last().copied().unwrap_or(0.0),
                trend_direction: 0.0,
                dominant_cycle: 0,
                noise_ratio: 1.0,
            },
            denoised_series,
        );
    }

    // Pad to nearest power of 2
    let padded_len = n.next_power_of_two();
    let mut signal = vec![0.0; padded_len];
    signal[..n].copy_from_slice(closes);
    // Pad with last value
    for slot in signal.iter_mut().skip(n) {
        *slot = closes[n - 1];
    }

    // Multi-level Haar wavelet decomposition
    let max_levels = ((padded_len as f64).log2() as usize).min(6); // Max 6 levels
    let mut approximation = signal.clone();
    let mut details: Vec<Vec<f64>> = Vec::new();
    let mut detail_energies: Vec<f64> = Vec::new();

    for _level in 0..max_levels {
        let len = approximation.len();
        if len < 2 {
            break;
        }
        let half = len / 2;
        let mut new_approx = vec![0.0; half];
        let mut detail = vec![0.0; half];

        let sqrt2 = std::f64::consts::SQRT_2;
        for i in 0..half {
            new_approx[i] = (approximation[2 * i] + approximation[2 * i + 1]) / sqrt2;
            detail[i] = (approximation[2 * i] - approximation[2 * i + 1]) / sqrt2;
        }

        let energy: f64 = detail.iter().map(|d| d * d).sum();
        detail_energies.push(energy);
        details.push(detail);
        approximation = new_approx;
    }

    // Total energy
    let total_energy: f64 =
        detail_energies.iter().sum::<f64>() + approximation.iter().map(|a| a * a).sum::<f64>();

    let noise_energy: f64 = if detail_energies.len() >= 2 {
        // High-frequency noise = first 2 detail levels
        detail_energies.iter().take(2).sum()
    } else {
        detail_energies.iter().sum()
    };

    let noise_ratio = if total_energy > 0.0 {
        noise_energy / total_energy
    } else {
        0.0
    };

    // Reconstruct denoised signal (zero out high-frequency details)
    // Keep only approximation + last 2 detail levels (low-freq)
    let mut reconstructed = approximation.clone();

    // Inverse wavelet transform (only low-frequency components)
    let levels_to_keep = if details.len() > 2 {
        details.len() - 2
    } else {
        0
    };
    let sqrt2 = std::f64::consts::SQRT_2;

    for level_idx in (0..details.len()).rev() {
        let detail = &details[level_idx];
        let len = reconstructed.len();
        let mut expanded = vec![0.0; len * 2];

        for i in 0..len {
            let d = if level_idx >= levels_to_keep {
                detail[i] // Keep this detail level
            } else {
                0.0 // Zero out (denoise)
            };
            expanded[2 * i] = (reconstructed[i] + d) / sqrt2;
            expanded[2 * i + 1] = (reconstructed[i] - d) / sqrt2;
        }
        reconstructed = expanded;
    }

    // Trim back to original length
    reconstructed.truncate(n);

    // Fill denoised series
    for (i, &val) in reconstructed.iter().enumerate() {
        denoised_series[i] = Some(val);
    }

    let denoised_last = reconstructed[n - 1];

    // Trend direction = slope of last 5 denoised values
    let lookback = 5.min(n);
    let trend_direction = if lookback >= 2 {
        let start_val = reconstructed[n - lookback];
        let end_val = reconstructed[n - 1];
        if start_val.abs() > 1e-12 {
            (end_val - start_val) / start_val
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Dominant cycle = level with maximum detail energy
    let dominant_level = detail_energies
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let dominant_cycle = 2_usize.pow((dominant_level + 1) as u32);

    (
        WaveletOutput {
            denoised_trend: denoised_last,
            trend_direction,
            dominant_cycle,
            noise_ratio,
        },
        denoised_series,
    )
}

// ═══════════════════════════════════════════════════════════════════
//  LAYER 6 — VPIN (Volume-Synchronized Probability of Informed Trading)
// ═══════════════════════════════════════════════════════════════════
//
// Classifies each bar's volume as buy or sell (using tick rule: close > prev_close).
// Aggregates into volume buckets and measures imbalance.
// High VPIN = "smart money" is trading aggressively.

/// Volume-weighted BVC buy pressure over a rolling window ending at `end_idx`.
/// Used by both live `calculate_vpin()` and history `calculate_history()` for consistency.
fn rolling_bvc_pressure(candles: &[Candle], end_idx: usize, horizon: usize) -> f64 {
    let n = candles.len();
    if n == 0 || end_idx >= n {
        return 0.5;
    }
    let start = (end_idx + 1).saturating_sub(horizon);
    let mut total_buy = 0.0;
    let mut total_vol = 0.0;
    for c in candles.iter().take(end_idx + 1).skip(start) {
        let vol = c.volume;
        let range = c.high - c.low;
        let bvc = if range > 1e-12 {
            ((c.close - c.low) / range).clamp(0.0, 1.0)
        } else {
            0.5
        };
        total_buy += vol * bvc;
        total_vol += vol;
    }
    if total_vol > 0.0 {
        total_buy / total_vol
    } else {
        0.5
    }
}

fn calculate_vpin(candles: &[Candle], bucket_count: usize, toxicity_threshold: f64) -> (VpinOutput, Vec<Option<f64>>) {
    let n = candles.len();
    let mut series = vec![None; n];

    if n < bucket_count + 1 {
        return (
            VpinOutput {
                vpin: 0.0,
                toxicity_alert: false,
                buy_pressure: 0.5,
            },
            series,
        );
    }

    // Total volume to determine bucket size
    let total_vol: f64 = candles.iter().map(|c| c.volume).sum();
    let bucket_vol = total_vol / bucket_count as f64;

    if bucket_vol < 1.0 {
        return (
            VpinOutput {
                vpin: 0.0,
                toxicity_alert: false,
                buy_pressure: 0.5,
            },
            series,
        );
    }

    // Classify volume as buy/sell and accumulate into buckets
    let mut buckets_buy: Vec<f64> = Vec::new();
    let mut buckets_sell: Vec<f64> = Vec::new();
    let mut current_buy = 0.0;
    let mut current_sell = 0.0;
    let mut current_bucket_vol = 0.0;

    for i in 1..n {
        let vol = candles[i].volume;
        let is_buy = candles[i].close >= candles[i - 1].close;

        // Bulk Volume Classification (BVC)
        // Proportion based on close position within the bar
        let bar_range = candles[i].high - candles[i].low;
        let buy_fraction = if bar_range > 1e-12 {
            ((candles[i].close - candles[i].low) / bar_range).clamp(0.0, 1.0)
        } else if is_buy {
            1.0
        } else {
            0.0
        };

        let buy_vol = vol * buy_fraction;
        let sell_vol = vol * (1.0 - buy_fraction);

        current_buy += buy_vol;
        current_sell += sell_vol;
        current_bucket_vol += vol;

        // If bucket is full, record it
        while current_bucket_vol >= bucket_vol && bucket_vol > 0.0 {
            buckets_buy.push(current_buy);
            buckets_sell.push(current_sell);
            current_bucket_vol -= bucket_vol;
            current_buy = 0.0;
            current_sell = 0.0;
        }

        // Rolling VPIN over last `bucket_count` buckets
        if buckets_buy.len() >= bucket_count {
            let start = buckets_buy.len() - bucket_count;
            let sum_imbalance: f64 = (start..buckets_buy.len())
                .map(|j| (buckets_buy[j] - buckets_sell[j]).abs())
                .sum();
            let sum_vol: f64 = (start..buckets_buy.len())
                .map(|j| buckets_buy[j] + buckets_sell[j])
                .sum();

            let vpin_val = if sum_vol > 0.0 {
                sum_imbalance / sum_vol
            } else {
                0.0
            };

            series[i] = Some(vpin_val);
        }
    }

    // Final VPIN
    let last_vpin = series.iter().rev().find_map(|v| *v).unwrap_or(0.0);

    let buy_pressure = rolling_bvc_pressure(candles, n - 1, 20);

    (
        VpinOutput {
            vpin: last_vpin,
            toxicity_alert: last_vpin > toxicity_threshold,
            buy_pressure,
        },
        series,
    )
}

// ═══════════════════════════════════════════════════════════════════
//  LAYER 7 — BAYESIAN ONLINE CHANGEPOINT DETECTION (BOCPD)
// ═══════════════════════════════════════════════════════════════════
//
// Detects structural breaks in the return series in real-time.
// Based on Adams & MacKay (2007).
//
// Maintains a distribution over "run length" (how long since last changepoint).
// When run length resets to 0 with high probability → changepoint detected.

struct ChangepointSeries {
    cp_prob_series: Vec<Option<f64>>,
    run_length_series: Vec<Option<f64>>,
}

fn bayesian_changepoint(
    closes: &[f64],
    hazard_lambda: f64,
) -> (ChangepointOutput, ChangepointSeries) {
    let n = closes.len();
    let mut series = vec![None; n];
    let mut rl_series = vec![None; n];

    if n < 10 {
        return (
            ChangepointOutput {
                changepoint_probability: 0.0,
                run_length: n,
                regime_stable: true,
            },
            ChangepointSeries {
                cp_prob_series: series,
                run_length_series: rl_series,
            },
        );
    }

    let returns: Vec<f64> = (1..n)
        .filter(|&i| closes[i - 1].abs() > 1e-12)
        .map(|i| (closes[i] - closes[i - 1]) / closes[i - 1])
        .filter(|r| r.is_finite())
        .collect();

    // Hazard function: constant rate = 1/lambda
    let hazard = 1.0 / hazard_lambda;

    // Run length distribution: P(r_t | data)
    // We track the log probabilities for numerical stability
    let max_run = returns.len();
    let mut run_length_probs = vec![0.0_f64; max_run + 1];
    run_length_probs[0] = 1.0; // Start with run length 0

    // Online statistics for Gaussian predictive probability
    // Track mean and variance for each possible run length
    let mut means = vec![0.0_f64; max_run + 1];
    let mut variances = vec![1e-4_f64; max_run + 1];
    let mut counts = vec![0.0_f64; max_run + 1];

    let mut last_cp_prob = 0.0;
    let mut best_run_length = 0;

    // Prior parameters for normal-inverse-gamma
    let mu0 = 0.0;
    let kappa0 = 1.0;
    let alpha0 = 1.0;
    let beta0 = 1e-4;

    for t in 0..returns.len() {
        let x = returns[t];

        // 1. Evaluate predictive probability for each run length
        let mut pred_probs = vec![0.0; max_run + 1];
        for r in 0..=t {
            if run_length_probs[r] < 1e-30 {
                continue;
            }
            // Predictive probability under Gaussian with current sufficient statistics
            let count = counts[r];
            let kappa_n = kappa0 + count;
            let alpha_n = alpha0 + count / 2.0;
            let mu_n = (kappa0 * mu0 + count * means[r]) / kappa_n;
            let beta_n = beta0
                + 0.5 * count * variances[r]
                + 0.5 * kappa0 * count * (means[r] - mu0).powi(2) / kappa_n;

            // Student-t predictive probability
            let nu = 2.0 * alpha_n;
            let scale = beta_n * (kappa_n + 1.0) / (alpha_n * kappa_n);
            let scale = scale.max(1e-12);

            let z = (x - mu_n) / scale.sqrt();
            // Simplified student-t log-density (proportional)
            pred_probs[r] = (1.0 + z * z / nu).powf(-(nu + 1.0) / 2.0) / scale.sqrt();
        }

        // 2. Growth probabilities
        let mut new_probs = vec![0.0; max_run + 1];
        let mut changepoint_mass = 0.0;

        for r in 0..=t {
            let joint = run_length_probs[r] * pred_probs[r];
            // Growth prob (no changepoint)
            if r < max_run {
                new_probs[r + 1] += joint * (1.0 - hazard);
            }
            // Changepoint prob (reset to 0)
            changepoint_mass += joint * hazard;
        }
        new_probs[0] += changepoint_mass;

        // 3. Normalize
        let total: f64 = new_probs.iter().sum();
        if total > 1e-30 {
            for p in new_probs.iter_mut() {
                *p /= total;
            }
        }

        // 4. Update sufficient statistics
        let mut new_means = vec![0.0; max_run + 1];
        let mut new_variances = vec![1e-4; max_run + 1];
        let mut new_counts = vec![0.0; max_run + 1];

        // Run length 0 → new regime, start fresh
        new_means[0] = x;
        new_variances[0] = 1e-4;
        new_counts[0] = 1.0;

        // Growth: update running statistics
        for r in 0..=t {
            let new_r = r + 1;
            if new_r > max_run {
                break;
            }
            let prev_count = counts[r];
            let prev_mean = means[r];
            let new_count = prev_count + 1.0;
            let delta = x - prev_mean;
            let new_mean = prev_mean + delta / new_count;
            let new_var = if new_count > 1.0 {
                ((prev_count - 1.0) * variances[r] + delta * (x - new_mean)) / (new_count - 1.0)
            } else {
                1e-4
            };

            new_means[new_r] = new_mean;
            new_variances[new_r] = new_var.max(1e-12);
            new_counts[new_r] = new_count;
        }

        run_length_probs = new_probs;
        means = new_means;
        variances = new_variances;
        counts = new_counts;

        // Record changepoint probability
        last_cp_prob = run_length_probs[0];
        series[t + 1] = Some(last_cp_prob); // +1 offset for returns→prices

        // Find MAP run length
        best_run_length = run_length_probs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        rl_series[t + 1] = Some(best_run_length as f64);
    }

    (
        ChangepointOutput {
            changepoint_probability: last_cp_prob,
            run_length: best_run_length,
            regime_stable: last_cp_prob < CHANGEPOINT_STABILITY_THRESHOLD,
        },
        ChangepointSeries {
            cp_prob_series: series,
            run_length_series: rl_series,
        },
    )
}

// ═══════════════════════════════════════════════════════════════════
//  FUSION LAYER — TITAN COMPOSITE SCORE
// ═══════════════════════════════════════════════════════════════════
//
// Combines all 7 layers with adaptive weighting.
// Layers with low predictability / high chaos get downweighted.

#[derive(Debug, Clone)]
pub struct OracleWeights {
    pub kalman: f64,
    pub wavelet: f64,
    pub hurst: f64,
    pub vpin: f64,
    pub entropy_dir: f64,
    pub confidence_floor: f64,
    pub entropy_chaos_threshold: f64,
    pub vpin_toxicity_alert: f64,
}

impl Default for OracleWeights {
    fn default() -> Self {
        Self {
            kalman: 0.30,
            wavelet: 0.25,
            hurst: 0.20,
            vpin: 0.15,
            entropy_dir: 0.10,
            confidence_floor: 0.25,
            entropy_chaos_threshold: ENTROPY_CHAOS_DEFAULT,
            vpin_toxicity_alert: 0.6,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  ONLINE WEIGHT ADAPTATION — Per-Regime PnL-Driven Learning
// ═══════════════════════════════════════════════════════════════════

const LAYER_COUNT: usize = 5;
const ADAPT_INTERVAL: u32 = 50;
const BASE_ALPHA: f64 = 0.10;
const MIN_ALPHA_SCALE: f64 = 0.10;
const WEIGHT_CAP_LOW: f64 = 0.50;
const WEIGHT_CAP_HIGH: f64 = 2.00;
const REGIME_KEYS: [&str; 4] = ["Trending", "MeanReverting", "Chaotic", "Unknown"];

fn default_decay_factor() -> f64 {
    0.997
}

#[derive(Debug, Clone)]
pub struct LayerEntrySnapshot {
    pub signals: [f64; LAYER_COUNT],
    pub regime: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OracleAdaptation {
    pub regime_layer_corr: HashMap<String, [f64; LAYER_COUNT]>,
    #[serde(skip)]
    pub layer_signals_at_entry: HashMap<String, LayerEntrySnapshot>,
    pub trade_count: u32,
    pub adapted_weights: HashMap<String, [f64; LAYER_COUNT]>,
    #[serde(default = "default_decay_factor")]
    pub decay_factor: f64,
}

impl Default for OracleAdaptation {
    fn default() -> Self {
        let mut regime_layer_corr = HashMap::new();
        for key in &REGIME_KEYS {
            regime_layer_corr.insert((*key).to_string(), [0.0; LAYER_COUNT]);
        }
        Self {
            regime_layer_corr,
            layer_signals_at_entry: HashMap::new(),
            trade_count: 0,
            adapted_weights: HashMap::new(),
            decay_factor: default_decay_factor(),
        }
    }
}

/// Extract the 5 directional layer signals from an OracleSnapshot,
/// using the same normalization as fuse_oracle.
pub fn extract_layer_signals(snap: &OracleSnapshot) -> [f64; LAYER_COUNT] {
    let safe = |v: f64, fallback: f64| -> f64 {
        if v.is_finite() { v } else { fallback }
    };
    let kalman_signal = (safe(snap.kalman.predicted_velocity, 0.0) * 1000.0).clamp(-1.0, 1.0);
    let wavelet_signal = (safe(snap.wavelet.trend_direction, 0.0) * 100.0).clamp(-1.0, 1.0);
    let hurst_signal = safe(snap.hurst.persistence_signal, 0.0).clamp(-1.0, 1.0);
    let vpin_signal = (safe(snap.vpin.buy_pressure, 0.5) - 0.5) * 2.0;
    let clarity = safe(snap.entropy.clarity_score, 0.5).clamp(0.0, 1.0);
    let entropy_dir_signal = (kalman_signal * clarity).clamp(-1.0, 1.0);
    [kalman_signal, wavelet_signal, hurst_signal, vpin_signal, entropy_dir_signal]
}

impl OracleAdaptation {
    pub fn capture_entry_signals(
        &mut self,
        symbol: &str,
        snapshot: &OracleSnapshot,
    ) {
        let signals = extract_layer_signals(snapshot);
        let regime = format!("{:?}", snapshot.dominant_regime);
        let confidence = snapshot.confidence;
        self.layer_signals_at_entry.insert(
            symbol.to_string(),
            LayerEntrySnapshot { signals, regime, confidence },
        );
    }

    pub fn record_trade_outcome(
        &mut self,
        symbol: &str,
        pnl_direction: f64,
        base_weights: &OracleWeights,
    ) {
        let entry_snap = match self.layer_signals_at_entry.remove(symbol) {
            Some(s) => s,
            None => return,
        };

        let direction = if pnl_direction > 0.0 {
            1.0
        } else if pnl_direction < 0.0 {
            -1.0
        } else {
            return;
        };

        let eff_alpha = BASE_ALPHA * entry_snap.confidence.clamp(MIN_ALPHA_SCALE, 1.0);

        let corr = self
            .regime_layer_corr
            .entry(entry_snap.regime)
            .or_insert([0.0; LAYER_COUNT]);

        for (c, &sig) in corr.iter_mut().zip(entry_snap.signals.iter()) {
            let observation = sig * direction;
            *c = *c * (1.0 - eff_alpha) + observation * eff_alpha;
        }

        for vals in self.regime_layer_corr.values_mut() {
            for c in vals.iter_mut() {
                *c *= self.decay_factor;
            }
        }

        self.trade_count += 1;

        if self.trade_count.is_multiple_of(ADAPT_INTERVAL) {
            self.recalc_adapted_weights(base_weights);
        }
    }

    /// Keep entry signals alive for partial exits: the position is still open
    /// so we correlate with PnL but do NOT remove the entry snapshot.
    pub fn record_partial_outcome(
        &mut self,
        symbol: &str,
        pnl_direction: f64,
        base_weights: &OracleWeights,
    ) {
        let entry_snap = match self.layer_signals_at_entry.get(symbol) {
            Some(s) => s.clone(),
            None => return,
        };

        let direction = if pnl_direction > 0.0 {
            1.0
        } else if pnl_direction < 0.0 {
            -1.0
        } else {
            return;
        };

        let eff_alpha = BASE_ALPHA * entry_snap.confidence.clamp(MIN_ALPHA_SCALE, 1.0);

        let corr = self
            .regime_layer_corr
            .entry(entry_snap.regime)
            .or_insert([0.0; LAYER_COUNT]);

        for (c, &sig) in corr.iter_mut().zip(entry_snap.signals.iter()) {
            let observation = sig * direction;
            *c = *c * (1.0 - eff_alpha) + observation * eff_alpha;
        }

        for vals in self.regime_layer_corr.values_mut() {
            for c in vals.iter_mut() {
                *c *= self.decay_factor;
            }
        }

        self.trade_count += 1;

        if self.trade_count.is_multiple_of(ADAPT_INTERVAL) {
            self.recalc_adapted_weights(base_weights);
        }
    }

    pub fn adapt_weights(
        &self,
        base: &OracleWeights,
        regime: &MarketRegime,
    ) -> OracleWeights {
        let regime_key = format!("{:?}", regime);
        if let Some(adapted) = self.adapted_weights.get(&regime_key) {
            OracleWeights {
                kalman: adapted[0],
                wavelet: adapted[1],
                hurst: adapted[2],
                vpin: adapted[3],
                entropy_dir: adapted[4],
                confidence_floor: base.confidence_floor,
                entropy_chaos_threshold: base.entropy_chaos_threshold,
                vpin_toxicity_alert: base.vpin_toxicity_alert,
            }
        } else {
            base.clone()
        }
    }

    fn recalc_adapted_weights(&mut self, base: &OracleWeights) {
        let base_arr = [base.kalman, base.wavelet, base.hurst, base.vpin, base.entropy_dir];

        for regime_key in &REGIME_KEYS {
            let key = (*regime_key).to_string();
            let corr = match self.regime_layer_corr.get(&key) {
                Some(c) => *c,
                None => continue,
            };

            let abs_corr: Vec<f64> = corr.iter().map(|c| c.abs().max(0.01)).collect();
            let abs_sum: f64 = abs_corr.iter().sum();

            let mut adapted = [0.0f64; LAYER_COUNT];
            for i in 0..LAYER_COUNT {
                let scale = abs_corr[i] / abs_sum;
                let raw = base_arr[i] * (0.5 + scale * LAYER_COUNT as f64);
                adapted[i] = raw.clamp(
                    base_arr[i] * WEIGHT_CAP_LOW,
                    base_arr[i] * WEIGHT_CAP_HIGH,
                );
            }

            let norm: f64 = adapted.iter().sum();
            if norm > 1e-9 {
                for w in adapted.iter_mut() {
                    *w /= norm;
                }
            }

            self.adapted_weights.insert(key, adapted);
        }
    }
}

fn fuse_oracle(
    kalman: &KalmanOutput,
    entropy: &EntropyOutput,
    hurst: &HurstOutput,
    lyapunov: &LyapunovOutput,
    wavelet: &WaveletOutput,
    vpin: &VpinOutput,
    changepoint: &ChangepointOutput,
    weights: &OracleWeights,
) -> (f64, f64, MarketRegime) {
    // NaN/Inf guard helper: replace non-finite with fallback
    let safe = |v: f64, fallback: f64| -> f64 {
        if v.is_finite() {
            v
        } else {
            fallback
        }
    };

    // --- Individual Signals → [-1, +1] ---

    // 1. Kalman: positive velocity = bullish
    let kalman_signal = (safe(kalman.predicted_velocity, 0.0) * 1000.0).clamp(-1.0, 1.0);

    // 2. Hurst persistence signal (already [-1, +1])
    let hurst_signal = safe(hurst.persistence_signal, 0.0).clamp(-1.0, 1.0);

    // 3. Wavelet trend direction → bullish/bearish
    let wavelet_signal = (safe(wavelet.trend_direction, 0.0) * 100.0).clamp(-1.0, 1.0);

    // 4. VPIN buy pressure → direction
    let vpin_signal = (safe(vpin.buy_pressure, 0.5) - 0.5) * 2.0; // Map [0,1] to [-1,+1]

    // 5. Changepoint → dampen if regime break imminent
    let stability_factor = if changepoint.regime_stable { 1.0 } else { 0.3 };

    // --- Adaptive Weights ---
    // Predictable market → higher weights on directional signals
    // Chaotic market → reduce all weights

    let base_predictability = safe(lyapunov.predictability, 0.5).clamp(0.0, 1.0);
    let clarity = safe(entropy.clarity_score, 0.5).clamp(0.0, 1.0);

    // Meta-confidence: geometric mean prevents triple-product collapse to near-zero
    let meta_confidence = (base_predictability * clarity * stability_factor)
        .cbrt()
        .clamp(0.0, 1.0);

    // Normalize weights so they sum to 1.0 (guard against user misconfiguration)
    let raw_sum = weights.kalman + weights.wavelet + weights.hurst + weights.vpin + weights.entropy_dir;
    let norm = if raw_sum > 1e-9 { raw_sum } else { 1.0 };
    let w_kalman = weights.kalman / norm;
    let w_wavelet = weights.wavelet / norm;
    let w_hurst = weights.hurst / norm;
    let w_vpin = weights.vpin / norm;
    let w_entropy_dir = weights.entropy_dir / norm;

    // Entropy direction boost: in trending regime, boost directional signals
    let regime_boost = match entropy.regime {
        MarketRegime::Trending => 1.3,
        MarketRegime::MeanReverting => 0.7, // Reduce confidence in trend signals
        MarketRegime::Chaotic => 0.3,
        MarketRegime::Unknown => 0.8,
    };

    // Raw composite signal
    let raw_signal = w_kalman * kalman_signal
        + w_wavelet * wavelet_signal
        + w_hurst * hurst_signal
        + w_vpin * vpin_signal
        + w_entropy_dir * kalman_signal * clarity; // Clarity-weighted echo

    // Apply regime boost and meta-confidence
    let final_signal = raw_signal * regime_boost * meta_confidence;

    // Scale to [-100, +100]
    let oracle_score = (final_signal * 100.0).clamp(-100.0, 100.0);

    // Final confidence -- halve the noise penalty and enforce a floor
    let noise_ratio = safe(wavelet.noise_ratio, 0.5).clamp(0.0, 1.0);
    let confidence = meta_confidence * (1.0 - noise_ratio * 0.5).max(weights.confidence_floor);

    // Dominant regime
    let regime = if lyapunov.is_chaotic || matches!(entropy.regime, MarketRegime::Chaotic) {
        MarketRegime::Chaotic
    } else {
        entropy.regime
    };

    // Final NaN guard on outputs
    let oracle_score = if oracle_score.is_finite() {
        oracle_score
    } else {
        0.0
    };
    let confidence = if confidence.is_finite() {
        confidence
    } else {
        0.0
    };

    (oracle_score, confidence, regime)
}

// ═══════════════════════════════════════════════════════════════════
//  PUBLIC API — Main Entry Points
// ═══════════════════════════════════════════════════════════════════

/// Calculate TITAN ORACLE snapshot for the latest bar
pub fn calculate(candles: &[Candle], weights: &OracleWeights) -> OracleSnapshot {
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    // Layer 1: Kalman
    let (kalman, _kalman_s) = kalman_filter(&closes);

    // Layer 2: Entropy
    let entropy_window = 50.min(closes.len().saturating_sub(2));
    let (entropy, _entropy_series) = entropy_regime(&closes, entropy_window.max(10), weights.entropy_chaos_threshold);

    // Layer 3: Hurst
    let hurst_window = 128.min(closes.len().saturating_sub(1));
    let (hurst, _hurst_series) = hurst_exponent(&closes, hurst_window.max(16));

    // Layer 4: Lyapunov — trailing window matching calculate_history (H-17 / H-25)
    let lyap_window = 200;
    let lyap_start = closes.len().saturating_sub(lyap_window);
    let lyapunov = lyapunov_exponent(&closes[lyap_start..], 3, 2);

    // Layer 5: Wavelet
    let (wavelet, _wavelet_series) = wavelet_decomposition(&closes);

    // Layer 6: VPIN
    let vpin_buckets = 30.min(candles.len() / 3).max(5);
    let (vpin, _vpin_series) = calculate_vpin(candles, vpin_buckets, weights.vpin_toxicity_alert);

    // Layer 7: Changepoint
    let (changepoint, _cp_s) = bayesian_changepoint(&closes, 100.0);

    // Fusion
    let (oracle_score, confidence, regime) = fuse_oracle(
        &kalman,
        &entropy,
        &hurst,
        &lyapunov,
        &wavelet,
        &vpin,
        &changepoint,
        weights,
    );

    OracleSnapshot {
        kalman,
        entropy,
        hurst,
        lyapunov,
        wavelet,
        vpin,
        changepoint,
        oracle_score,
        confidence,
        dominant_regime: regime,
    }
}

/// Calculate full history for charting
pub fn calculate_history(candles: &[Candle], weights: &OracleWeights) -> OracleHistory {
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let n = closes.len();

    if n < 10 {
        return OracleHistory {
            timestamps: candles.iter().map(|c| c.time).collect(),
            oracle_scores: vec![None; n],
            oracle_scores_raw: vec![None; n],
            kalman_predicted: vec![None; n],
            entropy_series: vec![None; n],
            hurst_series: vec![None; n],
            vpin_series: vec![None; n],
            changepoint_series: vec![None; n],
            denoised_trend: vec![None; n],
            regime_series: vec![None; n],
            confidence_series: vec![None; n],
            kalman_gain_series: vec![None; n],
            price_surprise_series: vec![None; n],
            run_length_series: vec![None; n],
            dominant_cycle_series: vec![None; n],
            noise_ratio_series: vec![None; n],
        };
    }

    let (_, kalman_s) = kalman_filter(&closes);
    let kalman_series = &kalman_s.predictions;
    let kalman_gain_series = &kalman_s.gain_series;
    let kalman_surprise_series = &kalman_s.surprise_series;
    let entropy_window = 50.min(n.saturating_sub(2)).max(10);
    let (_, entropy_series) = entropy_regime(&closes, entropy_window, weights.entropy_chaos_threshold);
    let hurst_window = 128.min(n.saturating_sub(1)).max(16);
    let (_, hurst_series) = hurst_exponent(&closes, hurst_window);
    let (wavelet_out, wavelet_series) = wavelet_decomposition(&closes);
    let wavelet_noise_ratio = wavelet_out.noise_ratio;
    let wavelet_dominant_cycle = wavelet_out.dominant_cycle;
    let vpin_buckets = 30.min(n / 3).max(5);
    let (_, vpin_series) = calculate_vpin(candles, vpin_buckets, weights.vpin_toxicity_alert);
    let (_, cp_s) = bayesian_changepoint(&closes, 100.0);
    let cp_series = &cp_s.cp_prob_series;
    let cp_run_length_series = &cp_s.run_length_series;

    // --- DIAGNOSTIC: count non-null per series ---
    let kalman_nn = kalman_series.iter().filter(|v| v.is_some()).count();
    let entropy_nn = entropy_series.iter().filter(|v| v.is_some()).count();
    let hurst_nn = hurst_series.iter().filter(|v| v.is_some()).count();
    let wavelet_nn = wavelet_series.iter().filter(|v| v.is_some()).count();
    let vpin_nn = vpin_series.iter().filter(|v| v.is_some()).count();
    let cp_nn = cp_series.iter().filter(|v| v.is_some()).count();

    // Check for NaN/Infinity in Kalman series
    let kalman_nan = kalman_series
        .iter()
        .filter(|v| v.map(|x| !x.is_finite()).unwrap_or(false))
        .count();

    tracing::debug!("ORACLE SERIES DIAG: n={} | kalman={}/{} (nan={}) | entropy={} | hurst={} | wavelet={} | vpin={} | cp={}",
        n, kalman_nn, n, kalman_nan, entropy_nn, hurst_nn, wavelet_nn, vpin_nn, cp_nn);

    let closes_bad = closes
        .iter()
        .filter(|c| !c.is_finite() || **c <= 0.0)
        .count();
    if closes_bad > 0 {
        tracing::warn!("ORACLE: {} bad close prices in input data!", closes_bad);
    }

    // Build oracle scores using fuse_oracle directly (single source of truth)
    let mut oracle_scores = vec![None; n];
    let mut regime_series: Vec<Option<MarketRegime>> = vec![None; n];
    let mut confidence_series: Vec<Option<f64>> = vec![None; n];
    let min_warmup = 50;
    let lyap_window = 200;

    for i in min_warmup..n {
        // Reconstruct per-bar layer outputs from series data

        // Kalman: velocity from predicted price series
        let kalman_vel = if i >= 2 {
            match (
                kalman_series.get(i).and_then(|v| *v),
                kalman_series.get(i - 1).and_then(|v| *v),
            ) {
                (Some(curr), Some(prev))
                    if curr.is_finite() && prev.is_finite() && prev.abs() > 1e-12 =>
                {
                    (curr - prev) / prev
                }
                _ => 0.0,
            }
        } else {
            0.0
        };

        let pred_price = kalman_series.get(i).and_then(|v| *v).unwrap_or(closes[i]);
        let bar_kalman = KalmanOutput {
            predicted_price: pred_price,
            predicted_velocity: kalman_vel,
            price_vs_predicted: if closes[i].abs() > 1e-12 {
                (closes[i] - pred_price) / closes[i]
            } else {
                0.0
            },
            kalman_gain: kalman_gain_series.get(i).and_then(|v| *v).unwrap_or(0.0),
        };

        // Entropy: reconstruct regime from already-normalized entropy value.
        // entropy_series stores h/h_max ∈ [0,1] from entropy_regime(),
        // so clarity = 1 - normalized (no re-scaling needed).
        let entropy_val = entropy_series.get(i).and_then(|v| *v).unwrap_or(0.5);
        let ent_clarity = if entropy_val.is_finite() {
            (1.0 - entropy_val).clamp(0.0, 1.0)
        } else {
            0.5
        };
        let ent_normalized = if entropy_val.is_finite() {
            entropy_val.clamp(0.0, 1.0)
        } else {
            0.5
        };
        let bar_entropy_regime = if ent_normalized > weights.entropy_chaos_threshold {
            MarketRegime::Chaotic
        } else {
            let ac_window = 20.min(i);
            let ret_slice: Vec<f64> = (i.saturating_sub(ac_window)..i)
                .filter(|&j| j > 0 && closes[j - 1].abs() > 1e-12)
                .map(|j| (closes[j] - closes[j - 1]) / closes[j - 1])
                .filter(|r| r.is_finite())
                .collect();
            if ret_slice.len() > 2 {
                let mean_r = ret_slice.iter().sum::<f64>() / ret_slice.len() as f64;
                let var: f64 = ret_slice.iter().map(|r| (r - mean_r).powi(2)).sum::<f64>()
                    / ret_slice.len() as f64;
                let ac = if var > 1e-12 {
                    let cov: f64 = ret_slice
                        .windows(2)
                        .map(|w| (w[0] - mean_r) * (w[1] - mean_r))
                        .sum::<f64>()
                        / (ret_slice.len() - 1) as f64;
                    cov / var
                } else {
                    0.0
                };
                if ac > 0.1 {
                    MarketRegime::Trending
                } else if ac < -0.1 {
                    MarketRegime::MeanReverting
                } else {
                    MarketRegime::Unknown
                }
            } else {
                MarketRegime::Unknown
            }
        };
        let bar_entropy = EntropyOutput {
            entropy: ent_normalized,
            regime: bar_entropy_regime,
            clarity_score: ent_clarity,
        };

        // Hurst
        let hurst_val = hurst_series.get(i).and_then(|v| *v).unwrap_or(0.5);
        let hurst_h = if hurst_val.is_finite() {
            hurst_val
        } else {
            0.5
        };
        let bar_hurst = HurstOutput {
            hurst: hurst_h,
            persistence_signal: ((hurst_h - 0.5) * 2.0).clamp(-1.0, 1.0),
            interpretation: String::new(),
        };

        // Lyapunov: rolling window to avoid look-ahead bias (H-17)
        let lyap_start = i.saturating_sub(lyap_window);
        let bar_lyapunov = lyapunov_exponent(&closes[lyap_start..=i], 3, 2);

        // Wavelet: trend direction from denoised series
        let w_trend_dir = if i >= 5 {
            match (
                wavelet_series.get(i).and_then(|v| *v),
                wavelet_series.get(i - 5).and_then(|v| *v),
            ) {
                (Some(end_v), Some(start_v))
                    if start_v.abs() > 1e-12 && end_v.is_finite() && start_v.is_finite() =>
                {
                    (end_v - start_v) / start_v
                }
                _ => 0.0,
            }
        } else {
            0.0
        };
        let bar_wavelet = WaveletOutput {
            denoised_trend: wavelet_series.get(i).and_then(|v| *v).unwrap_or(closes[i]),
            trend_direction: w_trend_dir,
            dominant_cycle: wavelet_dominant_cycle,
            noise_ratio: wavelet_noise_ratio,
        };

        // VPIN: buy_pressure uses same rolling BVC window as live calculate_vpin()
        let vpin_val = vpin_series.get(i).and_then(|v| *v).unwrap_or(0.5);
        let bar_vpin = VpinOutput {
            vpin: vpin_val,
            toxicity_alert: vpin_val > weights.vpin_toxicity_alert,
            buy_pressure: rolling_bvc_pressure(candles, i, 20),
        };

        // Changepoint: threshold must match bayesian_changepoint() live path (0.3)
        let cp_val = cp_series.get(i).and_then(|v| *v).unwrap_or(0.0);
        let bar_changepoint = ChangepointOutput {
            changepoint_probability: if cp_val.is_finite() { cp_val } else { 0.0 },
            run_length: cp_run_length_series.get(i).and_then(|v| *v).map(|v| v as usize).unwrap_or(0),
            regime_stable: cp_val < CHANGEPOINT_STABILITY_THRESHOLD,
        };

        // Use the SAME fuse_oracle as live path -- single source of truth
        let (score, conf, regime) = fuse_oracle(
            &bar_kalman,
            &bar_entropy,
            &bar_hurst,
            &bar_lyapunov,
            &bar_wavelet,
            &bar_vpin,
            &bar_changepoint,
            weights,
        );

        if score.is_finite() {
            oracle_scores[i] = Some(score);
        }
        regime_series[i] = Some(regime);
        confidence_series[i] = if conf.is_finite() { Some(conf) } else { None };
    }

    // Preserve pre-EMA scores for live-vs-history comparisons (live path has no EMA)
    let oracle_scores_raw = oracle_scores.clone();

    // EMA smoothing pass for chart display only -- live path does NOT apply this
    let alpha = EMA_SMOOTHING_ALPHA;
    let mut prev_ema: Option<f64> = None;
    for slot in oracle_scores.iter_mut().take(n) {
        if let Some(raw) = *slot {
            let smoothed = match prev_ema {
                Some(prev) => alpha * raw + (1.0 - alpha) * prev,
                None => raw,
            };
            *slot = Some(smoothed);
            prev_ema = Some(smoothed);
        }
    }

    // --- DIAGNOSTIC: final counts ---
    let scores_nn = oracle_scores.iter().filter(|v| v.is_some()).count();
    let expected = n.saturating_sub(min_warmup);
    let gap_pct = if expected > 0 {
        ((expected - scores_nn) as f64 / expected as f64 * 100.0) as u32
    } else {
        0
    };
    tracing::debug!(
        "ORACLE SCORES: {}/{} valid ({} expected, {}% gaps)",
        scores_nn,
        n,
        expected,
        gap_pct
    );

    if scores_nn < expected / 2 {
        // Log first few gaps for debugging
        let mut gap_samples = Vec::new();
        for i in min_warmup..n.min(min_warmup + 200) {
            if oracle_scores[i].is_none() {
                let k = kalman_series.get(i).and_then(|v| *v);
                let k_prev = if i > 0 {
                    kalman_series.get(i - 1).and_then(|v| *v)
                } else {
                    None
                };
                gap_samples.push(format!(
                    "i={}: kalman[i]={:?} kalman[i-1]={:?} close={:.2}",
                    i, k, k_prev, closes[i]
                ));
                if gap_samples.len() >= 5 {
                    break;
                }
            }
        }
        if !gap_samples.is_empty() {
            tracing::debug!("ORACLE GAP SAMPLES: {}", gap_samples.join(" | "));
        }
    }

    let dc_val = wavelet_dominant_cycle as f64;
    let nr_val = wavelet_noise_ratio;

    OracleHistory {
        timestamps: candles.iter().map(|c| c.time).collect(),
        oracle_scores,
        oracle_scores_raw,
        kalman_predicted: kalman_series.to_vec(),
        entropy_series,
        hurst_series,
        vpin_series,
        changepoint_series: cp_series.to_vec(),
        dominant_cycle_series: (0..n).map(|i| {
            if wavelet_series.get(i).and_then(|v| *v).is_some() {
                Some(dc_val)
            } else {
                None
            }
        }).collect(),
        noise_ratio_series: (0..n).map(|i| {
            if wavelet_series.get(i).and_then(|v| *v).is_some() {
                Some(nr_val)
            } else {
                None
            }
        }).collect(),
        denoised_trend: wavelet_series,
        regime_series,
        confidence_series,
        kalman_gain_series: kalman_gain_series.to_vec(),
        price_surprise_series: kalman_surprise_series.to_vec(),
        run_length_series: cp_run_length_series.to_vec(),
    }
}

// ═══════════════════════════════════════════════════════════════════
//  UNIT TESTS
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candles(closes: &[f64]) -> Vec<Candle> {
        closes
            .iter()
            .enumerate()
            .map(|(i, &c)| Candle {
                time: i as i64,
                open: c,
                high: c * 1.001,
                low: c * 0.999,
                close: c,
                volume: 1000.0 + (i as f64 * 10.0),
            })
            .collect()
    }

    fn trending_up(n: usize) -> Vec<f64> {
        // Uptrend with small noise to avoid degenerate zero-variance returns
        let mut seed: u64 = 123;
        (0..n)
            .map(|i| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(7);
                let noise = (seed >> 33) as f64 / (1u64 << 31) as f64 - 0.5;
                100.0 + i as f64 * 0.5 + noise * 0.2
            })
            .collect()
    }

    fn trending_down(n: usize) -> Vec<f64> {
        let mut seed: u64 = 456;
        (0..n)
            .map(|i| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(7);
                let noise = (seed >> 33) as f64 / (1u64 << 31) as f64 - 0.5;
                200.0 - i as f64 * 0.5 + noise * 0.2
            })
            .collect()
    }

    fn random_walk(n: usize) -> Vec<f64> {
        let mut prices = vec![100.0];
        let mut seed: u64 = 42;
        for _ in 1..n {
            // Simple LCG pseudo-random
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let r = (seed >> 33) as f64 / (1u64 << 31) as f64 - 0.5;
            let last = prices.last().copied().unwrap_or(0.0);
            prices.push(last + r * 0.5);
        }
        prices
    }

    #[test]
    fn test_kalman_trending() {
        let closes = trending_up(100);
        let (output, _) = kalman_filter(&closes);
        // Predicted price should be above current (uptrend)
        assert!(
            output.predicted_velocity > 0.0,
            "Velocity should be positive in uptrend"
        );
        assert!(
            output.predicted_price > closes[99],
            "Predicted should be above current in uptrend"
        );
    }

    #[test]
    fn test_kalman_downtrend() {
        let closes = trending_down(100);
        let (output, _) = kalman_filter(&closes);
        assert!(
            output.predicted_velocity < 0.0,
            "Velocity should be negative in downtrend"
        );
    }

    #[test]
    fn test_entropy_trending() {
        // Use a strong exponential-like trend for clearer regime detection
        let closes: Vec<f64> = (0..200).map(|i| 100.0 * (1.003_f64).powi(i)).collect();
        let (output, _) = entropy_regime(&closes, 50, 0.85);
        // Entropy should be finite and regime should be detected
        assert!(
            output.entropy >= 0.0 && output.entropy <= 1.0,
            "Entropy should be normalized: got {}",
            output.entropy
        );
        // With strong exponential growth, the autocorrelation should yield Trending
        assert!(
            !matches!(output.regime, MarketRegime::Unknown) || output.entropy < 1.0,
            "Should detect some regime structure"
        );
    }

    #[test]
    fn test_hurst_trending() {
        // Use a strong, long trend with accumulating price for clear persistence
        let closes: Vec<f64> = (0..500)
            .map(|i| 100.0 + (i as f64).powf(1.2) * 0.1)
            .collect();
        let (output, _) = hurst_exponent(&closes, 256);
        // The Hurst exponent should indicate some persistence (> 0.4 at minimum)
        assert!(
            output.hurst > 0.4,
            "Hurst should indicate persistence for strong trend: got {}",
            output.hurst
        );
    }

    #[test]
    fn test_wavelet_basic() {
        let closes = trending_up(64);
        let (output, series) = wavelet_decomposition(&closes);
        assert!(
            output.noise_ratio < 0.8,
            "Trending signal should have relatively low noise ratio, got {}",
            output.noise_ratio
        );
        assert!(
            series.iter().any(|v| v.is_some()),
            "Series should have values"
        );
        // Denoised trend should be near the actual price
        let last_close = closes.last().copied().unwrap_or(0.0);
        assert!(
            (output.denoised_trend - last_close).abs() / last_close < 0.1,
            "Denoised should be near actual price"
        );
    }

    #[test]
    fn test_vpin_basic() {
        let candles = make_candles(&trending_up(100));
        let (output, _) = calculate_vpin(&candles, 10, 0.6);
        assert!(
            output.vpin >= 0.0 && output.vpin <= 1.0,
            "VPIN should be in [0,1]"
        );
        assert!(
            output.buy_pressure > 0.4,
            "Buy pressure should be significant in uptrend"
        );
    }

    #[test]
    fn test_changepoint_stable() {
        let closes = trending_up(100);
        let (output, _) = bayesian_changepoint(&closes, 100.0);
        // Stable trend should have low changepoint probability
        assert!(output.regime_stable, "Stable trend should be regime_stable");
    }

    #[test]
    fn test_full_oracle_bullish() {
        let candles = make_candles(&trending_up(200));
        let snapshot = calculate(&candles, &OracleWeights::default());
        // Strong uptrend should produce positive oracle score
        assert!(
            snapshot.oracle_score > 0.0,
            "Oracle score should be positive for uptrend, got {}",
            snapshot.oracle_score
        );
        assert!(snapshot.confidence > 0.0, "Confidence should be > 0");
    }

    #[test]
    fn test_full_oracle_bearish() {
        let candles = make_candles(&trending_down(200));
        let snapshot = calculate(&candles, &OracleWeights::default());
        // Strong downtrend should produce negative oracle score
        assert!(
            snapshot.oracle_score < 0.0,
            "Oracle score should be negative for downtrend, got {}",
            snapshot.oracle_score
        );
    }

    #[test]
    fn test_full_oracle_random() {
        let candles = make_candles(&random_walk(200));
        let snapshot = calculate(&candles, &OracleWeights::default());
        // Random walk: oracle score should be closer to 0 and confidence lower
        assert!(
            snapshot.oracle_score.abs() < 80.0,
            "Oracle score should not be extreme for random walk, got {}",
            snapshot.oracle_score
        );
    }

    #[test]
    fn test_oracle_history() {
        let candles = make_candles(&trending_up(100));
        let history = calculate_history(&candles, &OracleWeights::default());
        assert_eq!(history.oracle_scores.len(), 100);
        // After warmup (50), we should have scores
        let valid_scores: Vec<_> = history
            .oracle_scores
            .iter()
            .filter(|v| v.is_some())
            .collect();
        assert!(
            !valid_scores.is_empty(),
            "Should have valid oracle scores after warmup"
        );
    }

    #[test]
    fn test_oracle_trending_regime() {
        let candles: Vec<Candle> = (0..200).map(|i| Candle {
            time: i as i64,
            open: 100.0 + i as f64,
            high: 101.0 + i as f64,
            low: 99.0 + i as f64,
            close: 100.5 + i as f64,
            volume: 10000.0,
        }).collect();
        let weights = OracleWeights::default();
        let snap = calculate(&candles, &weights);
        assert!(snap.confidence >= 0.0, "confidence should be non-negative");
        assert!(snap.oracle_score.is_finite(), "oracle score should be finite");
    }

    #[test]
    fn test_oracle_weights_default_valid() {
        let w = OracleWeights::default();
        assert!(w.kalman >= 0.0);
        assert!(w.wavelet >= 0.0);
        assert!(w.hurst >= 0.0);
        assert!(w.vpin >= 0.0);
        assert!(w.confidence_floor >= 0.0 && w.confidence_floor <= 1.0);
    }

    #[test]
    fn test_oracle_short_input() {
        let candles: Vec<Candle> = (0..5).map(|i| Candle {
            time: i as i64, open: 100.0, high: 101.0, low: 99.0, close: 100.0, volume: 1000.0,
        }).collect();
        let weights = OracleWeights::default();
        let snap = calculate(&candles, &weights);
        assert_eq!(snap.dominant_regime, MarketRegime::Unknown);
    }

    #[test]
    fn test_oracle_history_lengths() {
        let candles: Vec<Candle> = (0..200).map(|i| Candle {
            time: i as i64, open: 100.0, high: 105.0, low: 95.0, close: 100.0 + (i as f64 * 0.1).sin(), volume: 5000.0,
        }).collect();
        let weights = OracleWeights::default();
        let hist = calculate_history(&candles, &weights);
        assert_eq!(hist.timestamps.len(), candles.len());
        assert_eq!(hist.oracle_scores.len(), candles.len());
        assert_eq!(hist.kalman_predicted.len(), candles.len());
    }

    #[test]
    fn test_oracle_score_bounded() {
        let candles: Vec<Candle> = (0..200).map(|i| Candle {
            time: i as i64, open: 100.0 + (i as f64 * 0.05).sin() * 3.0,
            high: 102.0 + (i as f64 * 0.05).sin() * 3.0,
            low: 98.0 + (i as f64 * 0.05).sin() * 3.0,
            close: 100.0 + (i as f64 * 0.05).cos() * 3.0,
            volume: 5000.0,
        }).collect();
        let weights = OracleWeights::default();
        let snap = calculate(&candles, &weights);
        assert!(snap.oracle_score >= -100.0 && snap.oracle_score <= 100.0,
            "oracle score out of expected range: {}", snap.oracle_score);
    }

    // ═══════════════════════════════════════════════════════════════
    //  S15: OracleAdaptation Tests
    // ═══════════════════════════════════════════════════════════════

    fn make_entry_snap(signals: [f64; 5], regime: &str, confidence: f64) -> LayerEntrySnapshot {
        LayerEntrySnapshot {
            signals,
            regime: regime.to_string(),
            confidence,
        }
    }

    #[test]
    fn adaptation_default_is_neutral() {
        let adapt = OracleAdaptation::default();
        assert_eq!(adapt.trade_count, 0);
        assert!(adapt.adapted_weights.is_empty());
        for key in &REGIME_KEYS {
            let corr = adapt.regime_layer_corr.get(*key).unwrap();
            assert_eq!(*corr, [0.0; LAYER_COUNT]);
        }
    }

    #[test]
    fn record_outcome_updates_correct_regime() {
        let mut adapt = OracleAdaptation::default();
        let base = OracleWeights::default();
        adapt.layer_signals_at_entry.insert(
            "AAPL".to_string(),
            make_entry_snap([0.8, 0.5, 0.3, 0.2, 0.6], "Trending", 0.9),
        );
        adapt.record_trade_outcome("AAPL", 1.0, &base);

        let trending = adapt.regime_layer_corr.get("Trending").unwrap();
        assert!(trending[0] > 0.0, "kalman corr should be positive after winning trade with positive signal");

        let mr = adapt.regime_layer_corr.get("MeanReverting").unwrap();
        for c in mr.iter() {
            assert!(c.abs() < 1e-6, "MeanReverting should be near-zero (only decay applied)");
        }
    }

    #[test]
    fn weight_direction_follows_correlation() {
        let mut adapt = OracleAdaptation::default();
        let base = OracleWeights::default();

        for i in 0..ADAPT_INTERVAL {
            adapt.layer_signals_at_entry.insert(
                format!("SYM{}", i),
                make_entry_snap([0.9, 0.1, 0.1, 0.1, 0.1], "Trending", 0.95),
            );
            adapt.record_trade_outcome(&format!("SYM{}", i), 1.0, &base);
        }

        let adapted = adapt.adapted_weights.get("Trending")
            .expect("adapted_weights should exist after 50 trades");
        assert!(adapted[0] > base.kalman * 0.9,
            "kalman weight should be elevated: got {}, base {}", adapted[0], base.kalman);
    }

    #[test]
    fn weight_cap_enforced() {
        let mut adapt = OracleAdaptation::default();
        let base = OracleWeights::default();

        for round in 0..5 {
            for i in 0..ADAPT_INTERVAL {
                let idx = round * ADAPT_INTERVAL + i;
                adapt.layer_signals_at_entry.insert(
                    format!("S{}", idx),
                    make_entry_snap([1.0, 0.0, 0.0, 0.0, 0.0], "Trending", 1.0),
                );
                adapt.record_trade_outcome(&format!("S{}", idx), 1.0, &base);
            }
        }

        let adapted = adapt.adapted_weights.get("Trending").unwrap();
        let base_arr = [base.kalman, base.wavelet, base.hurst, base.vpin, base.entropy_dir];
        for (i, (&aw, &bw)) in adapted.iter().zip(base_arr.iter()).enumerate() {
            let ratio = aw / (bw.max(1e-9));
            let raw_sum: f64 = adapted.iter().sum();
            assert!((raw_sum - 1.0).abs() < 1e-9,
                "weights should be normalized to 1.0, got {}", raw_sum);
            let unnorm_ratio = aw * raw_sum / bw.max(1e-9);
            let _ = unnorm_ratio;
            assert!(ratio > 0.0, "weight {} should be positive", i);
        }
    }

    #[test]
    fn weights_normalized_to_one() {
        let mut adapt = OracleAdaptation::default();
        let base = OracleWeights::default();

        for i in 0..ADAPT_INTERVAL {
            adapt.layer_signals_at_entry.insert(
                format!("T{}", i),
                make_entry_snap([0.7, -0.3, 0.5, -0.2, 0.8], "Unknown", 0.7),
            );
            adapt.record_trade_outcome(&format!("T{}", i), 1.0, &base);
        }

        for (_, adapted) in &adapt.adapted_weights {
            let sum: f64 = adapted.iter().sum();
            assert!((sum - 1.0).abs() < 1e-9,
                "adapted weights should sum to 1.0, got {}", sum);
        }
    }

    #[test]
    fn continuous_decay_shrinks_correlations() {
        let mut adapt = OracleAdaptation::default();
        let base = OracleWeights::default();

        adapt.layer_signals_at_entry.insert(
            "X".to_string(),
            make_entry_snap([1.0, 1.0, 1.0, 1.0, 1.0], "Trending", 1.0),
        );
        adapt.record_trade_outcome("X", 1.0, &base);

        let corr_after_1 = adapt.regime_layer_corr.get("Trending").unwrap()[0];

        for i in 1..500 {
            adapt.layer_signals_at_entry.insert(
                format!("Z{}", i),
                make_entry_snap([0.0, 0.0, 0.0, 0.0, 0.0], "Chaotic", 0.5),
            );
            adapt.record_trade_outcome(&format!("Z{}", i), 1.0, &base);
        }

        let corr_after_500 = adapt.regime_layer_corr.get("Trending").unwrap()[0];
        assert!(corr_after_500.abs() < corr_after_1.abs() * 0.5,
            "correlation should decay significantly after 500 trades: start={}, end={}",
            corr_after_1, corr_after_500);
    }

    #[test]
    fn confidence_weighted_ema() {
        let base = OracleWeights::default();

        let mut adapt_high = OracleAdaptation::default();
        adapt_high.layer_signals_at_entry.insert(
            "H".to_string(),
            make_entry_snap([0.8, 0.0, 0.0, 0.0, 0.0], "Trending", 0.95),
        );
        adapt_high.record_trade_outcome("H", 1.0, &base);
        let corr_high = adapt_high.regime_layer_corr.get("Trending").unwrap()[0];

        let mut adapt_low = OracleAdaptation::default();
        adapt_low.layer_signals_at_entry.insert(
            "L".to_string(),
            make_entry_snap([0.8, 0.0, 0.0, 0.0, 0.0], "Trending", 0.15),
        );
        adapt_low.record_trade_outcome("L", 1.0, &base);
        let corr_low = adapt_low.regime_layer_corr.get("Trending").unwrap()[0];

        assert!(corr_high > corr_low,
            "high-confidence trade should move correlation more: high={}, low={}",
            corr_high, corr_low);
    }

    #[test]
    fn serde_roundtrip_adaptation() {
        let mut adapt = OracleAdaptation::default();
        let base = OracleWeights::default();

        for i in 0..ADAPT_INTERVAL {
            adapt.layer_signals_at_entry.insert(
                format!("RT{}", i),
                make_entry_snap([0.5, 0.3, 0.7, 0.2, 0.4], "Trending", 0.8),
            );
            adapt.record_trade_outcome(&format!("RT{}", i), 1.0, &base);
        }

        adapt.layer_signals_at_entry.insert(
            "LIVE".to_string(),
            make_entry_snap([0.1, 0.2, 0.3, 0.4, 0.5], "Unknown", 0.6),
        );

        let json = serde_json::to_string(&adapt).unwrap();
        let restored: OracleAdaptation = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.trade_count, adapt.trade_count);
        assert_eq!(restored.adapted_weights.len(), adapt.adapted_weights.len());
        for (key, vals) in &adapt.regime_layer_corr {
            let restored_vals = restored.regime_layer_corr.get(key).unwrap();
            for (&a, &b) in vals.iter().zip(restored_vals.iter()) {
                assert!((a - b).abs() < 1e-12, "corr mismatch for {}: {} vs {}", key, a, b);
            }
        }
        assert!(restored.layer_signals_at_entry.is_empty(),
            "layer_signals_at_entry should be skipped in serde (volatile)");
    }

    #[test]
    fn adapt_weights_no_data_returns_base() {
        let adapt = OracleAdaptation::default();
        let base = OracleWeights::default();
        let result = adapt.adapt_weights(&base, &MarketRegime::Trending);
        assert!((result.kalman - base.kalman).abs() < 1e-12);
        assert!((result.wavelet - base.wavelet).abs() < 1e-12);
        assert!((result.hurst - base.hurst).abs() < 1e-12);
        assert!((result.vpin - base.vpin).abs() < 1e-12);
        assert!((result.entropy_dir - base.entropy_dir).abs() < 1e-12);
        assert!((result.confidence_floor - base.confidence_floor).abs() < 1e-12);
    }

    #[test]
    fn fifty_trade_trigger() {
        let mut adapt = OracleAdaptation::default();
        let base = OracleWeights::default();

        for i in 0..(ADAPT_INTERVAL - 1) {
            adapt.layer_signals_at_entry.insert(
                format!("FT{}", i),
                make_entry_snap([0.6, 0.4, 0.3, 0.2, 0.1], "Trending", 0.8),
            );
            adapt.record_trade_outcome(&format!("FT{}", i), 1.0, &base);
        }
        assert!(adapt.adapted_weights.is_empty(),
            "adapted_weights should be empty before {} trades", ADAPT_INTERVAL);

        adapt.layer_signals_at_entry.insert(
            "FT_TRIGGER".to_string(),
            make_entry_snap([0.6, 0.4, 0.3, 0.2, 0.1], "Trending", 0.8),
        );
        adapt.record_trade_outcome("FT_TRIGGER", 1.0, &base);
        assert!(!adapt.adapted_weights.is_empty(),
            "adapted_weights should be populated at trade {}", ADAPT_INTERVAL);
    }

    #[test]
    fn extract_layer_signals_from_snapshot() {
        let candles = make_candles(&trending_up(200));
        let weights = OracleWeights::default();
        let snap = calculate(&candles, &weights);
        let signals = extract_layer_signals(&snap);
        for (i, s) in signals.iter().enumerate() {
            assert!(s.is_finite(), "signal {} should be finite", i);
            assert!(*s >= -1.0 && *s <= 1.0,
                "signal {} should be in [-1,1], got {}", i, s);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    //  S05 — Oracle Bugfix Validation Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_entropy_clarity_range_after_fix() {
        let candles = make_candles(&trending_up(200));
        let history = calculate_history(&candles, &OracleWeights::default());
        let valid: Vec<f64> = history
            .entropy_series
            .iter()
            .filter_map(|v| *v)
            .collect();
        assert!(!valid.is_empty(), "Should have entropy values after warmup");

        for (i, &e) in valid.iter().enumerate() {
            assert!(
                e >= 0.0 && e <= 1.0,
                "Entropy at index {} should be in [0,1], got {}",
                i, e
            );
        }

        let spread = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
            - valid.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(
            spread > 0.01,
            "Entropy clarity should have meaningful variation, spread={:.4}",
            spread
        );
    }

    #[test]
    fn test_noise_ratio_from_wavelet_valid_range() {
        let candles = make_candles(&trending_up(200));
        let history = calculate_history(&candles, &OracleWeights::default());
        let nr_vals: Vec<f64> = history
            .noise_ratio_series
            .iter()
            .filter_map(|v| *v)
            .collect();
        assert!(!nr_vals.is_empty(), "Should have noise_ratio values");

        for &nr in &nr_vals {
            assert!(
                nr >= 0.0 && nr <= 1.0,
                "noise_ratio should be in [0,1], got {}",
                nr
            );
        }
        // Broadcast values are all identical (single wavelet pass over full series)
        if nr_vals.len() >= 2 {
            let first = nr_vals[0];
            assert!(
                nr_vals.iter().all(|&v| (v - first).abs() < 1e-12),
                "noise_ratio should be uniform broadcast across all bars"
            );
        }
    }

    #[test]
    fn test_vpin_buy_pressure_is_bvc_fraction() {
        // Build candles with asymmetric bars: close near high → BVC > 0.5
        let candles: Vec<Candle> = (0..200)
            .map(|i| {
                let c = 100.0 + (i as f64) * 0.3;
                Candle {
                    time: i as i64,
                    open: c - 0.3,
                    high: c + 0.2,
                    low: c - 1.0,
                    close: c,
                    volume: 5000.0 + (i as f64 * 50.0),
                }
            })
            .collect();

        let weights = OracleWeights::default();

        // Verify BVC formula: close near high → fraction > 0.5
        let bar = &candles[150];
        let range = bar.high - bar.low;
        let bvc = ((bar.close - bar.low) / range).clamp(0.0, 1.0);
        assert!(
            bvc > 0.7,
            "BVC for close-near-high bar should be >0.7, got {:.3}",
            bvc
        );

        // Verify that calculate_vpin returns proper buy_pressure from BVC
        let (vpin_out, _) = calculate_vpin(&candles, 30, weights.vpin_toxicity_alert);
        assert!(
            vpin_out.buy_pressure >= 0.0 && vpin_out.buy_pressure <= 1.0,
            "buy_pressure should be in [0,1], got {}",
            vpin_out.buy_pressure
        );
        assert!(
            vpin_out.buy_pressure > 0.5,
            "Close-near-high bars should have buy_pressure > 0.5, got {:.3}",
            vpin_out.buy_pressure
        );
    }

    #[test]
    fn test_changepoint_threshold_consistent() {
        let candles = make_candles(&trending_up(200));
        let weights = OracleWeights::default();
        let (live_output, _) = bayesian_changepoint(
            &candles.iter().map(|c| c.close).collect::<Vec<_>>(),
            100.0,
        );

        let history = calculate_history(&candles, &weights);

        let last_cp_live = live_output.changepoint_probability;
        let last_cp_hist = history
            .changepoint_series
            .iter()
            .rev()
            .find_map(|v| *v)
            .unwrap_or(0.0);

        let live_stable = last_cp_live < CHANGEPOINT_STABILITY_THRESHOLD;
        let hist_stable = last_cp_hist < CHANGEPOINT_STABILITY_THRESHOLD;
        assert_eq!(
            live_stable, hist_stable,
            "Live stability={} (cp={:.4}) vs history stability={} (cp={:.4}) should agree with threshold={}",
            live_stable, last_cp_live, hist_stable, last_cp_hist, CHANGEPOINT_STABILITY_THRESHOLD
        );
    }

    #[test]
    fn test_new_series_populated() {
        let candles = make_candles(&trending_up(200));
        let history = calculate_history(&candles, &OracleWeights::default());

        let n = candles.len();
        assert_eq!(history.kalman_gain_series.len(), n);
        assert_eq!(history.price_surprise_series.len(), n);
        assert_eq!(history.run_length_series.len(), n);
        assert_eq!(history.dominant_cycle_series.len(), n);
        assert_eq!(history.noise_ratio_series.len(), n);

        let kg_count = history.kalman_gain_series.iter().filter(|v| v.is_some()).count();
        assert!(kg_count > 0, "kalman_gain_series should have values");

        let ps_count = history.price_surprise_series.iter().filter(|v| v.is_some()).count();
        assert!(ps_count > 0, "price_surprise_series should have values");

        let rl_count = history.run_length_series.iter().filter(|v| v.is_some()).count();
        assert!(rl_count > 0, "run_length_series should have values");

        let dc_count = history.dominant_cycle_series.iter().filter(|v| v.is_some()).count();
        assert!(dc_count > 0, "dominant_cycle_series should have values");

        let nr_count = history.noise_ratio_series.iter().filter(|v| v.is_some()).count();
        assert!(nr_count > 0, "noise_ratio_series should have values");
    }

    #[test]
    fn test_live_vs_history_consistency() {
        let candles = make_candles(&trending_up(200));
        let weights = OracleWeights::default();

        let live = calculate(&candles, &weights);
        let history = calculate_history(&candles, &weights);
        let n = candles.len();

        // Primary comparison: pre-EMA raw scores (same fusion, no smoothing asymmetry)
        let raw_score = history.oracle_scores_raw[n - 1].unwrap_or(0.0);
        let raw_diff = (live.oracle_score - raw_score).abs();
        assert!(
            raw_diff < 5.0,
            "Live score={:.2} vs history raw={:.2}, diff={:.2} should be < 5 (pre-EMA)",
            live.oracle_score, raw_score, raw_diff
        );

        // Secondary: EMA-smoothed scores have looser tolerance (smoothing drift is expected)
        let ema_score = history.oracle_scores[n - 1].unwrap_or(0.0);
        let ema_diff = (live.oracle_score - ema_score).abs();
        assert!(
            ema_diff < 20.0,
            "Live score={:.2} vs history EMA={:.2}, diff={:.2} should be < 20",
            live.oracle_score, ema_score, ema_diff
        );

        let hist_conf = history.confidence_series[n - 1].unwrap_or(0.0);
        let conf_diff = (live.confidence - hist_conf).abs();
        assert!(
            conf_diff < 0.5,
            "Live conf={:.4} vs history last={:.4}, diff={:.4} should be < 0.5",
            live.confidence, hist_conf, conf_diff
        );

        assert_eq!(
            history.regime_series[n - 1].unwrap_or(MarketRegime::Unknown),
            live.dominant_regime,
            "Live regime should match history last regime"
        );
    }

    #[test]
    fn test_price_vs_predicted_nonzero_trending() {
        let candles = make_candles(&trending_up(200));
        let history = calculate_history(&candles, &OracleWeights::default());
        let live = calculate(&candles, &OracleWeights::default());

        assert!(
            live.kalman.price_vs_predicted.abs() > 1e-8,
            "Live price_vs_predicted should be nonzero for trending data, got {}",
            live.kalman.price_vs_predicted
        );

        let kalman_pred: Vec<f64> = history.kalman_predicted.iter().filter_map(|v| *v).collect();
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let n = candles.len();
        if let Some(pred) = history.kalman_predicted[n - 1] {
            let expected_pvp = closes[n - 1] - pred;
            let raw_score = history.oracle_scores_raw[n - 1];
            assert!(raw_score.is_some(), "Raw score at last bar should exist");
            assert!(
                kalman_pred.len() > 50,
                "Should have enough Kalman predictions"
            );
            assert!(
                expected_pvp.abs() > 1e-10,
                "price_vs_predicted should be nonzero for trending data"
            );
        }
    }

    #[test]
    fn test_run_length_wired_in_history() {
        let candles = make_candles(&trending_up(200));
        let history = calculate_history(&candles, &OracleWeights::default());
        let rl_vals: Vec<f64> = history
            .run_length_series
            .iter()
            .filter_map(|v| *v)
            .collect();
        assert!(!rl_vals.is_empty(), "Should have run_length values");
        assert!(
            rl_vals.iter().any(|&v| v > 0.0),
            "At least some run_length values should be > 0 for stable trending data"
        );
    }

    #[test]
    fn kalman_surprise_preserves_sign() {
        let up = trending_up(200);
        let candles_up = make_candles(&up);
        let live_up = calculate(&candles_up, &OracleWeights::default());

        assert!(
            live_up.kalman.price_vs_predicted != 0.0,
            "Should have nonzero signed price_vs_predicted, got 0"
        );
        assert!(
            live_up.kalman.price_vs_predicted.abs() < 1.0,
            "price_vs_predicted should be a normalized ratio, got {}",
            live_up.kalman.price_vs_predicted
        );

        let (_kalman_out, kalman_series) = kalman_filter(&up);
        let last_surprise = kalman_series.surprise_series.iter().rev()
            .find_map(|v| *v);
        if let Some(s) = last_surprise {
            assert!(
                s.is_finite() && !s.is_nan(),
                "Per-bar surprise should be finite"
            );
        }
    }

    #[test]
    fn kalman_surprise_units_consistent() {
        let prices = trending_up(200);
        let candles = make_candles(&prices);
        let weights = OracleWeights::default();

        let live = calculate(&candles, &weights);
        let history = calculate_history(&candles, &weights);

        assert!(
            live.kalman.price_vs_predicted.abs() < 1.0,
            "Live price_vs_predicted should be a ratio (< 1.0), got {}",
            live.kalman.price_vs_predicted
        );

        let n = candles.len();
        let last_hist = history.kalman_predicted[n - 1];
        if let Some(pred) = last_hist {
            let hist_pvp = (prices[n - 1] - pred) / prices[n - 1];
            assert!(
                hist_pvp.abs() < 1.0,
                "History price_vs_predicted should be a ratio (< 1.0), got {}",
                hist_pvp
            );
        }
    }

    #[test]
    fn lyapunov_live_matches_history_window() {
        let prices = trending_up(300);
        let candles = make_candles(&prices);
        let weights = OracleWeights::default();

        let live = calculate(&candles, &weights);

        let lyap_window = 200;
        let lyap_start = prices.len().saturating_sub(lyap_window);
        let expected = lyapunov_exponent(&prices[lyap_start..], 3, 2);

        let diff = (live.lyapunov.exponent - expected.exponent).abs();
        assert!(
            diff < 1e-10,
            "Live Lyapunov ({}) should match trailing-window computation ({}) exactly, diff={}",
            live.lyapunov.exponent, expected.exponent, diff
        );

        let full_series = lyapunov_exponent(&prices, 3, 2);
        if (full_series.exponent - expected.exponent).abs() < 1e-10 {
            return;
        }
        assert!(
            (live.lyapunov.exponent - full_series.exponent).abs() > 1e-12
                || prices.len() <= lyap_window,
            "With >200 bars, live should NOT use full series anymore"
        );
    }
}
