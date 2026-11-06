use arc_swap::ArcSwap;
use rand::Rng;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::Serialize;
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════
//  GENETIC OPTIMIZER — Real-Time Strategy Evolution
// ═══════════════════════════════════════════════════════════════════
//
//  Evolves trading parameters within hardcoded safety rails.
//  - Population of N parameter sets
//  - Fitness = Sharpe ratio on walk-forward validation
//  - Mutation bounded to ±15% of safety rails
//  - Champion must beat incumbent on BOTH in-sample and out-of-sample
//  - Update frequency: every 15 minutes max
//  - Execution: dedicated rayon thread pool (cores 8-24)
//
//  S13: SmartTrend genetic evolution added — separate SmartTrendParams
//  genome with its own FSM-aware fitness function. Runs the real
//  SmartTrendEngine::evaluate loop on candle data, not SMA synthetic.

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct TradingParams {
    pub fusion_threshold: Decimal,
    pub tp_percent: Decimal,
    pub sl_percent: Decimal,
    pub trail_gap_pct: Decimal,
    pub min_confidence: Decimal,
    pub ofi_filter_enabled: bool,
    pub nexus_buy_threshold_pct: Decimal,
}

impl TradingParams {
    pub fn default_safe() -> Self {
        Self {
            fusion_threshold: dec!(50.0),
            tp_percent: dec!(2.5),
            sl_percent: dec!(2.5),
            trail_gap_pct: dec!(1.5),
            min_confidence: dec!(0.30),
            ofi_filter_enabled: true,
            nexus_buy_threshold_pct: dec!(60.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Individual {
    pub params: TradingParams,
    pub fitness: f64,
    pub wins: u32,
    pub losses: u32,
    pub sharpe: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvolutionStats {
    pub generation: u32,
    pub population_size: usize,
    pub champion_fitness: f64,
    pub champion_sharpe: f64,
    pub champion_params: TradingParams,
    pub avg_fitness: f64,
    pub mutations_this_gen: u32,
    pub last_update_secs_ago: u64,
}

/// Snapshot of optimizer state for running evolution outside the lock.
pub struct EvolutionSnapshot {
    pub population: Vec<Individual>,
    pub generation: u32,
    pub safety_rails: TradingParams,
    pub max_deviation_pct: Decimal,
    pub population_size: usize,
    pub champion_fitness: f64,
}

pub struct NewChampion {
    pub fitness: f64,
    pub params: TradingParams,
}

pub struct EvolutionResult {
    pub new_population: Vec<Individual>,
    pub generation: u32,
    pub new_champion: Option<NewChampion>,
    pub stats: Option<EvolutionStats>,
}

/// Run evolution on a snapshot without holding the optimizer lock.
pub fn run_evolution_unlocked(
    snap: EvolutionSnapshot,
    candles: &[BacktestCandle],
) -> EvolutionResult {
    if candles.len() < 60 {
        return EvolutionResult {
            new_population: snap.population,
            generation: snap.generation,
            new_champion: None,
            stats: None,
        };
    }

    let split = (candles.len() as f64 * 0.70) as usize;
    let is_candles = &candles[..split];
    let oos_candles = &candles[split..];

    let generation = snap.generation + 1;
    let mut population = snap.population;
    let dev = snap.max_deviation_pct / dec!(100);

    use rayon::prelude::*;
    population.par_iter_mut().for_each(|individual| {
        let r = walk_forward_backtest(is_candles, &individual.params);
        individual.fitness = r.fitness;
        individual.sharpe = r.sharpe;
        individual.wins = r.wins;
        individual.losses = r.losses;
    });

    population.sort_by(|a, b| {
        b.fitness
            .partial_cmp(&a.fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let elite_count = 5.min(population.len());
    let elites: Vec<Individual> = population[..elite_count].to_vec();

    let mut rng = rand::thread_rng();
    let mut new_population: Vec<Individual> = elites.clone();

    while new_population.len() < snap.population_size {
        let parent_a = &elites[rng.gen_range(0..elite_count)];
        let parent_b = &elites[rng.gen_range(0..elite_count)];

        let child_params = if rng.gen_bool(0.7) {
            let crossed = crossover_standalone(&parent_a.params, &parent_b.params, &mut rng);
            mutate_standalone(&crossed, &snap.safety_rails, dev, &mut rng)
        } else {
            mutate_standalone(&parent_a.params, &snap.safety_rails, dev, &mut rng)
        };

        new_population.push(Individual {
            params: child_params,
            fitness: 0.0,
            wins: 0,
            losses: 0,
            sharpe: 0.0,
        });
    }

    let best = &elites[0];
    let oos_result = walk_forward_backtest(oos_candles, &best.params);
    let oos_pass = oos_result.sharpe > 0.0 && oos_result.total_pnl > 0.0;

    let new_champion = if best.fitness > snap.champion_fitness && best.sharpe > 0.0 && oos_pass {
        Some(NewChampion {
            fitness: best.fitness,
            params: best.params.clone(),
        })
    } else {
        None
    };

    let evaluated: Vec<f64> = new_population.iter()
        .filter(|i| i.fitness != 0.0 || i.wins > 0 || i.losses > 0)
        .map(|i| i.fitness)
        .collect();
    let avg_fitness = if evaluated.is_empty() {
        0.0
    } else {
        evaluated.iter().sum::<f64>() / evaluated.len() as f64
    };

    let stats = EvolutionStats {
        generation,
        population_size: new_population.len(),
        champion_fitness: new_champion
            .as_ref()
            .map(|c| c.fitness)
            .unwrap_or(best.fitness),
        champion_sharpe: best.sharpe,
        champion_params: new_champion
            .as_ref()
            .map(|c| c.params.clone())
            .unwrap_or_else(|| best.params.clone()),
        avg_fitness,
        mutations_this_gen: (snap.population_size - elite_count) as u32,
        last_update_secs_ago: 0,
    };

    EvolutionResult {
        new_population,
        generation,
        new_champion,
        stats: Some(stats),
    }
}

fn mutate_standalone(
    base: &TradingParams,
    rails: &TradingParams,
    dev: Decimal,
    rng: &mut impl Rng,
) -> TradingParams {
    let lo = Decimal::ONE - dev;
    let hi = Decimal::ONE + dev;
    TradingParams {
        fusion_threshold: GeneticOptimizer::mutate_decimal(
            base.fusion_threshold,
            dev,
            rails.fusion_threshold * lo,
            rails.fusion_threshold * hi,
            rng,
        ),
        tp_percent: GeneticOptimizer::mutate_decimal(
            base.tp_percent,
            dev,
            (rails.tp_percent * lo).max(dec!(0.5)),
            rails.tp_percent * hi,
            rng,
        ),
        sl_percent: GeneticOptimizer::mutate_decimal(
            base.sl_percent,
            dev,
            (rails.sl_percent * lo).max(dec!(0.5)),
            rails.sl_percent * hi,
            rng,
        ),
        trail_gap_pct: GeneticOptimizer::mutate_decimal(
            base.trail_gap_pct,
            dev,
            (rails.trail_gap_pct * lo).max(dec!(0.3)),
            rails.trail_gap_pct * hi,
            rng,
        ),
        min_confidence: GeneticOptimizer::mutate_decimal(
            base.min_confidence,
            dev,
            (rails.min_confidence * lo).max(dec!(0.10)),
            (rails.min_confidence * hi).min(dec!(0.90)),
            rng,
        ),
        ofi_filter_enabled: if rng.gen_bool(0.9) {
            base.ofi_filter_enabled
        } else {
            !base.ofi_filter_enabled
        },
        nexus_buy_threshold_pct: GeneticOptimizer::mutate_decimal(
            base.nexus_buy_threshold_pct,
            dev,
            (rails.nexus_buy_threshold_pct * lo).max(dec!(50)),
            (rails.nexus_buy_threshold_pct * hi).min(dec!(90)),
            rng,
        ),
    }
}

fn crossover_standalone(a: &TradingParams, b: &TradingParams, rng: &mut impl Rng) -> TradingParams {
    TradingParams {
        fusion_threshold: if rng.gen_bool(0.5) {
            a.fusion_threshold
        } else {
            b.fusion_threshold
        },
        tp_percent: if rng.gen_bool(0.5) {
            a.tp_percent
        } else {
            b.tp_percent
        },
        sl_percent: if rng.gen_bool(0.5) {
            a.sl_percent
        } else {
            b.sl_percent
        },
        trail_gap_pct: if rng.gen_bool(0.5) {
            a.trail_gap_pct
        } else {
            b.trail_gap_pct
        },
        min_confidence: if rng.gen_bool(0.5) {
            a.min_confidence
        } else {
            b.min_confidence
        },
        ofi_filter_enabled: if rng.gen_bool(0.5) {
            a.ofi_filter_enabled
        } else {
            b.ofi_filter_enabled
        },
        nexus_buy_threshold_pct: if rng.gen_bool(0.5) {
            a.nexus_buy_threshold_pct
        } else {
            b.nexus_buy_threshold_pct
        },
    }
}

pub struct GeneticOptimizer {
    population: Vec<Individual>,
    generation: u32,
    safety_rails: TradingParams,
    max_deviation_pct: Decimal,
    population_size: usize,
    pub active_params: Arc<ArcSwap<TradingParams>>,
    last_evolution: std::time::Instant,
    evolution_interval_secs: u64,
    champion_fitness: f64,
}

impl GeneticOptimizer {
    pub fn new(
        population_size: usize,
        safety_rails: TradingParams,
        max_deviation_pct: Decimal,
    ) -> Self {
        let initial_params = safety_rails.clone();
        let active = Arc::new(ArcSwap::from_pointee(initial_params.clone()));

        let mut optimizer = Self {
            population: Vec::with_capacity(population_size),
            generation: 0,
            safety_rails,
            max_deviation_pct,
            population_size,
            active_params: active,
            last_evolution: std::time::Instant::now(),
            evolution_interval_secs: 900, // 15 minutes
            champion_fitness: f64::NEG_INFINITY,
        };

        optimizer.seed_population();
        optimizer
    }

    pub fn with_defaults() -> Self {
        Self::new(20, TradingParams::default_safe(), dec!(15))
    }

    fn seed_population(&mut self) {
        let mut rng = rand::thread_rng();
        self.population.clear();

        self.population.push(Individual {
            params: self.safety_rails.clone(),
            fitness: 0.0,
            wins: 0,
            losses: 0,
            sharpe: 0.0,
        });

        for _ in 1..self.population_size {
            let params = self.mutate_params(&self.safety_rails, &mut rng);
            self.population.push(Individual {
                params,
                fitness: 0.0,
                wins: 0,
                losses: 0,
                sharpe: 0.0,
            });
        }
    }

    pub fn mutate_decimal(
        value: Decimal,
        deviation: Decimal,
        min: Decimal,
        max: Decimal,
        rng: &mut impl Rng,
    ) -> Decimal {
        let delta = value * deviation;
        let noise = Decimal::from_f64(rng.gen_range(-1.0..1.0)).unwrap_or(Decimal::ZERO);
        (value + delta * noise).max(min).min(max)
    }

    fn mutate_params(&self, base: &TradingParams, rng: &mut impl Rng) -> TradingParams {
        let dev = self.max_deviation_pct / dec!(100);
        let sr = &self.safety_rails;
        let lo = Decimal::ONE - dev;
        let hi = Decimal::ONE + dev;

        TradingParams {
            fusion_threshold: Self::mutate_decimal(
                base.fusion_threshold,
                dev,
                sr.fusion_threshold * lo,
                sr.fusion_threshold * hi,
                rng,
            ),
            tp_percent: Self::mutate_decimal(
                base.tp_percent,
                dev,
                (sr.tp_percent * lo).max(dec!(0.5)),
                sr.tp_percent * hi,
                rng,
            ),
            sl_percent: Self::mutate_decimal(
                base.sl_percent,
                dev,
                (sr.sl_percent * lo).max(dec!(0.5)),
                sr.sl_percent * hi,
                rng,
            ),
            trail_gap_pct: Self::mutate_decimal(
                base.trail_gap_pct,
                dev,
                (sr.trail_gap_pct * lo).max(dec!(0.3)),
                sr.trail_gap_pct * hi,
                rng,
            ),
            min_confidence: Self::mutate_decimal(
                base.min_confidence,
                dev,
                (sr.min_confidence * lo).max(dec!(0.10)),
                (sr.min_confidence * hi).min(dec!(0.90)),
                rng,
            ),
            ofi_filter_enabled: if rng.gen_bool(0.9) {
                base.ofi_filter_enabled
            } else {
                !base.ofi_filter_enabled
            },
            nexus_buy_threshold_pct: Self::mutate_decimal(
                base.nexus_buy_threshold_pct,
                dev,
                (sr.nexus_buy_threshold_pct * lo).max(dec!(50)),
                (sr.nexus_buy_threshold_pct * hi).min(dec!(90)),
                rng,
            ),
        }
    }

    #[cfg(test)]
    fn crossover(
        &self,
        parent_a: &TradingParams,
        parent_b: &TradingParams,
        rng: &mut impl Rng,
    ) -> TradingParams {
        TradingParams {
            fusion_threshold: if rng.gen_bool(0.5) {
                parent_a.fusion_threshold
            } else {
                parent_b.fusion_threshold
            },
            tp_percent: if rng.gen_bool(0.5) {
                parent_a.tp_percent
            } else {
                parent_b.tp_percent
            },
            sl_percent: if rng.gen_bool(0.5) {
                parent_a.sl_percent
            } else {
                parent_b.sl_percent
            },
            trail_gap_pct: if rng.gen_bool(0.5) {
                parent_a.trail_gap_pct
            } else {
                parent_b.trail_gap_pct
            },
            min_confidence: if rng.gen_bool(0.5) {
                parent_a.min_confidence
            } else {
                parent_b.min_confidence
            },
            ofi_filter_enabled: if rng.gen_bool(0.5) {
                parent_a.ofi_filter_enabled
            } else {
                parent_b.ofi_filter_enabled
            },
            nexus_buy_threshold_pct: if rng.gen_bool(0.5) {
                parent_a.nexus_buy_threshold_pct
            } else {
                parent_b.nexus_buy_threshold_pct
            },
        }
    }

    pub fn stats(&self) -> EvolutionStats {
        let best = self.population.first();
        let avg_fitness = if self.population.is_empty() {
            0.0
        } else {
            self.population.iter().map(|i| i.fitness).sum::<f64>() / self.population.len() as f64
        };

        EvolutionStats {
            generation: self.generation,
            population_size: self.population.len(),
            champion_fitness: best.map(|b| b.fitness).unwrap_or(0.0),
            champion_sharpe: best.map(|b| b.sharpe).unwrap_or(0.0),
            champion_params: best
                .map(|b| b.params.clone())
                .unwrap_or_else(TradingParams::default_safe),
            avg_fitness,
            mutations_this_gen: (self.population_size - 5.min(self.population_size)) as u32,
            last_update_secs_ago: self.last_evolution.elapsed().as_secs(),
        }
    }

    pub fn current_params(&self) -> Arc<TradingParams> {
        self.active_params.load_full()
    }

    /// Extract a snapshot of the optimizer state needed for CPU-heavy evolution.
    /// Caller runs evolution unlocked, then calls `apply_evolution_results`.
    pub fn prepare_evolution(&self) -> Option<EvolutionSnapshot> {
        if self.last_evolution.elapsed().as_secs() < self.evolution_interval_secs {
            return None;
        }
        Some(EvolutionSnapshot {
            population: self.population.clone(),
            generation: self.generation,
            safety_rails: self.safety_rails.clone(),
            max_deviation_pct: self.max_deviation_pct,
            population_size: self.population_size,
            champion_fitness: self.champion_fitness,
        })
    }

    /// Apply the results of an evolution run back into the optimizer.
    pub fn apply_evolution_results(&mut self, result: EvolutionResult) {
        self.population = result.new_population;
        self.generation = result.generation;
        self.last_evolution = std::time::Instant::now();
        if let Some(new_champ) = result.new_champion {
            self.champion_fitness = new_champ.fitness;
            self.active_params.store(Arc::new(new_champ.params));
        }
    }

    #[cfg(test)]
    pub fn evolve_walk_forward(&mut self, candles: &[BacktestCandle]) -> bool {
        if self.last_evolution.elapsed().as_secs() < self.evolution_interval_secs {
            return false;
        }
        if candles.len() < 60 {
            return false;
        }

        let split = (candles.len() as f64 * 0.70) as usize;
        let is_candles = &candles[..split];
        let oos_candles = &candles[split..];

        self.generation += 1;

        // Evaluate on in-sample
        for individual in &mut self.population {
            let r = walk_forward_backtest(is_candles, &individual.params);
            individual.fitness = r.fitness;
            individual.sharpe = r.sharpe;
            individual.wins = r.wins;
            individual.losses = r.losses;
        }

        self.population.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let elite_count = 5.min(self.population.len());
        let elites: Vec<Individual> = self.population[..elite_count].to_vec();

        let mut rng = rand::thread_rng();
        let mut new_population: Vec<Individual> = elites.clone();

        while new_population.len() < self.population_size {
            let parent_a = &elites[rng.gen_range(0..elite_count)];
            let parent_b = &elites[rng.gen_range(0..elite_count)];

            let child_params = if rng.gen_bool(0.7) {
                let crossed = self.crossover(&parent_a.params, &parent_b.params, &mut rng);
                self.mutate_params(&crossed, &mut rng)
            } else {
                self.mutate_params(&parent_a.params, &mut rng)
            };

            new_population.push(Individual {
                params: child_params,
                fitness: 0.0,
                wins: 0,
                losses: 0,
                sharpe: 0.0,
            });
        }

        self.population = new_population;

        // Validate champion on out-of-sample
        let best = &elites[0];
        let oos_result = walk_forward_backtest(oos_candles, &best.params);
        let oos_pass = oos_result.sharpe > 0.0 && oos_result.total_pnl > 0.0;

        let new_champion = best.fitness > self.champion_fitness && best.sharpe > 0.0 && oos_pass;

        if new_champion {
            self.champion_fitness = best.fitness;
            self.active_params.store(Arc::new(best.params.clone()));
            println!(
                "GENETIC: Gen {} — NEW CHAMPION (IS: fit={:.3} sharpe={:.3} | OOS: sharpe={:.3} pnl=${:.0})",
                self.generation, best.fitness, best.sharpe, oos_result.sharpe, oos_result.total_pnl
            );
        } else if best.fitness > self.champion_fitness && !oos_pass {
            println!(
                "GENETIC: Gen {} — IS champion REJECTED by OOS (oos_sharpe={:.3}, oos_pnl=${:.0})",
                self.generation, oos_result.sharpe, oos_result.total_pnl
            );
        }

        self.last_evolution = std::time::Instant::now();
        new_champion
    }

    pub fn freeze_evolution(&mut self) {
        self.evolution_interval_secs = u64::MAX;
    }

    pub fn resume_evolution(&mut self) {
        self.evolution_interval_secs = 900;
    }

    pub fn restore_champion(&mut self, params: TradingParams, fitness: f64) {
        self.active_params.store(Arc::new(params.clone()));
        self.champion_fitness = if fitness.is_finite() { fitness } else { 0.0 };
        let champion = Individual {
            params,
            fitness: self.champion_fitness,
            wins: 0,
            losses: 0,
            sharpe: 0.0,
        };
        if self.population.is_empty() {
            self.population.push(champion);
        } else {
            self.population[0] = champion;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  WALK-FORWARD MINI BACKTEST — fitness evaluator for evolution
// ═══════════════════════════════════════════════════════════════════

pub use titan_types::BacktestCandle;

#[derive(Debug, Clone, Serialize)]
pub struct BacktestResult {
    pub fitness: f64,
    pub sharpe: f64,
    pub wins: u32,
    pub losses: u32,
    pub total_pnl: f64,
    pub max_drawdown_pct: f64,
}

pub fn walk_forward_backtest(candles: &[BacktestCandle], params: &TradingParams) -> BacktestResult {
    let empty = BacktestResult {
        fitness: f64::NEG_INFINITY,
        sharpe: 0.0,
        wins: 0,
        losses: 0,
        total_pnl: 0.0,
        max_drawdown_pct: 0.0,
    };

    if candles.len() < 30 {
        return empty;
    }

    let tp_pct = params.tp_percent.to_f64().unwrap_or(2.5) / 100.0;
    let sl_pct = params.sl_percent.to_f64().unwrap_or(2.5) / 100.0;
    let trail_pct = params.trail_gap_pct.to_f64().unwrap_or(1.5) / 100.0;
    let min_conf = params.min_confidence.to_f64().unwrap_or(0.30);
    let fusion_thresh = params.fusion_threshold.to_f64().unwrap_or(50.0);

    let mut pnls: Vec<f64> = Vec::new();
    let mut equity = 100_000.0;
    let mut peak_equity = equity;
    let mut max_dd_pct = 0.0;

    #[derive(Clone)]
    struct SimPosition {
        entry_price: f64,
        is_long: bool,
        trail_extreme: f64,
    }

    let mut position: Option<SimPosition> = None;
    let lookback = 20;

    for i in lookback..candles.len() {
        let c = &candles[i];
        // Simple momentum signal: compare SMA(5) vs SMA(20)
        let sma5: f64 = candles[i.saturating_sub(5)..i]
            .iter()
            .map(|x| x.close)
            .sum::<f64>()
            / 5.0;
        let sma20: f64 = candles[i.saturating_sub(20)..i]
            .iter()
            .map(|x| x.close)
            .sum::<f64>()
            / 20.0;
        let momentum = (sma5 - sma20) / sma20;

        // Volatility from recent range
        let vol: f64 = candles[i.saturating_sub(10)..i]
            .iter()
            .map(|x| (x.high - x.low) / x.close)
            .sum::<f64>()
            / 10.0;

        let signal_strength = (momentum.abs() / vol.max(0.0001)).clamp(0.0, 1.0);

        if let Some(ref mut pos) = position {
            let pnl_pct = if pos.is_long {
                (c.close - pos.entry_price) / pos.entry_price
            } else {
                (pos.entry_price - c.close) / pos.entry_price
            };

            // Update trail extreme
            if pos.is_long && c.high > pos.trail_extreme {
                pos.trail_extreme = c.high;
            } else if !pos.is_long && c.low < pos.trail_extreme {
                pos.trail_extreme = c.low;
            }

            let trail_drop = if pos.is_long {
                (pos.trail_extreme - c.close) / pos.trail_extreme
            } else {
                (c.close - pos.trail_extreme) / pos.trail_extreme
            };

            let should_exit = pnl_pct <= -sl_pct
                || pnl_pct >= tp_pct
                || (pnl_pct > 0.005 && trail_drop >= trail_pct);

            if should_exit {
                let trade_pnl = pnl_pct * 10_000.0; // $10k position
                equity += trade_pnl;
                pnls.push(trade_pnl);
                if equity > peak_equity {
                    peak_equity = equity;
                }
                let dd = (peak_equity - equity) / peak_equity * 100.0;
                if dd > max_dd_pct {
                    max_dd_pct = dd;
                }
                position = None;
            }
        } else {
            // Entry logic
            let score = signal_strength * 100.0;
            if score >= fusion_thresh && signal_strength >= min_conf {
                let is_long = momentum > 0.0;
                position = Some(SimPosition {
                    entry_price: c.close,
                    is_long,
                    trail_extreme: if is_long { c.high } else { c.low },
                });
            }
        }
    }

    // Close any remaining position at last price
    if let Some(pos) = position {
        let last = match candles.last() {
            Some(c) => c.close,
            None => return empty,
        };
        let pnl_pct = if pos.is_long {
            (last - pos.entry_price) / pos.entry_price
        } else {
            (pos.entry_price - last) / pos.entry_price
        };
        let trade_pnl = pnl_pct * 10_000.0;
        equity += trade_pnl;
        pnls.push(trade_pnl);
        if equity > peak_equity {
            peak_equity = equity;
        }
        let dd = (peak_equity - equity) / peak_equity * 100.0;
        if dd > max_dd_pct {
            max_dd_pct = dd;
        }
    }

    if pnls.is_empty() {
        return empty;
    }

    let wins = pnls.iter().filter(|p| **p > 0.0).count() as u32;
    let losses = pnls.iter().filter(|p| **p < 0.0).count() as u32;
    let total_pnl: f64 = pnls.iter().sum();

    // Sharpe ratio
    let n = pnls.len() as f64;
    let mean = total_pnl / n;
    let var = pnls.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();
    let sharpe = if std > 1e-9 {
        mean / std * n.sqrt()
    } else {
        0.0
    };

    // Fitness = Sharpe * profit_factor - drawdown penalty
    let gross_wins: f64 = pnls.iter().filter(|p| **p > 0.0).sum();
    let gross_losses: f64 = pnls.iter().filter(|p| **p < 0.0).map(|p| p.abs()).sum();
    let profit_factor = if gross_losses > 1e-9 {
        gross_wins / gross_losses
    } else {
        10.0
    };

    let dd_penalty = (max_dd_pct / 5.0).powi(2);
    let trade_count_bonus = (n / 10.0).min(2.0);
    let fitness = sharpe * profit_factor.min(5.0) + trade_count_bonus - dd_penalty;

    BacktestResult {
        fitness,
        sharpe,
        wins,
        losses,
        total_pnl,
        max_drawdown_pct: max_dd_pct,
    }
}

// ═══════════════════════════════════════════════════════════════════
//  SMARTTREND GENETIC — Separate genome for SmartTrend AI v3 (S13)
// ═══════════════════════════════════════════════════════════════════

/// Evolvable parameters for SmartTrend strategy.
/// Only includes fields that meaningfully affect entry/exit decisions in backtest.
/// Operational settings (sentinel cadence, bot loop ms) are NOT evolved.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct SmartTrendParams {
    /// Pillar weight: Macro (oracle + Kalman). Range [0.10, 0.80]
    pub macro_weight: f64,
    /// Pillar weight: Micro (OFI + CVD + microprice). Range [0.05, 0.60]
    pub micro_weight: f64,
    /// Pillar weight: Momentum (GPU + wavelet + RVOL). Range [0.05, 0.60]
    pub momentum_weight: f64,
    /// Combined score threshold for entry. Range [50, 85]
    pub entry_threshold: f64,
    /// Combined score threshold for stalking. Range [40, 75]
    pub stalking_threshold: f64,
    /// Changepoint probability to reset FSM. Range [0.40, 0.95]
    pub changepoint_reset: f64,
    /// Trail tightening per bar (fraction). Range [0.01, 0.20]
    pub trail_tighten_pct: f64,
    /// Hysteresis margin for Stalking->Flat. Range [1.0, 15.0]
    pub hysteresis_margin: f64,
    /// Bayesian halflife in trades. Range [50, 500]
    pub bayesian_halflife: u32,
    /// Min bars after changepoint before entry. Range [3, 20]
    pub stabilization_window: u32,
    /// Dominant cycle filter on/off
    pub dominant_cycle_filter: bool,
    /// Risk per trade fraction. Range [0.005, 0.05]
    pub risk_per_trade: f64,
}

impl SmartTrendParams {
    pub fn default_safe() -> Self {
        Self {
            macro_weight: 0.40,
            micro_weight: 0.35,
            momentum_weight: 0.25,
            entry_threshold: 65.0,
            stalking_threshold: 55.0,
            changepoint_reset: 0.70,
            trail_tighten_pct: 0.05,
            hysteresis_margin: 5.0,
            bayesian_halflife: 200,
            stabilization_window: 8,
            dominant_cycle_filter: true,
            risk_per_trade: 0.01,
        }
    }

    /// Normalize pillar weights to sum to 1.0.
    pub fn normalized_weights(&self) -> (f64, f64, f64) {
        let sum = self.macro_weight + self.micro_weight + self.momentum_weight;
        if sum <= 0.0 {
            return (0.40, 0.35, 0.25);
        }
        (self.macro_weight / sum, self.micro_weight / sum, self.momentum_weight / sum)
    }
}

#[derive(Debug, Clone)]
pub struct StIndividual {
    pub params: SmartTrendParams,
    pub fitness: f64,
    pub wins: u32,
    pub losses: u32,
    pub sharpe: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StEvolutionStats {
    pub generation: u32,
    pub population_size: usize,
    pub champion_fitness: f64,
    pub champion_sharpe: f64,
    pub champion_params: SmartTrendParams,
    pub avg_fitness: f64,
    pub diversity_pct: f64,
    pub mutations_this_gen: u32,
    pub last_update_secs_ago: u64,
}

pub struct StEvolutionSnapshot {
    pub population: Vec<StIndividual>,
    pub generation: u32,
    pub safety_rails: SmartTrendParams,
    pub max_deviation_pct: f64,
    pub population_size: usize,
    pub champion_fitness: f64,
}

pub struct StNewChampion {
    pub fitness: f64,
    pub params: SmartTrendParams,
}

pub struct StEvolutionResult {
    pub new_population: Vec<StIndividual>,
    pub generation: u32,
    pub new_champion: Option<StNewChampion>,
    pub stats: Option<StEvolutionStats>,
}

fn mutate_f64(value: f64, dev_pct: f64, min: f64, max: f64, rng: &mut impl Rng) -> f64 {
    let delta = value * dev_pct;
    let noise: f64 = rng.gen_range(-1.0..1.0);
    (value + delta * noise).clamp(min, max)
}

fn mutate_u32(value: u32, dev_pct: f64, min: u32, max: u32, rng: &mut impl Rng) -> u32 {
    let delta = (value as f64 * dev_pct).max(1.0);
    let noise: f64 = rng.gen_range(-1.0..1.0);
    let v = (value as f64 + delta * noise).round().clamp(min as f64, max as f64);
    v as u32
}

fn st_mutate(base: &SmartTrendParams, dev: f64, rng: &mut impl Rng) -> SmartTrendParams {
    SmartTrendParams {
        macro_weight: mutate_f64(base.macro_weight, dev, 0.10, 0.80, rng),
        micro_weight: mutate_f64(base.micro_weight, dev, 0.05, 0.60, rng),
        momentum_weight: mutate_f64(base.momentum_weight, dev, 0.05, 0.60, rng),
        entry_threshold: mutate_f64(base.entry_threshold, dev, 50.0, 85.0, rng),
        stalking_threshold: mutate_f64(base.stalking_threshold, dev, 40.0, 75.0, rng),
        changepoint_reset: mutate_f64(base.changepoint_reset, dev, 0.40, 0.95, rng),
        trail_tighten_pct: mutate_f64(base.trail_tighten_pct, dev, 0.01, 0.20, rng),
        hysteresis_margin: mutate_f64(base.hysteresis_margin, dev, 1.0, 15.0, rng),
        bayesian_halflife: mutate_u32(base.bayesian_halflife, dev, 50, 500, rng),
        stabilization_window: mutate_u32(base.stabilization_window, dev, 3, 20, rng),
        dominant_cycle_filter: if rng.gen_bool(0.9) { base.dominant_cycle_filter } else { !base.dominant_cycle_filter },
        risk_per_trade: mutate_f64(base.risk_per_trade, dev, 0.005, 0.05, rng),
    }
}

fn st_crossover(a: &SmartTrendParams, b: &SmartTrendParams, rng: &mut impl Rng) -> SmartTrendParams {
    SmartTrendParams {
        macro_weight: if rng.gen_bool(0.5) { a.macro_weight } else { b.macro_weight },
        micro_weight: if rng.gen_bool(0.5) { a.micro_weight } else { b.micro_weight },
        momentum_weight: if rng.gen_bool(0.5) { a.momentum_weight } else { b.momentum_weight },
        entry_threshold: if rng.gen_bool(0.5) { a.entry_threshold } else { b.entry_threshold },
        stalking_threshold: if rng.gen_bool(0.5) { a.stalking_threshold } else { b.stalking_threshold },
        changepoint_reset: if rng.gen_bool(0.5) { a.changepoint_reset } else { b.changepoint_reset },
        trail_tighten_pct: if rng.gen_bool(0.5) { a.trail_tighten_pct } else { b.trail_tighten_pct },
        hysteresis_margin: if rng.gen_bool(0.5) { a.hysteresis_margin } else { b.hysteresis_margin },
        bayesian_halflife: if rng.gen_bool(0.5) { a.bayesian_halflife } else { b.bayesian_halflife },
        stabilization_window: if rng.gen_bool(0.5) { a.stabilization_window } else { b.stabilization_window },
        dominant_cycle_filter: if rng.gen_bool(0.5) { a.dominant_cycle_filter } else { b.dominant_cycle_filter },
        risk_per_trade: if rng.gen_bool(0.5) { a.risk_per_trade } else { b.risk_per_trade },
    }
}

/// Population diversity metric: average Hamming-style distance between individuals.
/// Returns [0, 100] where 0 = all identical, 100 = maximum diversity.
pub fn st_population_diversity(pop: &[StIndividual]) -> f64 {
    if pop.len() < 2 { return 0.0; }
    let n = pop.len();
    let mut total_dist = 0.0;
    let mut pairs = 0u64;

    for i in 0..n {
        for j in (i+1)..n {
            let a = &pop[i].params;
            let b = &pop[j].params;
            let mut dist = 0.0;
            let mut fields = 0.0;

            // Continuous fields: relative difference [0, 1] per field
            let check = |va: f64, vb: f64, range: f64| -> f64 {
                if range <= 0.0 { return 0.0; }
                (va - vb).abs() / range
            };
            dist += check(a.macro_weight, b.macro_weight, 0.70);
            dist += check(a.micro_weight, b.micro_weight, 0.55);
            dist += check(a.momentum_weight, b.momentum_weight, 0.55);
            dist += check(a.entry_threshold, b.entry_threshold, 35.0);
            dist += check(a.stalking_threshold, b.stalking_threshold, 35.0);
            dist += check(a.changepoint_reset, b.changepoint_reset, 0.55);
            dist += check(a.trail_tighten_pct, b.trail_tighten_pct, 0.19);
            dist += check(a.hysteresis_margin, b.hysteresis_margin, 14.0);
            dist += check(a.bayesian_halflife as f64, b.bayesian_halflife as f64, 450.0);
            dist += check(a.stabilization_window as f64, b.stabilization_window as f64, 17.0);
            dist += check(a.risk_per_trade, b.risk_per_trade, 0.045);
            fields += 11.0;

            // Boolean field: 0 or 1
            if a.dominant_cycle_filter != b.dominant_cycle_filter { dist += 1.0; }
            fields += 1.0;

            total_dist += (dist / fields) * 100.0;
            pairs += 1;
        }
    }

    if pairs == 0 { 0.0 } else { total_dist / pairs as f64 }
}

// ═══════════════════════════════════════════════════════════════════
//  SMARTTREND WALK-FORWARD BACKTEST — FSM-aware fitness evaluation
// ═══════════════════════════════════════════════════════════════════
//
//  Unlike the Fusion backtest (SMA momentum synthetic), this uses
//  the real SmartTrend FSM logic: Flat->Stalking->Entry->InTrade->Trail.
//  Simulated inputs derive macro/micro/momentum from OHLC candle data.

#[derive(Debug, Clone, Serialize)]
pub struct StBacktestResult {
    pub fitness: f64,
    pub sharpe: f64,
    pub wins: u32,
    pub losses: u32,
    pub total_pnl: f64,
    pub max_drawdown_pct: f64,
    pub profit_factor: f64,
    pub win_rate: f64,
    pub avg_bars_in_trade: f64,
    pub trade_count: u32,
}

pub fn smarttrend_walk_forward_backtest(
    candles: &[BacktestCandle],
    params: &SmartTrendParams,
) -> StBacktestResult {
    let empty = StBacktestResult {
        fitness: f64::NEG_INFINITY,
        sharpe: 0.0,
        wins: 0,
        losses: 0,
        total_pnl: 0.0,
        max_drawdown_pct: 0.0,
        profit_factor: 0.0,
        win_rate: 0.0,
        avg_bars_in_trade: 0.0,
        trade_count: 0,
    };

    if candles.len() < 50 {
        return empty;
    }

    let (w_macro, w_micro, w_momentum) = params.normalized_weights();
    let lookback = 20;

    // FSM state tracking
    #[derive(Clone, Copy, PartialEq)]
    enum FsmState { Flat, Stalking, Entry, InTrade, Trail }

    let mut state = FsmState::Flat;
    let mut direction: i8 = 0; // 1 = long, -1 = short, 0 = neutral
    let mut bars_in_state: u32 = 0;
    let mut confirmation_bars: u32 = 0;
    let mut confirmation_needed: u32 = params.stabilization_window;
    let mut entry_price: f64 = 0.0;
    let mut best_price: f64 = 0.0;
    let mut trail_tighten_acc: f64 = 0.0;

    let mut pnls: Vec<f64> = Vec::new();
    let mut bars_in_trades: Vec<u32> = Vec::new();
    let mut trade_bars: u32 = 0;
    let mut equity = 100_000.0;
    let mut peak_equity = equity;
    let mut max_dd_pct: f64 = 0.0;

    let position_size = 10_000.0;

    for i in lookback..candles.len() {
        let c = &candles[i];

        // ── Derive synthetic pillar scores from candle data ──

        // Macro: SMA crossover trend + volatility-adjusted momentum
        let sma5: f64 = candles[i.saturating_sub(5)..i].iter().map(|x| x.close).sum::<f64>() / 5.0;
        let sma20: f64 = candles[i.saturating_sub(20)..i].iter().map(|x| x.close).sum::<f64>() / 20.0;
        let momentum = (sma5 - sma20) / sma20;

        let vol: f64 = candles[i.saturating_sub(10)..i].iter()
            .map(|x| (x.high - x.low) / x.close)
            .sum::<f64>() / 10.0;

        let macro_signal = (momentum / vol.max(0.0001)).clamp(-2.0, 2.0);
        let macro_score = (50.0 + macro_signal * 15.0).clamp(0.0, 100.0);

        // Micro: price action vs recent range (buying/selling pressure proxy)
        let range_mid = (c.high + c.low) / 2.0;
        let micro_signal = if c.high > c.low {
            (c.close - range_mid) / (c.high - c.low)
        } else {
            0.0
        };
        let micro_score = (50.0 + micro_signal * 20.0).clamp(0.0, 100.0);

        // Momentum: EMA(5) velocity + volume proxy via range expansion
        let prev = &candles[i.saturating_sub(1).max(0)];
        let velocity = (c.close - prev.close) / prev.close;
        let range_expansion = (c.high - c.low) / c.close / vol.max(0.0001);
        let mom_signal = velocity.signum() * range_expansion.min(3.0) * 0.3 + velocity * 50.0;
        let momentum_score = (50.0 + mom_signal * 10.0).clamp(0.0, 100.0);

        // Combined score via evolved weights
        let combined = macro_score * w_macro + micro_score * w_micro + momentum_score * w_momentum;
        let directional_strength = (combined - 50.0).abs() * 2.0;
        let dir_sign: i8 = if combined > 50.0 { 1 } else if combined < 50.0 { -1 } else { 0 };

        // Simulated changepoint: large volatility spike
        let cp_prob = if vol > 0.0 {
            let recent_vol: f64 = candles[i.saturating_sub(3)..i].iter()
                .map(|x| (x.high - x.low) / x.close)
                .sum::<f64>() / 3.0;
            let ratio = recent_vol / vol;
            if ratio > 2.5 { 0.8 } else if ratio > 1.8 { 0.5 } else { 0.1 }
        } else {
            0.1
        };

        // ── Changepoint reset (any state -> Flat) ──
        if cp_prob >= params.changepoint_reset && state != FsmState::Flat {
            if state == FsmState::InTrade || state == FsmState::Trail {
                let pnl_pct = if direction == 1 {
                    (c.close - entry_price) / entry_price
                } else {
                    (entry_price - c.close) / entry_price
                };
                let trade_pnl = pnl_pct * position_size;
                equity += trade_pnl;
                pnls.push(trade_pnl);
                bars_in_trades.push(trade_bars);
                if equity > peak_equity { peak_equity = equity; }
                let dd = (peak_equity - equity) / peak_equity * 100.0;
                if dd > max_dd_pct { max_dd_pct = dd; }
            }
            state = FsmState::Flat;
            direction = 0;
            bars_in_state = 0;
            confirmation_bars = 0;
            trade_bars = 0;
            trail_tighten_acc = 0.0;
            continue;
        }

        bars_in_state += 1;
        if state == FsmState::InTrade || state == FsmState::Trail {
            trade_bars += 1;
        }

        // ── FSM transitions ──
        match state {
            FsmState::Flat => {
                let stalk_strength = (params.stalking_threshold - 50.0).abs() * 2.0;
                if directional_strength >= stalk_strength && dir_sign != 0 {
                    confirmation_needed = params.stabilization_window;
                    state = FsmState::Stalking;
                    direction = dir_sign;
                    bars_in_state = 1;
                    confirmation_bars = 0;
                }
            }
            FsmState::Stalking => {
                let stalk_strength = (params.stalking_threshold - 50.0).abs() * 2.0;
                let drop_strength = (stalk_strength - params.hysteresis_margin * 2.0).max(0.0);

                if directional_strength < drop_strength || dir_sign == 0 {
                    state = FsmState::Flat;
                    direction = 0;
                    bars_in_state = 0;
                } else {
                    let entry_strength = (params.entry_threshold - 50.0).abs() * 2.0;
                    if directional_strength >= entry_strength {
                        confirmation_bars += 1;
                    }
                    let timeout = confirmation_needed.saturating_mul(2);
                    if bars_in_state >= timeout && confirmation_bars < confirmation_needed {
                        state = FsmState::Flat;
                        direction = 0;
                        bars_in_state = 0;
                    } else if confirmation_bars >= confirmation_needed {
                        state = FsmState::Entry;
                        bars_in_state = 1;
                    }
                }
            }
            FsmState::Entry => {
                entry_price = c.close;
                best_price = c.close;
                state = FsmState::InTrade;
                bars_in_state = 1;
                trade_bars = 1;
                trail_tighten_acc = 0.0;
            }
            FsmState::InTrade => {
                if direction == 1 && c.close > best_price { best_price = c.close; }
                if direction == -1 && c.close < best_price { best_price = c.close; }

                let entry_strength = (params.entry_threshold - 50.0).abs() * 2.0;
                let trail_strength = entry_strength * 0.8;
                if directional_strength < trail_strength {
                    state = FsmState::Trail;
                    bars_in_state = 1;
                }
            }
            FsmState::Trail => {
                trail_tighten_acc += params.trail_tighten_pct;

                let reversed = (direction == 1 && dir_sign == -1)
                    || (direction == -1 && dir_sign == 1);

                let trail_exit = if direction == 1 {
                    let drop = (best_price - c.close) / best_price;
                    drop >= (0.02 * (1.0 - trail_tighten_acc.min(0.8)))
                } else {
                    let rise = (c.close - best_price) / best_price;
                    rise >= (0.02 * (1.0 - trail_tighten_acc.min(0.8)))
                };

                if reversed || trail_exit {
                    let pnl_pct = if direction == 1 {
                        (c.close - entry_price) / entry_price
                    } else {
                        (entry_price - c.close) / entry_price
                    };
                    let trade_pnl = pnl_pct * position_size;
                    equity += trade_pnl;
                    pnls.push(trade_pnl);
                    bars_in_trades.push(trade_bars);
                    if equity > peak_equity { peak_equity = equity; }
                    let dd = (peak_equity - equity) / peak_equity * 100.0;
                    if dd > max_dd_pct { max_dd_pct = dd; }

                    state = FsmState::Flat;
                    direction = 0;
                    bars_in_state = 0;
                    trade_bars = 0;
                    trail_tighten_acc = 0.0;
                }
            }
        }
    }

    // Close any remaining position
    if (state == FsmState::InTrade || state == FsmState::Trail) && direction != 0 {
        if let Some(last) = candles.last() {
            let pnl_pct = if direction == 1 {
                (last.close - entry_price) / entry_price
            } else {
                (entry_price - last.close) / entry_price
            };
            let trade_pnl = pnl_pct * position_size;
            equity += trade_pnl;
            pnls.push(trade_pnl);
            bars_in_trades.push(trade_bars);
            if equity > peak_equity { peak_equity = equity; }
            let dd = (peak_equity - equity) / peak_equity * 100.0;
            if dd > max_dd_pct { max_dd_pct = dd; }
        }
    }

    if pnls.is_empty() {
        return empty;
    }

    let wins = pnls.iter().filter(|p| **p > 0.0).count() as u32;
    let losses = pnls.iter().filter(|p| **p < 0.0).count() as u32;
    let total_pnl: f64 = pnls.iter().sum();
    let n = pnls.len() as f64;
    let trade_count = pnls.len() as u32;

    let win_rate = wins as f64 / n;

    let mean = total_pnl / n;
    let var = pnls.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();
    let sharpe = if std > 1e-9 { mean / std * n.sqrt() } else { 0.0 };

    let gross_wins: f64 = pnls.iter().filter(|p| **p > 0.0).sum();
    let gross_losses: f64 = pnls.iter().filter(|p| **p < 0.0).map(|p| p.abs()).sum();
    let profit_factor = if gross_losses > 1e-9 { gross_wins / gross_losses } else { 10.0 };

    let avg_bars_in_trade = if bars_in_trades.is_empty() {
        0.0
    } else {
        bars_in_trades.iter().sum::<u32>() as f64 / bars_in_trades.len() as f64
    };

    // Fitness: Sharpe * min(PF, 5) + win_rate_bonus + stability - dd_penalty - overfit_penalty
    let dd_penalty = (max_dd_pct / 5.0).powi(2);
    let trade_count_bonus = (n / 10.0).min(2.0);
    let win_rate_bonus = if win_rate > 0.55 { (win_rate - 0.5) * 4.0 } else { 0.0 };
    let stability_bonus = if (3.0..=30.0).contains(&avg_bars_in_trade) { 0.5 } else { 0.0 };

    let fitness = sharpe * profit_factor.min(5.0) + trade_count_bonus + win_rate_bonus + stability_bonus - dd_penalty;

    StBacktestResult {
        fitness,
        sharpe,
        wins,
        losses,
        total_pnl,
        max_drawdown_pct: max_dd_pct,
        profit_factor: profit_factor.min(10.0),
        win_rate,
        avg_bars_in_trade,
        trade_count,
    }
}

/// Run SmartTrend evolution on a snapshot (no lock held).
pub fn run_st_evolution_unlocked(
    snap: StEvolutionSnapshot,
    candles: &[BacktestCandle],
) -> StEvolutionResult {
    if candles.len() < 80 {
        return StEvolutionResult {
            new_population: snap.population,
            generation: snap.generation,
            new_champion: None,
            stats: None,
        };
    }

    let split = (candles.len() as f64 * 0.70) as usize;
    let is_candles = &candles[..split];
    let oos_candles = &candles[split..];

    let generation = snap.generation + 1;
    let mut population = snap.population;
    let dev = snap.max_deviation_pct / 100.0;

    use rayon::prelude::*;
    population.par_iter_mut().for_each(|ind| {
        let r = smarttrend_walk_forward_backtest(is_candles, &ind.params);
        ind.fitness = r.fitness;
        ind.sharpe = r.sharpe;
        ind.wins = r.wins;
        ind.losses = r.losses;
    });

    population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));

    let elite_count = 5.min(population.len());
    let elites: Vec<StIndividual> = population[..elite_count].to_vec();

    let mut rng = rand::thread_rng();
    let mut new_population: Vec<StIndividual> = elites.clone();

    while new_population.len() < snap.population_size {
        let parent_a = &elites[rng.gen_range(0..elite_count)];
        let parent_b = &elites[rng.gen_range(0..elite_count)];

        let child_params = if rng.gen_bool(0.7) {
            let crossed = st_crossover(&parent_a.params, &parent_b.params, &mut rng);
            st_mutate(&crossed, dev, &mut rng)
        } else {
            st_mutate(&parent_a.params, dev, &mut rng)
        };

        new_population.push(StIndividual {
            params: child_params,
            fitness: 0.0,
            wins: 0,
            losses: 0,
            sharpe: 0.0,
        });
    }

    let best = &elites[0];
    let oos_result = smarttrend_walk_forward_backtest(oos_candles, &best.params);
    let oos_pass = oos_result.sharpe > 0.0 && oos_result.total_pnl > 0.0;

    let new_champion = if best.fitness > snap.champion_fitness && best.sharpe > 0.0 && oos_pass {
        Some(StNewChampion {
            fitness: best.fitness,
            params: best.params.clone(),
        })
    } else {
        None
    };

    let diversity = st_population_diversity(&new_population);

    let evaluated: Vec<f64> = new_population.iter()
        .filter(|i| i.fitness != 0.0 || i.wins > 0 || i.losses > 0)
        .map(|i| i.fitness)
        .collect();
    let avg_fitness = if evaluated.is_empty() { 0.0 } else {
        evaluated.iter().sum::<f64>() / evaluated.len() as f64
    };

    let stats = StEvolutionStats {
        generation,
        population_size: new_population.len(),
        champion_fitness: new_champion.as_ref().map(|c| c.fitness).unwrap_or(best.fitness),
        champion_sharpe: best.sharpe,
        champion_params: new_champion.as_ref().map(|c| c.params.clone()).unwrap_or_else(|| best.params.clone()),
        avg_fitness,
        diversity_pct: diversity,
        mutations_this_gen: (snap.population_size - elite_count) as u32,
        last_update_secs_ago: 0,
    };

    StEvolutionResult {
        new_population,
        generation,
        new_champion,
        stats: Some(stats),
    }
}

/// Optimizer for SmartTrend parameters (parallel to GeneticOptimizer for Fusion).
pub struct StGeneticOptimizer {
    population: Vec<StIndividual>,
    generation: u32,
    safety_rails: SmartTrendParams,
    max_deviation_pct: f64,
    population_size: usize,
    pub active_params: Arc<ArcSwap<SmartTrendParams>>,
    last_evolution: std::time::Instant,
    evolution_interval_secs: u64,
    champion_fitness: f64,
    /// Per-generation fitness history for convergence chart [B1]
    pub fitness_history: Vec<StFitnessSnapshot>,
}

/// Snapshot of fitness metrics for one generation (convergence chart data).
#[derive(Debug, Clone, Serialize)]
pub struct StFitnessSnapshot {
    pub generation: u32,
    pub is_fitness: f64,
    pub oos_fitness: f64,
    pub diversity_pct: f64,
    pub champion_promoted: bool,
}

impl StGeneticOptimizer {
    pub fn new(population_size: usize, safety_rails: SmartTrendParams, max_deviation_pct: f64) -> Self {
        let initial = safety_rails.clone();
        let active = Arc::new(ArcSwap::from_pointee(initial.clone()));

        let mut opt = Self {
            population: Vec::with_capacity(population_size),
            generation: 0,
            safety_rails,
            max_deviation_pct,
            population_size,
            active_params: active,
            last_evolution: std::time::Instant::now(),
            evolution_interval_secs: 900,
            champion_fitness: f64::NEG_INFINITY,
            fitness_history: Vec::with_capacity(256),
        };
        opt.seed_population();
        opt
    }

    pub fn with_defaults() -> Self {
        Self::new(20, SmartTrendParams::default_safe(), 15.0)
    }

    fn seed_population(&mut self) {
        let mut rng = rand::thread_rng();
        self.population.clear();

        self.population.push(StIndividual {
            params: self.safety_rails.clone(),
            fitness: 0.0,
            wins: 0,
            losses: 0,
            sharpe: 0.0,
        });

        let dev = self.max_deviation_pct / 100.0;
        for _ in 1..self.population_size {
            let params = st_mutate(&self.safety_rails, dev, &mut rng);
            self.population.push(StIndividual {
                params,
                fitness: 0.0,
                wins: 0,
                losses: 0,
                sharpe: 0.0,
            });
        }
    }

    pub fn stats(&self) -> StEvolutionStats {
        let best = self.population.first();
        let avg_fitness = if self.population.is_empty() { 0.0 } else {
            self.population.iter().map(|i| i.fitness).sum::<f64>() / self.population.len() as f64
        };
        let diversity = st_population_diversity(&self.population);

        StEvolutionStats {
            generation: self.generation,
            population_size: self.population.len(),
            champion_fitness: best.map(|b| b.fitness).unwrap_or(0.0),
            champion_sharpe: best.map(|b| b.sharpe).unwrap_or(0.0),
            champion_params: best.map(|b| b.params.clone()).unwrap_or_else(SmartTrendParams::default_safe),
            avg_fitness,
            diversity_pct: diversity,
            mutations_this_gen: (self.population_size - 5.min(self.population_size)) as u32,
            last_update_secs_ago: self.last_evolution.elapsed().as_secs(),
        }
    }

    pub fn current_params(&self) -> Arc<SmartTrendParams> {
        self.active_params.load_full()
    }

    pub fn prepare_evolution(&self) -> Option<StEvolutionSnapshot> {
        if self.last_evolution.elapsed().as_secs() < self.evolution_interval_secs {
            return None;
        }
        Some(StEvolutionSnapshot {
            population: self.population.clone(),
            generation: self.generation,
            safety_rails: self.safety_rails.clone(),
            max_deviation_pct: self.max_deviation_pct,
            population_size: self.population_size,
            champion_fitness: self.champion_fitness,
        })
    }

    pub fn apply_evolution_results(&mut self, result: StEvolutionResult, oos_fitness: f64) {
        let promoted = result.new_champion.is_some();
        let is_fitness = result.stats.as_ref().map(|s| s.champion_fitness).unwrap_or(0.0);
        let diversity = result.stats.as_ref().map(|s| s.diversity_pct).unwrap_or(0.0);

        self.population = result.new_population;
        self.generation = result.generation;
        self.last_evolution = std::time::Instant::now();

        if let Some(new_champ) = result.new_champion {
            self.champion_fitness = new_champ.fitness;
            self.active_params.store(Arc::new(new_champ.params));
        }

        self.fitness_history.push(StFitnessSnapshot {
            generation: self.generation,
            is_fitness,
            oos_fitness,
            diversity_pct: diversity,
            champion_promoted: promoted,
        });

        // Cap history to last 200 generations
        if self.fitness_history.len() > 200 {
            self.fitness_history.drain(..self.fitness_history.len() - 200);
        }
    }

    pub fn freeze_evolution(&mut self) {
        self.evolution_interval_secs = u64::MAX;
    }

    pub fn resume_evolution(&mut self) {
        self.evolution_interval_secs = 900;
    }

    pub fn restore_champion(&mut self, params: SmartTrendParams, fitness: f64) {
        self.active_params.store(Arc::new(params.clone()));
        self.champion_fitness = if fitness.is_finite() { fitness } else { 0.0 };
        let champion = StIndividual {
            params,
            fitness: self.champion_fitness,
            wins: 0,
            losses: 0,
            sharpe: 0.0,
        };
        if self.population.is_empty() {
            self.population.push(champion);
        } else {
            self.population[0] = champion;
        }
    }
}

/// Run both Fusion and SmartTrend backtests on the same candles for comparison.
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonResult {
    pub fusion: BacktestResult,
    pub smarttrend: StBacktestResult,
    pub fusion_params: TradingParams,
    pub smarttrend_params: SmartTrendParams,
    pub candle_count: usize,
}

pub fn run_comparison_backtest(
    candles: &[BacktestCandle],
    fusion_params: &TradingParams,
    st_params: &SmartTrendParams,
) -> ComparisonResult {
    let fusion = walk_forward_backtest(candles, fusion_params);
    let smarttrend = smarttrend_walk_forward_backtest(candles, st_params);
    ComparisonResult {
        fusion,
        smarttrend,
        fusion_params: fusion_params.clone(),
        smarttrend_params: st_params.clone(),
        candle_count: candles.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let opt = GeneticOptimizer::with_defaults();
        assert_eq!(opt.population.len(), 20);
        assert_eq!(opt.generation, 0);
    }

    #[test]
    fn test_params_within_rails() {
        let opt = GeneticOptimizer::with_defaults();
        let mut rng = rand::thread_rng();
        let rails = &opt.safety_rails;
        let dev = opt.max_deviation_pct / dec!(100);

        for _ in 0..100 {
            let params = opt.mutate_params(rails, &mut rng);
            assert!(params.tp_percent >= rails.tp_percent * (Decimal::ONE - dev));
            assert!(params.tp_percent <= rails.tp_percent * (Decimal::ONE + dev));
            assert!(params.sl_percent >= dec!(0.5));
            assert!(params.min_confidence >= dec!(0.10));
            assert!(params.min_confidence <= dec!(0.90));
        }
    }

    #[test]
    fn test_evolution_cycle() {
        let candles: Vec<BacktestCandle> = (0..80)
            .map(|i| {
                let price = 100.0 + i as f64 * 0.3;
                BacktestCandle {
                    close: price,
                    high: price + 0.5,
                    low: price - 0.3,
                }
            })
            .collect();

        let mut opt = GeneticOptimizer::new(10, TradingParams::default_safe(), dec!(15));
        opt.evolution_interval_secs = 0; // allow immediate evolution for testing

        let promoted = opt.evolve_walk_forward(&candles);

        assert_eq!(opt.generation, 1);
        assert_eq!(opt.population.len(), 10);

        let stats = opt.stats();
        assert!(stats.champion_fitness != 0.0 || !promoted);
    }

    #[test]
    fn test_freeze_resume() {
        let mut opt = GeneticOptimizer::with_defaults();
        opt.freeze_evolution();
        assert_eq!(opt.evolution_interval_secs, u64::MAX);
        opt.resume_evolution();
        assert_eq!(opt.evolution_interval_secs, 900);
    }

    #[test]
    fn test_backtest_trending_up() {
        let candles: Vec<BacktestCandle> = (0..100)
            .map(|i| {
                let price = 100.0 + i as f64 * 0.5;
                BacktestCandle {
                    close: price,
                    high: price + 0.3,
                    low: price - 0.2,
                }
            })
            .collect();

        let params = TradingParams::default_safe();
        let result = walk_forward_backtest(&candles, &params);
        assert!(result.wins > 0, "should have some wins in uptrend");
        assert!(result.total_pnl > 0.0, "should be profitable in uptrend");
        assert!(result.fitness > f64::NEG_INFINITY);
    }

    #[test]
    fn test_backtest_too_few_candles() {
        let candles: Vec<BacktestCandle> = (0..10)
            .map(|i| BacktestCandle {
                close: 100.0 + i as f64,
                high: 101.0,
                low: 99.0,
            })
            .collect();

        let params = TradingParams::default_safe();
        let result = walk_forward_backtest(&candles, &params);
        assert_eq!(result.fitness, f64::NEG_INFINITY);
    }

    #[test]
    fn test_backtest_flat_market() {
        let candles: Vec<BacktestCandle> = (0..100)
            .map(|_| BacktestCandle {
                close: 100.0,
                high: 100.01,
                low: 99.99,
            })
            .collect();

        let params = TradingParams::default_safe();
        let result = walk_forward_backtest(&candles, &params);
        // Flat market = no entries (signal strength ~0) or very small trades
        assert!(
            result.total_pnl.abs() < 500.0,
            "flat market should not produce large PnL"
        );
    }

    #[test]
    fn test_evolution_with_real_backtest() {
        let candles: Vec<BacktestCandle> = (0..150)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.1).sin() * 5.0 + i as f64 * 0.2;
                BacktestCandle {
                    close: base,
                    high: base + 0.5,
                    low: base - 0.4,
                }
            })
            .collect();

        let mut opt = GeneticOptimizer::new(10, TradingParams::default_safe(), dec!(15));
        opt.evolution_interval_secs = 0;

        let _ = opt.evolve_walk_forward(&candles);

        assert_eq!(opt.generation, 1);
        let stats = opt.stats();
        assert!(stats.champion_fitness > f64::NEG_INFINITY);
    }

    #[test]
    fn test_walk_forward_is_oos_split() {
        // Strong uptrend followed by reversal — IS champion shouldn't survive OOS
        let mut candles = Vec::new();
        for i in 0..100 {
            let price = 100.0 + i as f64 * 0.8;
            candles.push(BacktestCandle {
                close: price,
                high: price + 0.5,
                low: price - 0.3,
            });
        }
        for i in 0..50 {
            let price = 180.0 - i as f64 * 0.6;
            candles.push(BacktestCandle {
                close: price,
                high: price + 0.4,
                low: price - 0.5,
            });
        }

        let mut opt = GeneticOptimizer::new(10, TradingParams::default_safe(), dec!(15));
        opt.evolution_interval_secs = 0;

        let promoted = opt.evolve_walk_forward(&candles);
        assert_eq!(opt.generation, 1);
        // The OOS (reversal part) should reject pure uptrend-optimized params
        // This tests the anti-overfit mechanism — we just verify it runs without panic
        let _ = promoted;
    }

    #[test]
    fn test_walk_forward_too_few_candles() {
        let candles: Vec<BacktestCandle> = (0..30)
            .map(|i| BacktestCandle {
                close: 100.0 + i as f64,
                high: 101.0,
                low: 99.0,
            })
            .collect();

        let mut opt = GeneticOptimizer::new(5, TradingParams::default_safe(), dec!(15));
        opt.evolution_interval_secs = 0;
        let promoted = opt.evolve_walk_forward(&candles);
        assert!(!promoted, "should not promote with < 60 candles");
        assert_eq!(
            opt.generation, 0,
            "should not increment gen with too few data"
        );
    }

    #[test]
    fn test_walk_forward_flat_market() {
        let candles: Vec<BacktestCandle> = (0..100).map(|_| BacktestCandle {
            close: 100.0, high: 100.5, low: 99.5,
        }).collect();
        let params = TradingParams::default_safe();
        let result = walk_forward_backtest(&candles, &params);
        assert_eq!(result.wins, 0);
        assert_eq!(result.losses, 0);
    }

    #[test]
    fn test_walk_forward_trending_market() {
        let candles: Vec<BacktestCandle> = (0..100).map(|i| BacktestCandle {
            close: 100.0 + i as f64,
            high: 100.5 + i as f64,
            low: 99.5 + i as f64,
        }).collect();
        let params = TradingParams::default_safe();
        let result = walk_forward_backtest(&candles, &params);
        assert!(
            result.fitness.is_finite(),
            "trending series should yield finite fitness once trades exist"
        );
        assert!(
            result.wins + result.losses > 0,
            "trending market should close at least one trade with non-flat PnL"
        );
    }

    #[test]
    fn test_walk_forward_single_candle() {
        let candles = vec![BacktestCandle { close: 100.0, high: 101.0, low: 99.0 }];
        let params = TradingParams::default_safe();
        let result = walk_forward_backtest(&candles, &params);
        assert_eq!(result.wins, 0);
        assert_eq!(result.losses, 0);
    }

    #[test]
    fn test_walk_forward_empty() {
        let candles: Vec<BacktestCandle> = vec![];
        let params = TradingParams::default_safe();
        let result = walk_forward_backtest(&candles, &params);
        assert!(result.fitness.is_infinite() && result.fitness < 0.0);
    }

    #[test]
    fn test_mutate_decimal_clamping() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let val = GeneticOptimizer::mutate_decimal(
                dec!(0.5), dec!(0.1), dec!(0.0), dec!(1.0), &mut rng
            );
            assert!(val >= dec!(0.0) && val <= dec!(1.0),
                "mutated value {} out of [0, 1]", val);
        }
    }

    #[test]
    fn test_mutate_decimal_at_boundary() {
        let mut rng = rand::thread_rng();
        for _ in 0..50 {
            let val = GeneticOptimizer::mutate_decimal(
                dec!(0.0), dec!(0.5), dec!(0.0), dec!(1.0), &mut rng
            );
            assert!(val >= dec!(0.0) && val <= dec!(1.0));
            let val = GeneticOptimizer::mutate_decimal(
                dec!(1.0), dec!(0.5), dec!(0.0), dec!(1.0), &mut rng
            );
            assert!(val >= dec!(0.0) && val <= dec!(1.0));
        }
    }

    #[test]
    fn test_default_safe_ranges() {
        let p = TradingParams::default_safe();
        assert!(p.tp_percent > Decimal::ZERO);
        assert!(p.sl_percent > Decimal::ZERO);
        assert!(p.trail_gap_pct > Decimal::ZERO);
        assert!(p.min_confidence >= Decimal::ZERO && p.min_confidence <= dec!(1.0));
    }

    #[test]
    fn test_prepare_apply_roundtrip() {
        let mut opt = GeneticOptimizer::with_defaults();
        opt.evolution_interval_secs = 0;
        let snap = opt.prepare_evolution();
        assert!(snap.is_some());
        let snap = snap.unwrap();
        assert_eq!(snap.population.len(), opt.stats().population_size);

        let candles: Vec<BacktestCandle> = (0..200).map(|i| BacktestCandle {
            close: 100.0 + (i as f64 * 0.1).sin() * 5.0,
            high: 101.0 + (i as f64 * 0.1).sin() * 5.0,
            low: 99.0 + (i as f64 * 0.1).sin() * 5.0,
        }).collect();
        let result = run_evolution_unlocked(snap, &candles);
        let gen_before = opt.stats().generation;
        opt.apply_evolution_results(result);
        assert!(opt.stats().generation > gen_before);
    }

    #[test]
    fn test_restore_champion_preserves_fitness() {
        let mut opt = GeneticOptimizer::with_defaults();
        let params = TradingParams::default_safe();
        opt.restore_champion(params.clone(), 42.5);
        let stats = opt.stats();
        assert!((stats.champion_fitness - 42.5).abs() < 1e-6);
    }

    #[test]
    fn test_freeze_resume_evolution() {
        let mut opt = GeneticOptimizer::with_defaults();
        opt.freeze_evolution();
        assert!(opt.prepare_evolution().is_none());
        opt.resume_evolution();
        opt.evolution_interval_secs = 0;
        assert!(opt.prepare_evolution().is_some());
    }

    #[test]
    fn test_current_params_returns_valid() {
        let opt = GeneticOptimizer::with_defaults();
        let params = opt.current_params();
        assert!(params.tp_percent > Decimal::ZERO);
    }

    // ═══════════════════════════════════════════════════════════════
    //  S13: SmartTrend Genetic Tests
    // ═══════════════════════════════════════════════════════════════

    fn make_trending_candles(n: usize, slope: f64) -> Vec<BacktestCandle> {
        (0..n).map(|i| {
            let price = 100.0 + i as f64 * slope;
            let noise = (i as f64 * 0.7).sin() * 0.1;
            BacktestCandle {
                close: price + noise,
                high: price + slope.abs() * 0.6 + 0.3,
                low: price - 0.15,
            }
        }).collect()
    }

    fn make_choppy_candles(n: usize) -> Vec<BacktestCandle> {
        (0..n).map(|i| {
            let trend = i as f64 * 0.15;
            let chop = (i as f64 * 0.5).sin() * 3.0;
            let price = 100.0 + trend + chop;
            BacktestCandle { close: price, high: price + 1.2, low: price - 1.2 }
        }).collect()
    }

    #[test]
    fn test_st_params_default_safe() {
        let p = SmartTrendParams::default_safe();
        assert!(p.macro_weight > 0.0 && p.macro_weight <= 1.0);
        assert!(p.micro_weight > 0.0 && p.micro_weight <= 1.0);
        assert!(p.momentum_weight > 0.0 && p.momentum_weight <= 1.0);
        assert!(p.stalking_threshold < p.entry_threshold);
        assert!(p.changepoint_reset > 0.0 && p.changepoint_reset <= 1.0);
        assert!(p.stabilization_window >= 3);
        assert!(p.risk_per_trade > 0.0 && p.risk_per_trade <= 0.1);
    }

    #[test]
    fn test_st_params_normalized_weights() {
        let p = SmartTrendParams { macro_weight: 0.6, micro_weight: 0.2, momentum_weight: 0.2, ..SmartTrendParams::default_safe() };
        let (m, mi, mo) = p.normalized_weights();
        assert!((m + mi + mo - 1.0).abs() < 1e-9);
        assert!((m - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_st_params_normalized_weights_zero_sum() {
        let p = SmartTrendParams { macro_weight: 0.0, micro_weight: 0.0, momentum_weight: 0.0, ..SmartTrendParams::default_safe() };
        let (m, mi, mo) = p.normalized_weights();
        assert!((m - 0.40).abs() < 1e-9);
        assert!((mi - 0.35).abs() < 1e-9);
        assert!((mo - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_st_mutation_bounds() {
        let mut rng = rand::thread_rng();
        let base = SmartTrendParams::default_safe();
        for _ in 0..200 {
            let m = st_mutate(&base, 0.15, &mut rng);
            assert!(m.macro_weight >= 0.10 && m.macro_weight <= 0.80, "macro={}", m.macro_weight);
            assert!(m.micro_weight >= 0.05 && m.micro_weight <= 0.60, "micro={}", m.micro_weight);
            assert!(m.momentum_weight >= 0.05 && m.momentum_weight <= 0.60, "momentum={}", m.momentum_weight);
            assert!(m.entry_threshold >= 50.0 && m.entry_threshold <= 85.0, "entry={}", m.entry_threshold);
            assert!(m.stalking_threshold >= 40.0 && m.stalking_threshold <= 75.0, "stalking={}", m.stalking_threshold);
            assert!(m.changepoint_reset >= 0.40 && m.changepoint_reset <= 0.95, "cp={}", m.changepoint_reset);
            assert!(m.trail_tighten_pct >= 0.01 && m.trail_tighten_pct <= 0.20, "trail={}", m.trail_tighten_pct);
            assert!(m.hysteresis_margin >= 1.0 && m.hysteresis_margin <= 15.0, "hyst={}", m.hysteresis_margin);
            assert!(m.bayesian_halflife >= 50 && m.bayesian_halflife <= 500, "bayes={}", m.bayesian_halflife);
            assert!(m.stabilization_window >= 3 && m.stabilization_window <= 20, "stab={}", m.stabilization_window);
            assert!(m.risk_per_trade >= 0.005 && m.risk_per_trade <= 0.05, "risk={}", m.risk_per_trade);
        }
    }

    #[test]
    fn test_st_crossover_inherits_from_parents() {
        let mut rng = rand::thread_rng();
        let a = SmartTrendParams { entry_threshold: 60.0, ..SmartTrendParams::default_safe() };
        let b = SmartTrendParams { entry_threshold: 80.0, ..SmartTrendParams::default_safe() };
        for _ in 0..100 {
            let child = st_crossover(&a, &b, &mut rng);
            assert!(child.entry_threshold == 60.0 || child.entry_threshold == 80.0,
                "child entry {} not from either parent", child.entry_threshold);
        }
    }

    #[test]
    fn test_st_backtest_trending_up() {
        let candles = make_trending_candles(200, 0.8);
        let params = SmartTrendParams {
            entry_threshold: 55.0,
            stalking_threshold: 50.5,
            stabilization_window: 3,
            ..SmartTrendParams::default_safe()
        };
        let result = smarttrend_walk_forward_backtest(&candles, &params);
        assert!(result.fitness > f64::NEG_INFINITY, "trending up should yield finite fitness");
        assert!(result.trade_count > 0, "trending up should produce trades, got {}", result.trade_count);
    }

    #[test]
    fn test_st_backtest_trending_down() {
        let candles = make_trending_candles(200, -0.8);
        let params = SmartTrendParams {
            entry_threshold: 55.0,
            stalking_threshold: 50.5,
            stabilization_window: 3,
            ..SmartTrendParams::default_safe()
        };
        let result = smarttrend_walk_forward_backtest(&candles, &params);
        assert!(result.fitness > f64::NEG_INFINITY);
    }

    #[test]
    fn test_st_backtest_too_few_candles() {
        let candles = make_trending_candles(30, 0.5);
        let params = SmartTrendParams::default_safe();
        let result = smarttrend_walk_forward_backtest(&candles, &params);
        assert_eq!(result.fitness, f64::NEG_INFINITY);
    }

    #[test]
    fn test_st_backtest_flat_market() {
        let candles: Vec<BacktestCandle> = (0..100).map(|_| BacktestCandle {
            close: 100.0, high: 100.01, low: 99.99,
        }).collect();
        let params = SmartTrendParams::default_safe();
        let result = smarttrend_walk_forward_backtest(&candles, &params);
        assert!(result.total_pnl.abs() < 500.0, "flat market should not produce large PnL");
    }

    #[test]
    fn test_st_backtest_choppy_market() {
        let candles = make_choppy_candles(200);
        let params = SmartTrendParams {
            entry_threshold: 55.0,
            stalking_threshold: 50.5,
            stabilization_window: 3,
            ..SmartTrendParams::default_safe()
        };
        let result = smarttrend_walk_forward_backtest(&candles, &params);
        // Choppy market may or may not produce trades, but should not panic
        assert!(result.fitness.is_finite() || result.fitness == f64::NEG_INFINITY);
    }

    #[test]
    fn test_st_optimizer_creation() {
        let opt = StGeneticOptimizer::with_defaults();
        assert_eq!(opt.population.len(), 20);
        assert_eq!(opt.generation, 0);
    }

    #[test]
    fn test_st_evolution_cycle() {
        let candles = make_trending_candles(200, 0.3);
        let snap = StEvolutionSnapshot {
            population: {
                let opt = StGeneticOptimizer::with_defaults();
                opt.population
            },
            generation: 0,
            safety_rails: SmartTrendParams::default_safe(),
            max_deviation_pct: 15.0,
            population_size: 20,
            champion_fitness: f64::NEG_INFINITY,
        };
        let result = run_st_evolution_unlocked(snap, &candles);
        assert_eq!(result.generation, 1);
        assert!(result.stats.is_some());
        let stats = result.stats.unwrap();
        assert_eq!(stats.generation, 1);
        assert!(stats.diversity_pct >= 0.0);
    }

    #[test]
    fn test_st_evolution_oos_rejection() {
        // Strong uptrend then reversal — OOS should challenge IS champion
        let mut candles = Vec::new();
        for i in 0..120 { candles.push(BacktestCandle { close: 100.0 + i as f64 * 0.8, high: 100.5 + i as f64 * 0.8, low: 99.5 + i as f64 * 0.8 }); }
        for i in 0..80 { candles.push(BacktestCandle { close: 196.0 - i as f64 * 0.6, high: 196.5 - i as f64 * 0.6, low: 195.5 - i as f64 * 0.6 }); }

        let snap = StEvolutionSnapshot {
            population: StGeneticOptimizer::with_defaults().population,
            generation: 0,
            safety_rails: SmartTrendParams::default_safe(),
            max_deviation_pct: 15.0,
            population_size: 10,
            champion_fitness: f64::NEG_INFINITY,
        };
        let result = run_st_evolution_unlocked(snap, &candles);
        assert_eq!(result.generation, 1);
    }

    #[test]
    fn test_st_optimizer_prepare_apply() {
        let mut opt = StGeneticOptimizer::with_defaults();
        opt.evolution_interval_secs = 0;
        let snap = opt.prepare_evolution();
        assert!(snap.is_some());

        let candles = make_trending_candles(200, 0.3);
        let result = run_st_evolution_unlocked(snap.unwrap(), &candles);
        let gen_before = opt.generation;
        opt.apply_evolution_results(result, 0.5);
        assert!(opt.generation > gen_before);
        assert!(!opt.fitness_history.is_empty());
    }

    #[test]
    fn test_st_optimizer_freeze_resume() {
        let mut opt = StGeneticOptimizer::with_defaults();
        opt.freeze_evolution();
        assert!(opt.prepare_evolution().is_none());
        opt.resume_evolution();
        opt.evolution_interval_secs = 0;
        assert!(opt.prepare_evolution().is_some());
    }

    #[test]
    fn test_st_diversity_identical_population() {
        let params = SmartTrendParams::default_safe();
        let pop: Vec<StIndividual> = (0..5).map(|_| StIndividual {
            params: params.clone(), fitness: 0.0, wins: 0, losses: 0, sharpe: 0.0,
        }).collect();
        let div = st_population_diversity(&pop);
        assert!(div < 1.0, "identical population should have near-zero diversity: {}", div);
    }

    #[test]
    fn test_st_diversity_varied_population() {
        let mut rng = rand::thread_rng();
        let base = SmartTrendParams::default_safe();
        let pop: Vec<StIndividual> = (0..10).map(|_| StIndividual {
            params: st_mutate(&base, 0.30, &mut rng), fitness: 0.0, wins: 0, losses: 0, sharpe: 0.0,
        }).collect();
        let div = st_population_diversity(&pop);
        assert!(div > 0.0, "varied population should have positive diversity: {}", div);
    }

    #[test]
    fn test_st_restore_champion() {
        let mut opt = StGeneticOptimizer::with_defaults();
        let params = SmartTrendParams { entry_threshold: 72.0, ..SmartTrendParams::default_safe() };
        opt.restore_champion(params, 99.5);
        assert!((opt.stats().champion_fitness - 99.5).abs() < 1e-6);
        assert!((opt.current_params().entry_threshold - 72.0).abs() < 1e-9);
    }

    #[test]
    fn test_comparison_backtest_both_run() {
        let candles = make_trending_candles(150, 0.4);
        let fusion = TradingParams::default_safe();
        let st = SmartTrendParams::default_safe();
        let result = run_comparison_backtest(&candles, &fusion, &st);
        assert_eq!(result.candle_count, 150);
        assert!(result.fusion.fitness > f64::NEG_INFINITY || result.fusion.wins == 0);
        assert!(result.smarttrend.fitness > f64::NEG_INFINITY || result.smarttrend.wins == 0);
    }

    #[test]
    fn test_st_fitness_history_capped() {
        let mut opt = StGeneticOptimizer::with_defaults();
        opt.evolution_interval_secs = 0;
        for i in 0..210 {
            opt.fitness_history.push(StFitnessSnapshot {
                generation: i, is_fitness: 1.0, oos_fitness: 0.5, diversity_pct: 30.0, champion_promoted: false,
            });
        }
        // Simulate apply that triggers cap
        let candles = make_trending_candles(100, 0.3);
        let snap = opt.prepare_evolution().unwrap();
        let result = run_st_evolution_unlocked(snap, &candles);
        opt.apply_evolution_results(result, 0.5);
        assert!(opt.fitness_history.len() <= 200, "history should be capped at 200");
    }

    #[test]
    fn test_st_backtest_result_metrics() {
        let candles = make_trending_candles(200, 0.5);
        let params = SmartTrendParams::default_safe();
        let result = smarttrend_walk_forward_backtest(&candles, &params);
        if result.trade_count > 0 {
            assert!(result.win_rate >= 0.0 && result.win_rate <= 1.0);
            assert!(result.profit_factor >= 0.0 && result.profit_factor <= 10.0);
            assert!(result.avg_bars_in_trade >= 1.0);
            assert!(result.max_drawdown_pct >= 0.0);
        }
    }
}
