        (function () {
            let _hftSettingsLoaded = false;
            const hftSliders = [
                { id: 'hft-signal-min', field: 'entry_signal_strength_min', valId: 'hft-signal-min-val', fmt: v => v, div: 1 },
                { id: 'hft-ofi-thresh', field: 'min_ofi_threshold', valId: 'hft-ofi-thresh-val', fmt: v => v.toFixed(1), div: 1 },
                { id: 'hft-max-spread', field: 'max_spread_bps', valId: 'hft-max-spread-val', fmt: v => v, div: 1 },
                { id: 'hft-cooldown', field: 'entry_cooldown_secs', valId: 'hft-cooldown-val', fmt: v => v + 's', div: 1 },
                { id: 'hft-trail', field: 'trailing_stop_pct', valId: 'hft-trail-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-tp', field: 'take_profit_pct', valId: 'hft-tp-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-max-hold', field: 'max_hold_secs', valId: 'hft-max-hold-val', fmt: v => v >= 3600 ? (v / 3600).toFixed(1) + 'h' : (v / 60).toFixed(0) + 'm', div: 1 },
                { id: 'hft-risk', field: 'risk_per_trade', valId: 'hft-risk-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-max-pos', field: 'max_open_positions', valId: 'hft-max-pos-val', fmt: v => v, div: 1 },
                { id: 'hft-max-daily', field: 'max_daily_trades', valId: 'hft-max-daily-val', fmt: v => v, div: 1 },
                { id: 'hft-daily-loss', field: 'daily_loss_limit_pct', valId: 'hft-daily-loss-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
            ];

            const hftAdvancedSliders = [
                // Phase 1: Autopilot Core
                { id: 'hft-min-hold-secs', field: 'min_hold_secs', valId: 'hft-min-hold-secs-val', fmt: v => v + 's', div: 1 },
                { id: 'hft-nexus-consensus', field: 'nexus_consensus_threshold', valId: 'hft-nexus-consensus-val', fmt: v => v, div: 1 },
                { id: 'hft-loop-ms', field: 'autopilot_loop_ms', valId: 'hft-loop-ms-val', fmt: v => v, div: 1 },
                { id: 'hft-ofi-stale', field: 'ofi_stale_ms', valId: 'hft-ofi-stale-val', fmt: v => v, div: 1 },
                { id: 'hft-min-bars', field: 'min_bars_for_entry', valId: 'hft-min-bars-val', fmt: v => v, div: 1 },
                { id: 'hft-ofi-entry', field: 'ofi_entry_threshold', valId: 'hft-ofi-entry-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-max-qty', field: 'max_qty_cap', valId: 'hft-max-qty-val', fmt: v => v, div: 1 },
                { id: 'hft-min-qty', field: 'min_qty_threshold', valId: 'hft-min-qty-val', fmt: v => (v / 10000).toFixed(4), div: 10000 },
                { id: 'hft-equity-fb', field: 'default_equity_fallback', valId: 'hft-equity-fb-val', fmt: v => v, div: 1 },
                { id: 'hft-bypass-risk', field: 'max_bypass_risk_pct', valId: 'hft-bypass-risk-val', fmt: v => v + '%', div: 1 },
                // Phase 2: Exit Scaling
                { id: 'hft-vol-min', field: 'vol_factor_min', valId: 'hft-vol-min-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-vol-max', field: 'vol_factor_max', valId: 'hft-vol-max-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-fb-sl', field: 'fallback_sl_pct', valId: 'hft-fb-sl-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-fb-tp', field: 'fallback_tp_pct', valId: 'hft-fb-tp-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-fb-trail', field: 'fallback_trail_pct', valId: 'hft-fb-trail-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-scalp-stop', field: 'scalping_stop_mult', valId: 'hft-scalp-stop-val', fmt: v => (v / 10).toFixed(2), div: 10 },
                { id: 'hft-scalp-take', field: 'scalping_take_mult', valId: 'hft-scalp-take-val', fmt: v => (v / 10).toFixed(2), div: 10 },
                { id: 'hft-scalp-trail', field: 'scalping_trail_mult', valId: 'hft-scalp-trail-val', fmt: v => (v / 10).toFixed(2), div: 10 },
                { id: 'hft-partial-frac', field: 'partial_first_target_frac', valId: 'hft-partial-frac-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-partial-minq', field: 'partial_min_qty', valId: 'hft-partial-minq-val', fmt: v => (v / 10000).toFixed(4), div: 10000 },
                { id: 'hft-atr-period', field: 'atr_period', valId: 'hft-atr-period-val', fmt: v => v, div: 1 },
                { id: 'hft-atr-smin', field: 'atr_stop_clamp_min', valId: 'hft-atr-smin-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-atr-smax', field: 'atr_stop_clamp_max', valId: 'hft-atr-smax-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-atr-tmin', field: 'atr_take_clamp_min', valId: 'hft-atr-tmin-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-atr-tmax', field: 'atr_take_clamp_max', valId: 'hft-atr-tmax-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-atr-trmin', field: 'atr_trail_clamp_min', valId: 'hft-atr-trmin-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-atr-trmax', field: 'atr_trail_clamp_max', valId: 'hft-atr-trmax-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                // Phase 3: Oracle Exit
                { id: 'hft-oexit-conf', field: 'oracle_exit_min_confidence', valId: 'hft-oexit-conf-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-oexit-score', field: 'oracle_exit_min_score', valId: 'hft-oexit-score-val', fmt: v => v, div: 1 },
                { id: 'hft-oflip-score', field: 'oracle_flip_score_threshold', valId: 'hft-oflip-score-val', fmt: v => v, div: 1 },
                { id: 'hft-oflip-conf', field: 'oracle_flip_confidence', valId: 'hft-oflip-conf-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-chaos-loss', field: 'chaos_exit_max_loss_pct', valId: 'hft-chaos-loss-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-chaos-conf', field: 'chaos_exit_min_confidence', valId: 'hft-chaos-conf-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                // Phase 4: Circuit Breaker
                { id: 'hft-brk-caution', field: 'breaker_caution_pct', valId: 'hft-brk-caution-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-brk-halt', field: 'breaker_halt_pct', valId: 'hft-brk-halt-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-brk-panic', field: 'breaker_panic_pct', valId: 'hft-brk-panic-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-max-notional', field: 'max_notional_equity_frac', valId: 'hft-max-notional-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-min-stop-frac', field: 'min_stop_distance_frac', valId: 'hft-min-stop-frac-val', fmt: v => (v / 10000).toFixed(4), div: 10000 },
                // Phase 5: Strategy Scoring
                { id: 'hft-base-score', field: 'strategy_base_score', valId: 'hft-base-score-val', fmt: v => v, div: 1 },
                { id: 'hft-vpin-scalp', field: 'vpin_gate_scalping', valId: 'hft-vpin-scalp-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-vpin-normal', field: 'vpin_gate_normal', valId: 'hft-vpin-normal-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-min-rvol', field: 'min_rvol_threshold', valId: 'hft-min-rvol-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-rvol-win', field: 'rvol_window', valId: 'hft-rvol-win-val', fmt: v => v, div: 1 },
                { id: 'hft-rvol-relax', field: 'rvol_relax_threshold', valId: 'hft-rvol-relax-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-rvol-relaxv', field: 'rvol_relax_value', valId: 'hft-rvol-relaxv-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-chaos-kill', field: 'chaos_confidence_kill', valId: 'hft-chaos-kill-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-chaos-mult', field: 'chaos_regime_mult', valId: 'hft-chaos-mult-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                // Phase 6: Engine Stops
                { id: 'hft-long-stop-n', field: 'long_stop_normal', valId: 'hft-long-stop-n-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-long-stop-a', field: 'long_stop_aggressive', valId: 'hft-long-stop-a-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-short-stop', field: 'short_stop_pct', valId: 'hft-short-stop-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-long-trail-mp', field: 'long_trail_min_profit', valId: 'hft-long-trail-mp-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-short-trail-g', field: 'short_trail_gap', valId: 'hft-short-trail-g-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                { id: 'hft-short-trail-mp', field: 'short_trail_min_profit', valId: 'hft-short-trail-mp-val', fmt: v => (v / 10).toFixed(1) + '%', div: 10 },
                // Phase 7: Oracle Fusion
                { id: 'hft-ok-w', field: 'oracle_kalman_weight', valId: 'hft-ok-w-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-oe-w', field: 'oracle_entropy_weight', valId: 'hft-oe-w-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-oh-w', field: 'oracle_hurst_weight', valId: 'hft-oh-w-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-ov-w', field: 'oracle_vpin_weight', valId: 'hft-ov-w-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-ow-w', field: 'oracle_wavelet_weight', valId: 'hft-ow-w-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-oconf-floor', field: 'oracle_confidence_floor', valId: 'hft-oconf-floor-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-ent-chaos', field: 'entropy_chaos_threshold', valId: 'hft-ent-chaos-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'hft-vpin-toxic', field: 'vpin_toxicity_alert', valId: 'hft-vpin-toxic-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                // Phase 8: Indicator Periods
                { id: 'hft-ema-fast', field: 'ema_fast_period', valId: 'hft-ema-fast-val', fmt: v => v, div: 1 },
                { id: 'hft-ema-mid', field: 'ema_mid_period', valId: 'hft-ema-mid-val', fmt: v => v, div: 1 },
                { id: 'hft-ema-slow', field: 'ema_slow_period', valId: 'hft-ema-slow-val', fmt: v => v, div: 1 },
                { id: 'hft-ema-trend', field: 'ema_trend_period', valId: 'hft-ema-trend-val', fmt: v => v, div: 1 },
                { id: 'hft-rsi-per', field: 'rsi_period', valId: 'hft-rsi-per-val', fmt: v => v, div: 1 },
                { id: 'hft-macd-f', field: 'macd_fast', valId: 'hft-macd-f-val', fmt: v => v, div: 1 },
                { id: 'hft-macd-s', field: 'macd_slow', valId: 'hft-macd-s-val', fmt: v => v, div: 1 },
                { id: 'hft-macd-sig', field: 'macd_signal', valId: 'hft-macd-sig-val', fmt: v => v, div: 1 },
                { id: 'hft-bb-per', field: 'bb_period', valId: 'hft-bb-per-val', fmt: v => v, div: 1 },
                { id: 'hft-bb-std', field: 'bb_std_dev', valId: 'hft-bb-std-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'hft-adx-per', field: 'adx_period', valId: 'hft-adx-per-val', fmt: v => v, div: 1 },
                { id: 'hft-stoch-per', field: 'stoch_rsi_period', valId: 'hft-stoch-per-val', fmt: v => v, div: 1 },
                // Phase 9: Adaptive Exit Multipliers
                { id: 'hft-exit-floor', field: 'exit_floor_pct', valId: 'hft-exit-floor-val', fmt: v => (v / 100).toFixed(2) + '%', div: 100 },
                { id: 'hft-regime-chaotic-stop', field: 'regime_chaotic_stop_mult', valId: 'hft-regime-chaotic-stop-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-regime-chaotic-take', field: 'regime_chaotic_take_mult', valId: 'hft-regime-chaotic-take-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-regime-chaotic-trail', field: 'regime_chaotic_trail_mult', valId: 'hft-regime-chaotic-trail-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-regime-range-stop', field: 'regime_range_stop_mult', valId: 'hft-regime-range-stop-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-regime-range-take', field: 'regime_range_take_mult', valId: 'hft-regime-range-take-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-regime-range-trail', field: 'regime_range_trail_mult', valId: 'hft-regime-range-trail-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-regime-trend-stop', field: 'regime_trend_stop_mult', valId: 'hft-regime-trend-stop-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-regime-trend-take', field: 'regime_trend_take_mult', valId: 'hft-regime-trend-take-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-regime-trend-trail', field: 'regime_trend_trail_mult', valId: 'hft-regime-trend-trail-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-sig-align-stop', field: 'signal_aligned_stop_mult', valId: 'hft-sig-align-stop-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-sig-align-take', field: 'signal_aligned_take_mult', valId: 'hft-sig-align-take-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-sig-align-trail', field: 'signal_aligned_trail_mult', valId: 'hft-sig-align-trail-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-sig-oppose-stop', field: 'signal_oppose_stop_mult', valId: 'hft-sig-oppose-stop-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-sig-oppose-take', field: 'signal_oppose_take_mult', valId: 'hft-sig-oppose-take-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-tf-short-stop', field: 'tf_short_stop_mult', valId: 'hft-tf-short-stop-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-tf-short-take', field: 'tf_short_take_mult', valId: 'hft-tf-short-take-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                { id: 'hft-tf-short-trail', field: 'tf_short_trail_mult', valId: 'hft-tf-short-trail-val', fmt: v => (v / 100).toFixed(2) + 'x', div: 100 },
                // N4: Score Modifiers
                { id: 'sm-chaos-kill-penalty', field: 'sm_chaos_kill_penalty', valId: 'sm-chaos-kill-penalty-val', fmt: v => v, div: 1 },
                { id: 'sm-chaos-kill-fallback', field: 'sm_chaos_kill_fallback', valId: 'sm-chaos-kill-fallback-val', fmt: v => v, div: 1 },
                { id: 'sm-chaos-conf-mr', field: 'sm_chaos_conf_mult_mean_rev', valId: 'sm-chaos-conf-mr-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'sm-chaos-conf-ch', field: 'sm_chaos_conf_mult_chaotic', valId: 'sm-chaos-conf-ch-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'sm-chaos-conf-unk', field: 'sm_chaos_conf_mult_unknown', valId: 'sm-chaos-conf-unk-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'sm-regime-mr', field: 'sm_regime_mult_mean_rev', valId: 'sm-regime-mr-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'sm-regime-ch', field: 'sm_regime_mult_chaotic', valId: 'sm-regime-ch-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'sm-regime-unk', field: 'sm_regime_mult_unknown', valId: 'sm-regime-unk-val', fmt: v => (v / 100).toFixed(2), div: 100 },
                { id: 'sm-vpin-hi', field: 'sm_vpin_penalty_high', valId: 'sm-vpin-hi-val', fmt: v => v, div: 1 },
                { id: 'sm-vpin-med', field: 'sm_vpin_penalty_medium', valId: 'sm-vpin-med-val', fmt: v => v, div: 1 },
                { id: 'sm-fvg-bull', field: 'sm_fvg_bullish_bonus', valId: 'sm-fvg-bull-val', fmt: v => v, div: 1 },
                { id: 'sm-fvg-bear', field: 'sm_fvg_bearish_penalty', valId: 'sm-fvg-bear-val', fmt: v => v, div: 1 },
                { id: 'sm-rvol-t1-thr', field: 'sm_rvol_tier1_threshold', valId: 'sm-rvol-t1-thr-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'sm-rvol-t1-bon', field: 'sm_rvol_tier1_bonus', valId: 'sm-rvol-t1-bon-val', fmt: v => v, div: 1 },
                { id: 'sm-rvol-t2-thr', field: 'sm_rvol_tier2_threshold', valId: 'sm-rvol-t2-thr-val', fmt: v => (v / 10).toFixed(1), div: 10 },
                { id: 'sm-rvol-t2-bon', field: 'sm_rvol_tier2_bonus', valId: 'sm-rvol-t2-bon-val', fmt: v => v, div: 1 },
                { id: 'sm-stoch-ob', field: 'sm_stoch_rsi_overbought', valId: 'sm-stoch-ob-val', fmt: v => v, div: 1 },
                { id: 'sm-stoch-os', field: 'sm_stoch_rsi_oversold', valId: 'sm-stoch-os-val', fmt: v => v, div: 1 },
                { id: 'sm-bb-squeeze', field: 'sm_bb_squeeze_width', valId: 'sm-bb-squeeze-val', fmt: v => (v / 1000).toFixed(3), div: 1000 },
                { id: 'sm-label-strong', field: 'sm_entry_label_strong', valId: 'sm-label-strong-val', fmt: v => v, div: 1 },
                { id: 'sm-label-mild', field: 'sm_entry_label_mild', valId: 'sm-label-mild-val', fmt: v => v, div: 1 },
                // N4: Classical Bonuses
                { id: 'cb-rsi', field: 'cb_rsi_bonus', valId: 'cb-rsi-val', fmt: v => v, div: 1 },
                { id: 'cb-ema', field: 'cb_ema_alignment_bonus', valId: 'cb-ema-val', fmt: v => v, div: 1 },
                { id: 'cb-adx', field: 'cb_adx_bonus', valId: 'cb-adx-val', fmt: v => v, div: 1 },
                { id: 'cb-bb', field: 'cb_bb_squeeze_bonus', valId: 'cb-bb-val', fmt: v => v, div: 1 },
                { id: 'cb-vwap', field: 'cb_vwap_bonus', valId: 'cb-vwap-val', fmt: v => v, div: 1 },
                { id: 'cb-cap', field: 'cb_cap', valId: 'cb-cap-val', fmt: v => v, div: 1 },
            ];

            function wireHftEngine() {
                const allSliders = hftSliders.concat(hftAdvancedSliders);
                allSliders.forEach(s => {
                    const slider = $el(s.id);
                    const valEl = $el(s.valId);
                    if (!slider || !valEl) return;
                    slider.addEventListener('input', () => {
                        const raw = parseFloat(slider.value);
                        valEl.textContent = s.fmt(raw);
                    });
                });

                const btnSave = $el('btn-hft-save');
                if (btnSave) btnSave.addEventListener('click', async () => {
                    const batch = {};
                    for (const s of hftSliders.concat(hftAdvancedSliders)) {
                        const slider = $el(s.id);
                        if (!slider) continue;
                        const raw = parseFloat(slider.value);
                        batch[s.field] = String(s.div > 1 ? (raw / s.div) : raw);
                    }
                    batch.discord_webhook_url = $el('hft-discord-url')?.value || '';
                    batch.alert_on_fill = String($el('hft-alert-fill')?.checked || false);
                    batch.alert_on_breaker = String($el('hft-alert-breaker')?.checked || false);
                    batch.alert_on_daily_summary = String($el('hft-alert-daily')?.checked || false);
                    const corrSlider = $el('hft-max-corr');
                    if (corrSlider) batch.max_correlation = String(parseFloat(corrSlider.value) / 100);
                    batch.telegram_bot_token = $el('hft-tg-token')?.value || '';
                    batch.telegram_chat_id = $el('hft-tg-chatid')?.value || '';
                    batch.telegram_enabled = String($el('hft-tg-enabled')?.checked || false);
                    const atrStop = $el('hft-atr-stop');
                    if (atrStop) batch.atr_stop_multiplier = String(parseFloat(atrStop.value) / 10);
                    const atrTake = $el('hft-atr-take');
                    if (atrTake) batch.atr_take_multiplier = String(parseFloat(atrTake.value) / 10);
                    batch.partial_take_enabled = String($el('hft-partial-enabled')?.checked !== false);
                    const partialPct = $el('hft-partial-pct');
                    if (partialPct) batch.partial_take_pct = partialPct.value;
                    batch.llm_api_key = $el('hft-llm-key')?.value || '';
                    batch.llm_provider = $el('hft-llm-provider')?.value || 'openai';
                    batch.llm_sentiment_enabled = String($el('hft-llm-enabled')?.checked || false);
                    batch.options_hedging_enabled = String($el('hft-options-enabled')?.checked || false);
                    const hedgeOtm = $el('hft-hedge-otm');
                    if (hedgeOtm) batch.hedge_otm_pct = hedgeOtm.value;
                    const hedgePrem = $el('hft-hedge-premium');
                    if (hedgePrem) batch.hedge_max_premium_pct = String(parseFloat(hedgePrem.value) / 10);
                    batch.web_dashboard_enabled = String($el('hft-web-enabled')?.checked || false);
                    const webPort = $el('hft-web-port');
                    if (webPort) batch.web_dashboard_port = webPort.value;
                    batch.disable_adaptive_exits = String($el('hft-disable-adaptive')?.classList.contains('on') || false);
                    const multiOn = $el('hft-multi-asset-toggle')?.classList.contains('on') || false;
                    batch.multi_symbol_enabled = String(multiOn);
                    const maxConc = $el('hft-max-concurrent');
                    if (maxConc) batch.max_concurrent_assets = maxConc.value;
                    // TA Gates
                    const _gateIds = ['ta-gate-rsi','ta-gate-ema','ta-gate-adx','ta-gate-bb','ta-gate-macd','ta-gate-stoch'];
                    _gateIds.forEach(id => { const el = $el(id); if (el) batch[el.dataset.field] = el.value; });
                    const penEl = $el('ta-gate-penalty');
                    if (penEl) batch.ta_gate_soft_penalty = penEl.value;
                    // Portfolio Risk Gates (S9)
                    batch.enable_var_gate = String($el('hft-var-gate')?.checked || false);
                    const varMaxEl = $el('hft-var-max');
                    if (varMaxEl) batch.max_portfolio_var_pct = String(parseFloat(varMaxEl.value) / 10);
                    const varLookEl = $el('hft-var-lookback');
                    if (varLookEl) batch.var_lookback_periods = varLookEl.value;
                    batch.enable_concentration_gate = String($el('hft-hhi-gate')?.checked || false);
                    const hhiMaxEl = $el('hft-hhi-max');
                    if (hhiMaxEl) batch.max_concentration_hhi = String(parseFloat(hhiMaxEl.value) / 100);
                    // Safety Gates (S04.1)
                    batch.var_fail_closed = String($el('sg-var-fail-closed')?.checked ?? true);
                    batch.asset_check_hard_gate = String($el('sg-asset-check-hard')?.checked ?? true);
                    const sgBlF = $el('sg-blackout-first');
                    if (sgBlF) batch.blackout_first_mins = sgBlF.value;
                    const sgBlL = $el('sg-blackout-last');
                    if (sgBlL) batch.blackout_last_mins = sgBlL.value;
                    const sgExm = $el('sg-blackout-exempt');
                    if (sgExm) batch.blackout_exempt_symbols = sgExm.value;
                    const sgDV = $el('sg-min-dvol');
                    if (sgDV) batch.scan_min_dollar_volume = String((parseFloat(sgDV.value) / 10) * 1000000);
                    // S15 Trading Edge
                    batch.max_hold_decay_enabled = String(!!$el('chk-max-hold-decay')?.querySelector('.chk-box.checked'));
                    const s15ds = $el('s15-decay-start');
                    if (s15ds) batch.max_hold_decay_start_pct = String(parseInt(s15ds.value) / 100);
                    const s15dt = $el('s15-decay-tighten');
                    if (s15dt) batch.max_hold_decay_trail_tighten = String(parseInt(s15dt.value) / 1000);
                    batch.mtf_gate_enabled = String(!!$el('chk-mtf-gate')?.querySelector('.chk-box.checked'));
                    const s15mm = $el('s15-mtf-mode');
                    if (s15mm) batch.mtf_gate_mode = s15mm.value;
                    const s15mp = $el('s15-mtf-penalty');
                    if (s15mp) batch.mtf_gate_penalty = s15mp.value;
                    batch.hhi_gate_manual_orders = String(!!$el('chk-hhi-manual')?.querySelector('.chk-box.checked'));
                    const s15hl = $el('s15-hhi-limit');
                    if (s15hl) batch.max_concentration_hhi_manual = String(parseInt(s15hl.value) / 100);
                    batch.tiered_scan_enabled = String(!!$el('chk-tiered-scan')?.querySelector('.chk-box.checked'));
                    _collectEntryThresholdBatch(batch);
                    await invoke('batch_update_settings', { settings: batch });
                    _hftSettingsLoaded = false;
                    titanLogAdd('HFT Engine config saved');
                });

                // TA Gate selects: live-update on change
                document.querySelectorAll('.ta-gate-select').forEach(sel => {
                    sel.addEventListener('change', () => {
                        invoke('update_settings', { field: sel.dataset.field, value: sel.value });
                        const presetEl = $el('ta-gate-preset');
                        if (presetEl) presetEl.value = '';
                    });
                });
                const _taPresetEl = $el('ta-gate-preset');
                if (_taPresetEl) _taPresetEl.addEventListener('change', async () => {
                    if (!_taPresetEl.value) return;
                    await invoke('update_settings', { field: 'ta_gate_preset', value: _taPresetEl.value });
                    const snap = await invoke('get_settings_snapshot');
                    _restoreTaGates(snap);
                });
                const _taPenSlider = $el('ta-gate-penalty');
                const _taPenVal = $el('ta-gate-penalty-val');
                if (_taPenSlider) _taPenSlider.addEventListener('input', () => {
                    if (_taPenVal) _taPenVal.textContent = _taPenSlider.value;
                    invoke('update_settings', { field: 'ta_gate_soft_penalty', value: _taPenSlider.value });
                });

                // Portfolio Risk Gates (S9): live-update on change
                const _varGateEl = $el('hft-var-gate');
                if (_varGateEl) _varGateEl.addEventListener('change', () => {
                    invoke('update_settings', { field: 'enable_var_gate', value: String(_varGateEl.checked) });
                });
                const _varMaxSlider = $el('hft-var-max');
                const _varMaxVal = $el('hft-var-max-val');
                if (_varMaxSlider) _varMaxSlider.addEventListener('input', () => {
                    const v = (parseFloat(_varMaxSlider.value) / 10).toFixed(1);
                    if (_varMaxVal) _varMaxVal.textContent = v + '%';
                    invoke('update_settings', { field: 'max_portfolio_var_pct', value: v });
                });
                const _varLookSlider = $el('hft-var-lookback');
                const _varLookVal = $el('hft-var-lookback-val');
                if (_varLookSlider) _varLookSlider.addEventListener('input', () => {
                    if (_varLookVal) _varLookVal.textContent = _varLookSlider.value;
                    invoke('update_settings', { field: 'var_lookback_periods', value: _varLookSlider.value });
                });
                const _hhiGateEl = $el('hft-hhi-gate');
                if (_hhiGateEl) _hhiGateEl.addEventListener('change', () => {
                    invoke('update_settings', { field: 'enable_concentration_gate', value: String(_hhiGateEl.checked) });
                });
                const _hhiMaxSlider = $el('hft-hhi-max');
                const _hhiMaxVal = $el('hft-hhi-max-val');
                if (_hhiMaxSlider) _hhiMaxSlider.addEventListener('input', () => {
                    const v = (parseFloat(_hhiMaxSlider.value) / 100).toFixed(2);
                    if (_hhiMaxVal) _hhiMaxVal.textContent = v;
                    invoke('update_settings', { field: 'max_concentration_hhi', value: v });
                });

                // Safety Gates (S04.1): live-update on change
                const _sgVarFail = $el('sg-var-fail-closed');
                if (_sgVarFail) _sgVarFail.addEventListener('change', () => {
                    invoke('update_settings', { field: 'var_fail_closed', value: String(_sgVarFail.checked) });
                });
                const _sgAssetHard = $el('sg-asset-check-hard');
                if (_sgAssetHard) _sgAssetHard.addEventListener('change', () => {
                    invoke('update_settings', { field: 'asset_check_hard_gate', value: String(_sgAssetHard.checked) });
                });
                const _sgBlFirst = $el('sg-blackout-first');
                const _sgBlFirstVal = $el('sg-blackout-first-val');
                if (_sgBlFirst) _sgBlFirst.addEventListener('input', () => {
                    if (_sgBlFirstVal) _sgBlFirstVal.textContent = _sgBlFirst.value + ' min';
                    invoke('update_settings', { field: 'blackout_first_mins', value: _sgBlFirst.value });
                });
                const _sgBlLast = $el('sg-blackout-last');
                const _sgBlLastVal = $el('sg-blackout-last-val');
                if (_sgBlLast) _sgBlLast.addEventListener('input', () => {
                    if (_sgBlLastVal) _sgBlLastVal.textContent = _sgBlLast.value + ' min';
                    invoke('update_settings', { field: 'blackout_last_mins', value: _sgBlLast.value });
                });
                const _sgExempt = $el('sg-blackout-exempt');
                if (_sgExempt) _sgExempt.addEventListener('change', () => {
                    invoke('update_settings', { field: 'blackout_exempt_symbols', value: _sgExempt.value });
                });
                const _sgDvol = $el('sg-min-dvol');
                const _sgDvolVal = $el('sg-min-dvol-val');
                if (_sgDvol) _sgDvol.addEventListener('input', () => {
                    const raw = parseFloat(_sgDvol.value) / 10;
                    if (_sgDvolVal) _sgDvolVal.textContent = '$' + raw.toFixed(1) + 'M';
                    invoke('update_settings', { field: 'scan_min_dollar_volume', value: String(raw * 1000000) });
                });
                // S15 Trading Edge live handlers
                const _s15ds = $el('s15-decay-start');
                const _s15dsVal = $el('s15-decay-start-val');
                if (_s15ds) _s15ds.addEventListener('input', () => {
                    if (_s15dsVal) _s15dsVal.textContent = _s15ds.value + '%';
                    invoke('update_settings', { field: 'max_hold_decay_start_pct', value: String(parseInt(_s15ds.value) / 100) });
                });
                const _s15dt = $el('s15-decay-tighten');
                const _s15dtVal = $el('s15-decay-tighten-val');
                if (_s15dt) _s15dt.addEventListener('input', () => {
                    if (_s15dtVal) _s15dtVal.textContent = (parseInt(_s15dt.value) / 1000).toFixed(3);
                    invoke('update_settings', { field: 'max_hold_decay_trail_tighten', value: String(parseInt(_s15dt.value) / 1000) });
                });
                const _s15mm = $el('s15-mtf-mode');
                if (_s15mm) _s15mm.addEventListener('change', () => {
                    invoke('update_settings', { field: 'mtf_gate_mode', value: _s15mm.value });
                });
                const _s15mp = $el('s15-mtf-penalty');
                const _s15mpVal = $el('s15-mtf-penalty-val');
                if (_s15mp) _s15mp.addEventListener('input', () => {
                    if (_s15mpVal) _s15mpVal.textContent = _s15mp.value;
                    invoke('update_settings', { field: 'mtf_gate_penalty', value: _s15mp.value });
                });
                const _s15hl = $el('s15-hhi-limit');
                const _s15hlVal = $el('s15-hhi-limit-val');
                if (_s15hl) _s15hl.addEventListener('input', () => {
                    if (_s15hlVal) _s15hlVal.textContent = (parseInt(_s15hl.value) / 100).toFixed(2);
                    invoke('update_settings', { field: 'max_concentration_hhi_manual', value: String(parseInt(_s15hl.value) / 100) });
                });

                // N4: Entry Threshold Matrix (helpers hoisted to module scope)
                const _etModeEl = $el('et-mode');
                if (_etModeEl) _etModeEl.addEventListener('change', async () => {
                    const snap = await invoke('get_settings_snapshot');
                    _loadEntryThresholds(snap);
                });

                function _collectEntryThresholdBatch(batch) {
                    const modeEl = $el('et-mode');
                    const mode = modeEl ? modeEl.value : 'scalp';
                    for (const regime of _etRegimes) {
                        for (const fld of _etFields) {
                            const el = $el(_etInputId(regime, fld));
                            if (el && el.value !== '') {
                                batch['et_' + mode + '_' + regime + '_' + fld] = el.value;
                            }
                        }
                    }
                }

                const btnValidate = $el('btn-validate-thresholds');
                if (btnValidate) btnValidate.addEventListener('click', async () => {
                    try {
                        const warnings = await invoke('validate_thresholds');
                        if (warnings.length === 0) {
                            if (typeof showToast === 'function') showToast('All thresholds valid', 'success');
                            else titanLogAdd('All thresholds valid');
                        } else {
                            warnings.forEach(w => {
                                if (typeof showToast === 'function') showToast(w, 'warning');
                                else titanLogAdd('WARNING: ' + w);
                            });
                        }
                    } catch (e) { titanLogAdd('Validation error: ' + normalizeError(e)); }
                });

                const btnResetThresh = $el('btn-reset-thresholds');
                if (btnResetThresh) btnResetThresh.addEventListener('click', async () => {
                    const defaults = {
                        scalp: { trending_aggro: {l:15,s:-15,c:0.12}, trending: {l:22,s:-22,c:0.18}, mean_rev_aggro: {l:25,s:-25,c:0.22}, mean_rev: {l:32,s:-32,c:0.28}, chaotic: {l:50,s:-50,c:0.38}, unknown_aggro: {l:22,s:-22,c:0.18}, unknown: {l:32,s:-32,c:0.25} },
                        normal: { trending_aggro: {l:25,s:-25,c:0.20}, trending: {l:35,s:-35,c:0.25}, mean_rev_aggro: {l:40,s:-40,c:0.30}, mean_rev: {l:50,s:-50,c:0.35}, chaotic: {l:70,s:-70,c:0.55}, unknown_aggro: {l:35,s:-35,c:0.25}, unknown: {l:50,s:-50,c:0.30} }
                    };
                    const mode = _etModeEl ? _etModeEl.value : 'scalp';
                    const batch = {};
                    for (const [regime, vals] of Object.entries(defaults[mode])) {
                        batch['et_' + mode + '_' + regime + '_long'] = String(vals.l);
                        batch['et_' + mode + '_' + regime + '_short'] = String(vals.s);
                        batch['et_' + mode + '_' + regime + '_conf'] = String(vals.c);
                    }
                    await invoke('batch_update_settings', { settings: batch });
                    const snap = await invoke('get_settings_snapshot');
                    _loadEntryThresholds(snap);
                    if (typeof showToast === 'function') showToast('Thresholds reset to defaults', 'success');
                    else titanLogAdd('Thresholds reset to defaults');
                });

                const btnDiscord = $el('btn-discord-test');
                if (btnDiscord) btnDiscord.addEventListener('click', async () => {
                    const url = $el('hft-discord-url')?.value || '';
                    if (!url) { titanLogAdd('Set webhook URL first'); return; }
                    await invoke('update_settings', { field: 'discord_webhook_url', value: url });
                    try {
                        const r = await invoke('test_discord_alert');
                        titanLogAdd(r);
                    } catch (e) { titanLogAdd('Discord test failed: ' + normalizeError(e)); }
                });

                const btnTgTest = $el('btn-tg-test');
                if (btnTgTest) btnTgTest.addEventListener('click', async () => {
                    const token = $el('hft-tg-token')?.value || '';
                    const chatId = $el('hft-tg-chatid')?.value || '';
                    if (!token || !chatId) { titanLogAdd('Set Telegram Bot Token and Chat ID first'); return; }
                    await invoke('batch_update_settings', { settings: {
                        telegram_bot_token: token,
                        telegram_chat_id: chatId,
                        telegram_enabled: 'true'
                    }});
                    try {
                        const r = await invoke('test_telegram_alert');
                        titanLogAdd(r);
                    } catch (e) { titanLogAdd('Telegram test failed: ' + normalizeError(e)); }
                });

                // ATR slider wiring
                const atrSliders = [
                    { id: 'hft-atr-stop', valId: 'hft-atr-stop-val', fmt: v => (v / 10).toFixed(1) + 'x' },
                    { id: 'hft-atr-take', valId: 'hft-atr-take-val', fmt: v => (v / 10).toFixed(1) + 'x' },
                    { id: 'hft-partial-pct', valId: 'hft-partial-pct-val', fmt: v => v + '%' },
                ];
                atrSliders.forEach(s => {
                    const slider = $el(s.id);
                    const valEl = $el(s.valId);
                    if (slider && valEl) {
                        slider.addEventListener('input', () => { valEl.textContent = s.fmt(parseFloat(slider.value)); });
                    }
                });

                // Options hedging slider wiring
                const hedgeSliders = [
                    { id: 'hft-hedge-otm', valId: 'hft-hedge-otm-val', fmt: v => v + '%' },
                    { id: 'hft-hedge-premium', valId: 'hft-hedge-premium-val', fmt: v => (v / 10).toFixed(1) + '%' },
                ];
                hedgeSliders.forEach(s => {
                    const slider = $el(s.id);
                    const valEl = $el(s.valId);
                    if (slider && valEl) {
                        slider.addEventListener('input', () => { valEl.textContent = s.fmt(parseFloat(slider.value)); });
                    }
                });

                const btnHedgeAll = $el('hft-hedge-all');
                if (btnHedgeAll) btnHedgeAll.addEventListener('click', async () => {
                    titanLogAdd('Hedging all positions...');
                    try {
                        const status = await invoke('get_hedge_status');
                        titanLogAdd('Hedge status: ' + JSON.stringify(status.length) + ' active hedges');
                    } catch (e) { titanLogAdd('Hedge error: ' + normalizeError(e)); }
                });

                // Web Dashboard URL copy
                const btnWebCopy = $el('hft-web-copy');
                if (btnWebCopy) btnWebCopy.addEventListener('click', () => {
                    const port = $el('hft-web-port')?.value || '7777';
                    const url = 'http://localhost:' + port;
                    navigator.clipboard.writeText(url).then(() => titanLogAdd('URL copied: ' + url));
                });

                // Replay tab wiring
                wireReplayTab();

                const btnReset = $el('btn-hft-reset');
                if (btnReset) btnReset.addEventListener('click', () => {
                    const defaults = {
                        'hft-signal-min': 40, 'hft-ofi-thresh': 5, 'hft-max-spread': 50,
                        'hft-cooldown': 30, 'hft-trail': 15, 'hft-tp': 25,
                        'hft-max-hold': 3600, 'hft-risk': 20, 'hft-max-pos': 1,
                        'hft-max-daily': 50, 'hft-daily-loss': 30,
                    };
                    Object.entries(defaults).forEach(([id, val]) => {
                        const el = $el(id);
                        if (el) { el.value = val; el.dispatchEvent(new Event('input')); }
                    });
                });

                // ═══ TOGGLE SWITCHES ═══
                document.querySelectorAll('#tab-4 .hft-toggle-switch').forEach(sw => {
                    sw.addEventListener('click', () => { sw.classList.toggle('on'); });
                });

                // ═══ DISABLE ADAPTIVE EXITS ═══
                const adaptiveToggle = $el('hft-disable-adaptive');
                if (adaptiveToggle) {
                    adaptiveToggle.addEventListener('click', () => {
                        invoke('update_settings', { field: 'disable_adaptive_exits', value: String(adaptiveToggle.classList.contains('on')) });
                    });
                }

                // ═══ MULTI-ASSET TOGGLE ═══
                const multiToggle = $el('hft-multi-asset-toggle');
                const multiSlider = $el('hft-max-concurrent');
                const multiVal = $el('hft-max-concurrent-val');
                if (multiSlider && multiVal) {
                    multiSlider.addEventListener('input', () => { multiVal.textContent = multiSlider.value; });
                }
                const chkMulti = $el('chk-multi-symbol-central');
                if (multiToggle && chkMulti) {
                    multiToggle.addEventListener('click', () => {
                        chkMulti.checked = multiToggle.classList.contains('on');
                        chkMulti.dispatchEvent(new Event('change'));
                    });
                }

                // ═══ TOAST HELPER (delegates to TitanToasts) ═══
                function hftToast(msg, type) {
                    const map = { '#00FF88': 'success', '#FF8800': 'warning', '#FF4444': 'error', '#FF0000': 'error', '#FF6600': 'warning' };
                    showCyberToast(msg, map[type] || type || 'info');
                }

                // ═══ UNDO (disk-persisted snapshot) ═══
                let _undoSnapshot = null;
                async function captureSnapshotToDisk() {
                    try { await invoke('save_undo_snapshot'); _undoSnapshot = true; }
                    catch(e) { console.warn('undo snapshot save failed', e); }
                }

                // ═══ SAVE CONFIG with toast + undo ═══
                const origSaveHandler = btnSave?.onclick;
                if (btnSave) {
                    const existingListeners = btnSave.cloneNode(true);
                    const newSave = $el('btn-hft-save');
                    if (newSave) newSave.addEventListener('click', () => {
                        hftToast('Config saved', 'success');
                    });
                }

                const btnUndo = $el('btn-hft-undo');
                if (btnUndo) btnUndo.addEventListener('click', async () => {
                    try {
                        const snap = await invoke('load_undo_snapshot');
                        if (!snap) { hftToast('No undo available', '#FF8800'); return; }
                        await invoke('apply_full_settings', { payload: snap });
                        await loadHftSettings();
                        hftToast('Undo applied - settings restored', '#00FF88');
                    } catch(e) {
                        hftToast('Undo failed: ' + (e.message || e), '#FF4444');
                    }
                });

                hftSliders.concat(hftAdvancedSliders).forEach(s => {
                    const el = $el(s.id);
                    if (el) el.addEventListener('change', () => {
                        if (!_undoSnapshot) captureSnapshotToDisk();
                    });
                });

                // ═══ SAFE DEFAULT PRESET ═══
                const SAFE_PRESET = {
                    // Entry
                    'hft-signal-min': 60, 'hft-ofi-thresh': 8, 'hft-max-spread': 30, 'hft-cooldown': 60,
                    // Exit
                    'hft-trail': 20, 'hft-tp': 30, 'hft-max-hold': 7200,
                    // Position
                    'hft-risk': 10, 'hft-max-pos': 1, 'hft-max-daily': 20, 'hft-daily-loss': 20,
                    // Autopilot Core
                    'hft-min-hold-secs': 180, 'hft-nexus-consensus': 70, 'hft-loop-ms': 1500,
                    'hft-ofi-stale': 10000, 'hft-min-bars': 20, 'hft-ofi-entry': 50,
                    'hft-max-qty': 1001, 'hft-min-qty': 10, 'hft-equity-fb': 10000, 'hft-bypass-risk': 25,
                    // Exit Scaling
                    'hft-vol-min': 6, 'hft-vol-max': 25, 'hft-fb-sl': 25, 'hft-fb-tp': 25,
                    'hft-fb-trail': 15, 'hft-scalp-stop': 7, 'hft-scalp-take': 6, 'hft-scalp-trail': 5,
                    'hft-partial-frac': 5, 'hft-partial-minq': 10, 'hft-atr-period': 14,
                    'hft-atr-smin': 5, 'hft-atr-smax': 80, 'hft-atr-tmin': 10, 'hft-atr-tmax': 150,
                    'hft-atr-trmin': 3, 'hft-atr-trmax': 50,
                    // Oracle Exit
                    'hft-oexit-conf': 40, 'hft-oexit-score': 30, 'hft-oflip-score': 10, 'hft-oflip-conf': 20,
                    'hft-chaos-loss': -5, 'hft-chaos-conf': 25,
                    // Circuit Breaker
                    'hft-brk-caution': -30, 'hft-brk-halt': -50, 'hft-brk-panic': -80,
                    'hft-max-notional': 25, 'hft-min-stop-frac': 1,
                    // Strategy Scoring
                    'hft-base-score': 50, 'hft-vpin-scalp': 85, 'hft-vpin-normal': 75,
                    'hft-min-rvol': 5, 'hft-rvol-win': 20, 'hft-rvol-relax': 30, 'hft-rvol-relaxv': 50,
                    'hft-chaos-kill': 10, 'hft-chaos-mult': 55,
                    // Engine Stops
                    'hft-long-stop-n': -25, 'hft-long-stop-a': -80, 'hft-short-stop': -50,
                    'hft-long-trail-mp': 15, 'hft-short-trail-g': 15, 'hft-short-trail-mp': 10,
                    // Oracle Fusion
                    'hft-ok-w': 30, 'hft-oe-w': 25, 'hft-oh-w': 20, 'hft-ov-w': 15, 'hft-ow-w': 10,
                    'hft-oconf-floor': 20, 'hft-ent-chaos': 85, 'hft-vpin-toxic': 60,
                    // Indicator Periods
                    'hft-ema-fast': 9, 'hft-ema-mid': 20, 'hft-ema-slow': 50, 'hft-ema-trend': 200,
                    'hft-rsi-per': 14, 'hft-macd-f': 12, 'hft-macd-s': 26, 'hft-macd-sig': 9,
                    'hft-bb-per': 20, 'hft-bb-std': 20, 'hft-adx-per': 14, 'hft-stoch-per': 14,
                    // Adaptive Exit Multipliers
                    'hft-exit-floor': 30,
                    'hft-regime-chaotic-stop': 70, 'hft-regime-chaotic-take': 60, 'hft-regime-chaotic-trail': 50,
                    'hft-regime-range-stop': 85, 'hft-regime-range-take': 75, 'hft-regime-range-trail': 85,
                    'hft-regime-trend-stop': 105, 'hft-regime-trend-take': 130, 'hft-regime-trend-trail': 110,
                    'hft-sig-align-stop': 105, 'hft-sig-align-take': 115, 'hft-sig-align-trail': 105,
                    'hft-sig-oppose-stop': 75, 'hft-sig-oppose-take': 70,
                    'hft-tf-short-stop': 80, 'hft-tf-short-take': 80, 'hft-tf-short-trail': 80,
                    // ATR Dynamic Stops
                    'hft-atr-stop': 15, 'hft-atr-take': 30, 'hft-partial-pct': 50,
                    // Correlation
                    'hft-max-corr': 85,
                    // Options Hedging
                    'hft-hedge-otm': 5, 'hft-hedge-premium': 10,
                    // Multi-Asset
                    'hft-max-concurrent': 5,
                };
                const SCALP_PRESET = {
                    // Entry -- aggressive, low barrier
                    'hft-signal-min': 30, 'hft-ofi-thresh': 3, 'hft-max-spread': 80, 'hft-cooldown': 10,
                    // Exit -- tight, fast
                    'hft-trail': 8, 'hft-tp': 12, 'hft-max-hold': 600,
                    // Position -- higher risk, more positions
                    'hft-risk': 30, 'hft-max-pos': 3, 'hft-max-daily': 100, 'hft-daily-loss': 50,
                    // Autopilot Core
                    'hft-min-hold-secs': 30, 'hft-nexus-consensus': 50, 'hft-loop-ms': 800,
                    'hft-ofi-stale': 5000, 'hft-min-bars': 10, 'hft-ofi-entry': 30,
                    'hft-max-qty': 2000, 'hft-min-qty': 5, 'hft-equity-fb': 10000, 'hft-bypass-risk': 40,
                    // Exit Scaling
                    'hft-vol-min': 4, 'hft-vol-max': 30, 'hft-fb-sl': 15, 'hft-fb-tp': 15,
                    'hft-fb-trail': 10, 'hft-scalp-stop': 5, 'hft-scalp-take': 4, 'hft-scalp-trail': 4,
                    'hft-partial-frac': 3, 'hft-partial-minq': 5, 'hft-atr-period': 10,
                    'hft-atr-smin': 3, 'hft-atr-smax': 60, 'hft-atr-tmin': 5, 'hft-atr-tmax': 100,
                    'hft-atr-trmin': 2, 'hft-atr-trmax': 40,
                    // Oracle Exit
                    'hft-oexit-conf': 30, 'hft-oexit-score': 20, 'hft-oflip-score': 8, 'hft-oflip-conf': 15,
                    'hft-chaos-loss': -3, 'hft-chaos-conf': 20,
                    // Circuit Breaker
                    'hft-brk-caution': -20, 'hft-brk-halt': -40, 'hft-brk-panic': -60,
                    'hft-max-notional': 35, 'hft-min-stop-frac': 1,
                    // Strategy Scoring
                    'hft-base-score': 40, 'hft-vpin-scalp': 90, 'hft-vpin-normal': 80,
                    'hft-min-rvol': 3, 'hft-rvol-win': 15, 'hft-rvol-relax': 20, 'hft-rvol-relaxv': 40,
                    'hft-chaos-kill': 8, 'hft-chaos-mult': 45,
                    // Engine Stops
                    'hft-long-stop-n': -15, 'hft-long-stop-a': -50, 'hft-short-stop': -30,
                    'hft-long-trail-mp': 8, 'hft-short-trail-g': 10, 'hft-short-trail-mp': 6,
                    // Oracle Fusion
                    'hft-ok-w': 25, 'hft-oe-w': 20, 'hft-oh-w': 20, 'hft-ov-w': 20, 'hft-ow-w': 15,
                    'hft-oconf-floor': 15, 'hft-ent-chaos': 80, 'hft-vpin-toxic': 50,
                    // Indicator Periods -- faster
                    'hft-ema-fast': 5, 'hft-ema-mid': 12, 'hft-ema-slow': 30, 'hft-ema-trend': 100,
                    'hft-rsi-per': 10, 'hft-macd-f': 8, 'hft-macd-s': 18, 'hft-macd-sig': 6,
                    'hft-bb-per': 14, 'hft-bb-std': 15, 'hft-adx-per': 10, 'hft-stoch-per': 10,
                    // Adaptive Exit Multipliers
                    'hft-exit-floor': 15,
                    'hft-regime-chaotic-stop': 60, 'hft-regime-chaotic-take': 50, 'hft-regime-chaotic-trail': 45,
                    'hft-regime-range-stop': 90, 'hft-regime-range-take': 80, 'hft-regime-range-trail': 85,
                    'hft-regime-trend-stop': 115, 'hft-regime-trend-take': 160, 'hft-regime-trend-trail': 120,
                    'hft-sig-align-stop': 115, 'hft-sig-align-take': 140, 'hft-sig-align-trail': 115,
                    'hft-sig-oppose-stop': 80, 'hft-sig-oppose-take': 75,
                    'hft-tf-short-stop': 95, 'hft-tf-short-take': 95, 'hft-tf-short-trail': 95,
                    // ATR Dynamic Stops
                    'hft-atr-stop': 10, 'hft-atr-take': 20, 'hft-partial-pct': 40,
                    // Correlation
                    'hft-max-corr': 70,
                    // Options Hedging
                    'hft-hedge-otm': 3, 'hft-hedge-premium': 5,
                    // Multi-Asset
                    'hft-max-concurrent': 10,
                };

                async function applyPreset(preset, name) {
                    await captureSnapshotToDisk();
                    Object.entries(preset).forEach(([id, val]) => {
                        const el = $el(id);
                        if (el) { el.value = val; el.dispatchEvent(new Event('input')); }
                    });
                    hftToast(name + ' applied', 'success');
                }

                const btnSafe = $el('btn-hft-safe-default');
                if (btnSafe) btnSafe.addEventListener('click', () => applyPreset(SAFE_PRESET, 'SAFE DEFAULT'));
                const btnScalp = $el('btn-hft-scalp-default');
                if (btnScalp) btnScalp.addEventListener('click', () => applyPreset(SCALP_PRESET, 'SCALP DEFAULT'));

                // ═══ PRESET LOCK TOGGLES ═══
                const safeLock = $el('hft-safe-lock');
                const scalpLock = $el('hft-scalp-lock');
                if (safeLock) safeLock.addEventListener('click', () => {
                    if (safeLock.classList.contains('on')) {
                        if (scalpLock) scalpLock.classList.remove('on');
                        applyPreset(SAFE_PRESET, 'SAFE DEFAULT locked');
                        localStorage.setItem('titan_preset_lock', 'safe');
                    } else {
                        localStorage.removeItem('titan_preset_lock');
                    }
                });
                if (scalpLock) scalpLock.addEventListener('click', () => {
                    if (scalpLock.classList.contains('on')) {
                        if (safeLock) safeLock.classList.remove('on');
                        applyPreset(SCALP_PRESET, 'SCALP DEFAULT locked');
                        localStorage.setItem('titan_preset_lock', 'scalp');
                    } else {
                        localStorage.removeItem('titan_preset_lock');
                    }
                });

                // Restore lock on load
                const savedLock = localStorage.getItem('titan_preset_lock');
                if (savedLock === 'safe' && safeLock) {
                    safeLock.classList.add('on');
                } else if (savedLock === 'scalp' && scalpLock) {
                    scalpLock.classList.add('on');
                }

                // ═══ EXPORT / IMPORT CONFIG ═══
                const btnExport = $el('btn-hft-export');
                if (btnExport) btnExport.addEventListener('click', async () => {
                    try {
                        const snap = await invoke('get_settings_snapshot');
                        const blob = new Blob([JSON.stringify(snap, null, 2)], { type: 'application/json' });
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url; a.download = 'titan_config_' + Date.now() + '.json';
                        a.click(); URL.revokeObjectURL(url);
                        hftToast('Config exported', 'success');
                    } catch (e) { hftToast('Export failed', 'warning'); }
                });

                const btnImport = $el('btn-hft-import');
                if (btnImport) btnImport.addEventListener('click', () => {
                    const input = document.createElement('input');
                    input.type = 'file'; input.accept = '.json';
                    input.addEventListener('change', async () => {
                        const file = input.files[0];
                        if (!file) return;
                        const text = await file.text();
                        try {
                            const config = JSON.parse(text);
                            await invoke('apply_full_settings', { payload: config });
                            _hftSettingsLoaded = false;
                            loadHftSettings();
                            hftToast('Config imported', 'success');
                        } catch (e) { hftToast('Invalid config file', 'warning'); }
                    });
                    input.click();
                });

                // ═══ PARAMETER SEARCH ═══
                const searchInput = $el('hft-param-search');
                if (searchInput) searchInput.addEventListener('input', () => {
                    const query = searchInput.value.toLowerCase();
                    const paramEls = '#tab-4 .mb-8, #tab-4 .hft-grid-2 > div, #tab-4 .hft-grid-3 > div, #tab-4 .hft-grid-4 > div';
                    document.querySelectorAll(paramEls).forEach(el => {
                        const label = el.querySelector('label');
                        if (label) {
                            el.style.display = (!query || label.textContent.toLowerCase().includes(query)) ? '' : 'none';
                        }
                    });
                    document.querySelectorAll('#tab-4 .hft-section-card, #tab-4 details').forEach(sec => {
                        const items = sec.querySelectorAll('.mb-8, .hft-grid-2 > div, .hft-grid-3 > div, .hft-grid-4 > div');
                        if (!items.length) return;
                        const allHidden = [...items].every(i => i.style.display === 'none');
                        sec.style.display = allHidden ? 'none' : '';
                    });
                });

                // ═══ DRY RUN / LIVE BADGE ═══
                async function updateDryLiveBadge() {
                    const badge = $el('hft-drylive-badge');
                    if (!badge) return;
                    try {
                        const snap = await invoke('get_settings_snapshot');
                        const isDry = snap.dry_run_enabled !== false;
                        badge.textContent = isDry ? 'DRY RUN' : 'LIVE';
                        badge.className = 'drylive-badge ' + (isDry ? 'dry' : 'live');
                    } catch (e) { /* ignore */ }
                }
                updateDryLiveBadge();
                TitanBus.on('dryRunChanged', updateDryLiveBadge);
                TitanBus.on('tabSwitch', (idx) => { if (idx === 4) updateDryLiveBadge(); });

                // ═══ EMERGENCY KILL DASHBOARD ═══
                const killBtn = $el('btn-emergency-kill');
                if (killBtn) {
                    killBtn.addEventListener('click', async () => {
                        if (!(await cyberConfirm('EMERGENCY LIQUIDATION\n\nThis will:\n- Close ALL positions\n- Cancel ALL orders\n- Activate kill switch\n\nARE YOU SURE?'))) return;
                        killBtn.disabled = true;
                        killBtn.textContent = 'KILLING...';
                        try {
                            const result = await invoke('emergency_liquidate', { confirmToken: 'CONFIRM_EMERGENCY' });
                            hftToast(result || 'EMERGENCY LIQUIDATION COMPLETE', '#FF0000');
                        } catch (e) {
                            hftToast('KILL FAILED: ' + (e.message || e), '#FF0000');
                        } finally {
                            killBtn.disabled = false;
                            killBtn.textContent = 'KILL ALL';
                            refreshEmergencyPanel();
                        }
                    });
                }

                async function refreshEmergencyPanel() {
                    try {
                        const risk = await invoke('get_risk_status');
                        const breakerEl = $el('hft-em-breaker');
                        const equityEl = $el('hft-em-equity');
                        const pnlEl = $el('hft-em-dailypnl');
                        const posEl = $el('hft-em-positions');
                        const panel = $el('hft-emergency-panel');

                        if (breakerEl) {
                            const level = (risk.breaker_level || 'Normal').toUpperCase();
                            breakerEl.textContent = level;
                            breakerEl.className = 'hft-em-val breaker-' + level.toLowerCase();
                        }
                        if (equityEl && risk.session) {
                            const eq = parseFloat(risk.session.current_equity || '0');
                            equityEl.textContent = '$' + eq.toLocaleString('en-US', {minimumFractionDigits:2, maximumFractionDigits:2});
                        }
                        if (pnlEl && risk.session) {
                            const pnl = parseFloat(risk.session.daily_pnl || '0');
                            pnlEl.textContent = (pnl >= 0 ? '+' : '') + '$' + pnl.toLocaleString('en-US', {minimumFractionDigits:2, maximumFractionDigits:2});
                            pnlEl.style.color = pnl >= 0 ? '#00FF88' : '#FF4444';
                        }
                        if (posEl) {
                            try {
                                const snapshot = await invoke('get_monitor_snapshot');
                                posEl.textContent = (snapshot.open_positions || []).length;
                            } catch(_) { posEl.textContent = '-'; }
                        }
                        if (panel) {
                            const level = (risk.breaker_level || 'Normal').toUpperCase();
                            panel.classList.toggle('panic-active', level === 'PANIC');
                        }
                    } catch(e) { /* ignore */ }
                }
                refreshEmergencyPanel();
                TitanTimers.register('hft:emergency-panel', refreshEmergencyPanel, 3000, { tab: 3 });

                // ═══ SETTINGS DIFF VIEW ═══
                let _lastSavedSnapshot = null;
                async function captureSavedSnapshot() {
                    try { _lastSavedSnapshot = await invoke('get_settings_snapshot'); }
                    catch(e) { /* ignore */ }
                }
                captureSavedSnapshot();

                const btnShowChanges = $el('btn-hft-show-changes');
                const diffPanel = $el('hft-diff-panel');
                const diffBody = $el('hft-diff-body');
                const btnCloseDiff = $el('btn-hft-close-diff');

                if (btnShowChanges) btnShowChanges.addEventListener('click', () => {
                    if (!_lastSavedSnapshot) { hftToast('No saved snapshot to compare', '#FF8800'); return; }
                    const allSliders = hftSliders.concat(hftAdvancedSliders);
                    const changes = [];
                    for (const s of allSliders) {
                        const el = $el(s.id);
                        if (!el) continue;
                        const curVal = parseFloat(el.value);
                        const savedKey = s.field || s.id.replace('hft-', '').replace(/-/g, '_');
                        const savedVal = parseFloat(_lastSavedSnapshot[savedKey] ?? el.getAttribute('value') ?? curVal);
                        if (Math.abs(curVal - savedVal) > 0.001) {
                            changes.push({ label: s.label || savedKey, oldVal: savedVal, newVal: curVal });
                        }
                    }
                    if (!changes.length) {
                        diffBody.innerHTML = '<div class="hft-diff-no-changes">No changes detected</div>';
                    } else {
                        let html = '<table class="hft-diff-table"><thead><tr><th>PARAMETER</th><th>SAVED</th><th>CURRENT</th></tr></thead><tbody>';
                        for (const c of changes) {
                            html += `<tr><td class="diff-field">${c.label}</td><td class="diff-old">${c.oldVal}</td><td class="diff-new">${c.newVal}</td></tr>`;
                        }
                        html += '</tbody></table>';
                        diffBody.innerHTML = html;
                    }
                    diffPanel.style.display = '';
                });
                if (btnCloseDiff) btnCloseDiff.addEventListener('click', () => { diffPanel.style.display = 'none'; });

                // ═══ LIVE IMPACT PREVIEW ═══
                async function refreshImpactPreview() {
                    try {
                        const data = await invoke('preview_effective_exits');
                        const fmt = (v) => (v * 100 / 100).toFixed(2) + '%';
                        const el = (id) => $el(id);
                        if (el('hft-eff-stop')) el('hft-eff-stop').textContent = fmt(data.effective_stop_min);
                        if (el('hft-eff-take')) el('hft-eff-take').textContent = fmt(data.effective_take_min);
                        if (el('hft-eff-trail')) el('hft-eff-trail').textContent = fmt(data.effective_trail_min);
                        if (el('hft-eff-stop-base')) el('hft-eff-stop-base').textContent = fmt(data.base_stop);
                        if (el('hft-eff-take-base')) el('hft-eff-take-base').textContent = fmt(data.base_take);
                        if (el('hft-eff-trail-base')) el('hft-eff-trail-base').textContent = fmt(data.base_trail);
                    } catch(e) { /* ignore */ }
                }
                refreshImpactPreview();
                let _impactDebounce = null;
                hftSliders.concat(hftAdvancedSliders).forEach(s => {
                    const el = $el(s.id);
                    if (el) el.addEventListener('input', () => {
                        clearTimeout(_impactDebounce);
                        _impactDebounce = setTimeout(refreshImpactPreview, 500);
                    });
                });

                // ═══ REGIME HEATMAP ═══
                async function refreshRegimeHeatmap() {
                    try {
                        const data = await invoke('preview_effective_exits');
                        const rm = data.regime_mult || {};
                        const set = (id, v) => { const e = $el(id); if (e) e.textContent = (v||1).toFixed(2); };
                        set('hft-rh-trend-stop', rm.trending?.stop);
                        set('hft-rh-trend-take', rm.trending?.take);
                        set('hft-rh-trend-trail', rm.trending?.trail);
                        set('hft-rh-range-stop', rm.range?.stop);
                        set('hft-rh-range-take', rm.range?.take);
                        set('hft-rh-range-trail', rm.range?.trail);
                        set('hft-rh-chaotic-stop', rm.chaotic?.stop);
                        set('hft-rh-chaotic-take', rm.chaotic?.take);
                        set('hft-rh-chaotic-trail', rm.chaotic?.trail);

                        const risk = await invoke('get_risk_status');
                        const regimeRaw = (risk.breaker_level || '').toLowerCase();
                        document.querySelectorAll('.hft-regime-box').forEach(b => b.classList.remove('active'));
                        if (regimeRaw.includes('panic') || regimeRaw.includes('halt')) {
                            $el('hft-regime-chaotic')?.classList.add('active', 'regime-chaotic');
                        } else if (regimeRaw.includes('caution')) {
                            $el('hft-regime-range')?.classList.add('active', 'regime-range');
                        } else {
                            $el('hft-regime-trending')?.classList.add('active', 'regime-trending');
                        }
                    } catch(e) { /* ignore */ }
                }
                refreshRegimeHeatmap();
                TitanTimers.register('hft:regime-heatmap', refreshRegimeHeatmap, 5000, { tab: 3 });

                // ═══ PARAMETER PROFILES (RF-C: disk persistence) ═══
                const PROFILES_LEGACY_KEY = 'titan_hft_profiles';

                async function refreshProfileSelect() {
                    const sel = $el('hft-profile-select');
                    if (!sel) return;
                    try {
                        const metas = await invoke('list_profiles');
                        sel.innerHTML = '<option value="">-- Select Profile --</option>';
                        metas.forEach(m => {
                            const opt = document.createElement('option');
                            opt.value = m.name; opt.textContent = m.name;
                            sel.appendChild(opt);
                        });
                    } catch(e) {
                        sel.innerHTML = '<option value="">-- Select Profile --</option>';
                    }
                }

                // One-shot migration from localStorage to disk
                (async function migrateLocalStorageProfiles() {
                    try {
                        const raw = localStorage.getItem(PROFILES_LEGACY_KEY);
                        if (!raw) return;
                        const oldProfiles = JSON.parse(raw);
                        const names = Object.keys(oldProfiles);
                        if (!names.length) return;
                        for (const name of names) {
                            const entry = {
                                schema_v: 1,
                                saved_at: new Date().toISOString(),
                                data: oldProfiles[name]
                            };
                            await invoke('import_profile', { name, data: entry });
                        }
                        localStorage.removeItem(PROFILES_LEGACY_KEY);
                        console.log('[profiles] Migrated ' + names.length + ' profiles from localStorage to disk');
                    } catch(e) {
                        console.warn('[profiles] localStorage migration failed:', e);
                    }
                })();

                refreshProfileSelect();

                const btnSaveProfile = $el('btn-hft-save-profile');
                if (btnSaveProfile) btnSaveProfile.addEventListener('click', async () => {
                    const nameInput = $el('hft-profile-name');
                    const name = (nameInput?.value || '').trim();
                    if (!name) { hftToast('Enter a profile name', '#FF8800'); return; }
                    try {
                        await invoke('save_profile', { name });
                        await refreshProfileSelect();
                        hftToast('Profile "' + name + '" saved', '#00FF88');
                        if (nameInput) nameInput.value = '';
                    } catch(e) { hftToast('Save failed: ' + normalizeError(e), '#FF4444'); }
                });

                const btnLoadProfile = $el('btn-hft-load-profile');
                if (btnLoadProfile) btnLoadProfile.addEventListener('click', async () => {
                    const sel = $el('hft-profile-select');
                    const name = sel?.value;
                    if (!name) { hftToast('Select a profile', '#FF8800'); return; }
                    await captureSnapshotToDisk();
                    try {
                        await invoke('load_profile', { name });
                        _hftSettingsLoaded = false;
                        await loadHftSettings();
                        hftToast('Profile "' + name + '" loaded', '#00FF88');
                    } catch(e) { hftToast('Load failed: ' + normalizeError(e), '#FF4444'); }
                });

                const btnDeleteProfile = $el('btn-hft-delete-profile');
                if (btnDeleteProfile) btnDeleteProfile.addEventListener('click', async () => {
                    const sel = $el('hft-profile-select');
                    const name = sel?.value;
                    if (!name) { hftToast('Select a profile', '#FF8800'); return; }
                    if (!(await cyberConfirm('Delete profile "' + name + '"?'))) return;
                    try {
                        await invoke('delete_profile', { name });
                        await refreshProfileSelect();
                        hftToast('Profile "' + name + '" deleted', '#FF4444');
                    } catch(e) { hftToast('Delete failed: ' + normalizeError(e), '#FF4444'); }
                });

                const btnExportProfile = $el('btn-hft-export-profile');
                if (btnExportProfile) btnExportProfile.addEventListener('click', async () => {
                    const sel = $el('hft-profile-select');
                    const name = sel?.value;
                    if (!name) { hftToast('Select a profile to export', '#FF8800'); return; }
                    try {
                        const data = await invoke('export_profile', { name });
                        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url; a.download = 'titan_profile_' + name.replace(/[^a-zA-Z0-9_-]/g, '_') + '.json';
                        a.click(); URL.revokeObjectURL(url);
                        hftToast('Profile "' + name + '" exported', '#00FF88');
                    } catch(e) { hftToast('Export failed: ' + normalizeError(e), '#FF4444'); }
                });

                const btnImportProfile = $el('btn-hft-import-profile');
                if (btnImportProfile) btnImportProfile.addEventListener('click', () => {
                    const input = document.createElement('input');
                    input.type = 'file'; input.accept = '.json';
                    input.addEventListener('change', async () => {
                        const file = input.files[0];
                        if (!file) return;
                        const text = await file.text();
                        try {
                            const data = JSON.parse(text);
                            let name = file.name.replace(/\.json$/i, '').replace(/^titan_profile_/, '');
                            const nameInput = $el('hft-profile-name');
                            if (nameInput?.value?.trim()) name = nameInput.value.trim();
                            await invoke('import_profile', { name, data });
                            await refreshProfileSelect();
                            hftToast('Profile "' + name + '" imported', '#00FF88');
                        } catch(e) { hftToast('Import failed: ' + normalizeError(e), '#FF4444'); }
                    });
                    input.click();
                });

                // ═══ PER-SYMBOL OVERRIDES ═══
                async function refreshOverridesList() {
                    const listEl = $el('hft-overrides-list');
                    if (!listEl) return;
                    try {
                        const overrides = await invoke('get_symbol_overrides');
                        const entries = Object.entries(overrides || {});
                        if (!entries.length) { listEl.innerHTML = '<span style="color:#666;">No per-symbol overrides set</span>'; return; }
                        listEl.innerHTML = entries.map(([sym, ovr]) => {
                            const parts = [];
                            if (ovr.trailing_stop_pct != null) parts.push('Trail:' + ovr.trailing_stop_pct + '%');
                            if (ovr.take_profit_pct != null) parts.push('Take:' + ovr.take_profit_pct + '%');
                            if (ovr.risk_per_trade != null) parts.push('Risk:' + ovr.risk_per_trade + '%');
                            return '<span style="color:#00BFFF; font-weight:700;">' + sym + '</span>: ' + (parts.join(', ') || 'empty');
                        }).join(' | ');
                    } catch(e) { listEl.innerHTML = '<span style="color:#FF4444;">Failed to load</span>'; }
                }
                refreshOverridesList();

                const btnSetOverride = $el('btn-hft-set-override');
                if (btnSetOverride) btnSetOverride.addEventListener('click', async () => {
                    const sym = ($el('hft-override-symbol')?.value || '').trim().toUpperCase();
                    if (!sym) { hftToast('Enter a symbol', '#FF8800'); return; }
                    const trail = $el('hft-ovr-trail')?.value;
                    const take = $el('hft-ovr-take')?.value;
                    const risk = $el('hft-ovr-risk')?.value;
                    const overrides = {};
                    if (trail) overrides.trailing_stop_pct = parseFloat(trail);
                    if (take) overrides.take_profit_pct = parseFloat(take);
                    if (risk) overrides.risk_per_trade = parseFloat(risk);
                    try {
                        await invoke('set_symbol_override', { symbol: sym, overrides });
                        hftToast('Override set for ' + sym, '#00FF88');
                        refreshOverridesList();
                    } catch(e) { hftToast('Failed: ' + normalizeError(e), '#FF4444'); }
                });

                const btnRmOverride = $el('btn-hft-rm-override');
                if (btnRmOverride) btnRmOverride.addEventListener('click', async () => {
                    const sym = ($el('hft-override-symbol')?.value || '').trim().toUpperCase();
                    if (!sym) { hftToast('Enter a symbol', '#FF8800'); return; }
                    try {
                        await invoke('remove_symbol_override', { symbol: sym });
                        hftToast('Override removed for ' + sym, '#FF4444');
                        refreshOverridesList();
                    } catch(e) { hftToast('Failed: ' + normalizeError(e), '#FF4444'); }
                });

                // ═══ RISK CALCULATOR LIVE ═══
                async function refreshRiskCalculator() {
                    try {
                        const risk = await invoke('get_risk_status');
                        const snap = await invoke('get_settings_snapshot');
                        const equity = parseFloat(risk.session?.current_equity || '0');
                        const riskPct = snap.risk_per_trade || 2;
                        const stopPct = snap.trailing_stop_pct || 15;
                        const takePct = snap.take_profit_pct || 25;
                        const dailyLossPct = snap.max_daily_loss_pct || 5;

                        const maxLoss = equity * (riskPct / 100);
                        const posSize = stopPct > 0 ? maxLoss / (stopPct / 100) : 0;
                        const rr = stopPct > 0 ? (takePct / stopPct).toFixed(1) : '0';
                        const dailyMax = equity * (dailyLossPct / 100);

                        const el = (id) => $el(id);
                        const fmt = (v) => '$' + v.toLocaleString('en-US', {minimumFractionDigits:2, maximumFractionDigits:2});
                        if (el('hft-rc-max-loss')) el('hft-rc-max-loss').textContent = fmt(maxLoss);
                        if (el('hft-rc-pos-size')) el('hft-rc-pos-size').textContent = fmt(posSize);
                        if (el('hft-rc-rr')) el('hft-rc-rr').textContent = '1:' + rr;
                        if (el('hft-rc-daily-max')) el('hft-rc-daily-max').textContent = fmt(dailyMax);
                    } catch(e) { /* ignore */ }
                }
                refreshRiskCalculator();
                let _riskDebounce = null;
                hftSliders.concat(hftAdvancedSliders).forEach(s => {
                    const el = $el(s.id);
                    if (el) el.addEventListener('input', () => {
                        clearTimeout(_riskDebounce);
                        _riskDebounce = setTimeout(refreshRiskCalculator, 500);
                    });
                });

                // ═══ PARAMETER HEALTH ANALYSIS ═══
                function analyzeParamHealth() {
                    const warnings = [];
                    const v = (id) => parseFloat($el(id)?.value || '0');

                    const trail = v('hft-trail');
                    const take = v('hft-tp');
                    const risk = v('hft-risk');
                    const maxPos = v('hft-max-pos');
                    const maxDaily = v('hft-max-daily');
                    const cooldown = v('hft-cooldown');
                    const dailyLoss = v('hft-daily-loss');
                    const signalMin = v('hft-signal-min');

                    if (trail > 0 && take > 0 && trail >= take)
                        warnings.push({level:'danger', msg:'Trailing stop >= Take profit (will exit before TP)'});
                    if (risk > 5)
                        warnings.push({level:'warn', msg:'Risk per trade > 5% (aggressive)'});
                    if (risk > 10)
                        warnings.push({level:'danger', msg:'Risk per trade > 10% (extremely aggressive)'});
                    if (maxPos > 10)
                        warnings.push({level:'warn', msg:'Max positions > 10 (high exposure)'});
                    if (maxDaily > 50)
                        warnings.push({level:'warn', msg:'Max daily trades > 50 (overtrading risk)'});
                    if (cooldown < 10)
                        warnings.push({level:'warn', msg:'Cooldown < 10s (rapid fire entries)'});
                    if (dailyLoss > 10)
                        warnings.push({level:'danger', msg:'Daily loss limit > 10% (capital at risk)'});
                    if (signalMin < 30)
                        warnings.push({level:'warn', msg:'Signal minimum < 30 (low quality entries)'});

                    const chaosStop = v('hft-regime-chaotic-stop');
                    const chaosTrail = v('hft-regime-chaotic-trail');
                    if (chaosStop < 40)
                        warnings.push({level:'warn', msg:'Chaotic stop mult < 0.40x (very tight in chaos)'});
                    if (chaosTrail < 40)
                        warnings.push({level:'warn', msg:'Chaotic trail mult < 0.40x (premature exits in chaos)'});

                    const trendTake = v('hft-regime-trend-take');
                    if (trendTake > 180)
                        warnings.push({level:'warn', msg:'Trend take mult > 1.80x (may miss TP in trends)'});

                    if (take < 5)
                        warnings.push({level:'warn', msg:'Take profit < 5% (scalping micro gains)'});
                    if (trail < 3)
                        warnings.push({level:'warn', msg:'Trailing stop < 3% (noise will trigger exits)'});

                    if (risk * maxPos > 30)
                        warnings.push({level:'danger', msg:'Risk x MaxPositions = ' + (risk * maxPos).toFixed(0) + '% (total portfolio risk too high)'});

                    const healthEl = $el('hft-param-health');
                    if (!healthEl) return;

                    if (!warnings.length) {
                        healthEl.innerHTML = '<span class="hft-health-badge ok">All parameters healthy</span>';
                        return;
                    }
                    healthEl.innerHTML = warnings.map(w =>
                        '<span class="hft-health-badge ' + w.level + '">' + w.msg + '</span>'
                    ).join('');
                }
                analyzeParamHealth();
                hftSliders.concat(hftAdvancedSliders).forEach(s => {
                    const el = $el(s.id);
                    if (el) el.addEventListener('input', () => setTimeout(analyzeParamHealth, 200));
                });

                // ═══ AUTO-SUGGEST ═══
                const btnAutoSuggest = $el('btn-hft-auto-suggest');
                const btnCloseSuggest = $el('btn-hft-close-suggest');
                if (btnCloseSuggest) btnCloseSuggest.addEventListener('click', () => {
                    const resultsEl = $el('hft-auto-suggest-results');
                    if (resultsEl) { resultsEl.style.display = 'none'; }
                    btnCloseSuggest.style.display = 'none';
                });
                if (btnAutoSuggest) btnAutoSuggest.addEventListener('click', async () => {
                    const resultsEl = $el('hft-auto-suggest-results');
                    if (!resultsEl) return;
                    resultsEl.style.display = '';
                    if (btnCloseSuggest) btnCloseSuggest.style.display = '';
                    resultsEl.innerHTML = '<span style="color:#888;">Analyzing trades...</span>';

                    try {
                        const trades = await invoke('get_journal_trades', {
                            symbol: null, side: null, dateFrom: null, dateTo: null, minPnl: null
                        });
                        const recent = trades.slice(-50);
                        if (recent.length < 5) {
                            resultsEl.innerHTML = '<span style="color:#FF8800;">Need at least 5 trades for analysis</span>';
                            return;
                        }

                        const suggestions = [];
                        const snap = await invoke('get_settings_snapshot');

                        const slHits = recent.filter(t => (t.reason || '').toUpperCase().includes('STOP'));
                        const trailHits = recent.filter(t => (t.reason || '').toUpperCase().includes('TRAIL'));
                        const tpHits = recent.filter(t => (t.reason || '').toUpperCase().includes('TAKE') || (t.reason || '').toUpperCase().includes('TP'));
                        const wins = recent.filter(t => t.pnl > 0);
                        const losses = recent.filter(t => t.pnl < 0);
                        const avgWinPct = wins.length ? wins.reduce((s,t) => s + t.pnl_pct, 0) / wins.length : 0;
                        const avgLossPct = losses.length ? Math.abs(losses.reduce((s,t) => s + t.pnl_pct, 0) / losses.length) : 0;
                        const avgDuration = recent.reduce((s,t) => s + (t.duration_secs || 0), 0) / recent.length;

                        if (slHits.length > recent.length * 0.5) {
                            const newTrail = Math.min((snap.trailing_stop_pct || 15) * 1.3, 50);
                            suggestions.push({
                                msg: `${((slHits.length/recent.length)*100).toFixed(0)}% of exits are stop losses. Consider widening trailing stop to ${newTrail.toFixed(1)}%.`,
                                apply: {'trailing_stop_pct': newTrail.toFixed(1)}
                            });
                        }

                        if (trailHits.length > recent.length * 0.6 && avgWinPct < 1) {
                            const newTrail = Math.min((snap.trailing_stop_pct || 15) * 1.2, 40);
                            suggestions.push({
                                msg: `Trailing stop exits too frequent (${trailHits.length}/${recent.length}) with small avg win ${avgWinPct.toFixed(2)}%. Widen trail to ${newTrail.toFixed(1)}%.`,
                                apply: {'trailing_stop_pct': newTrail.toFixed(1)}
                            });
                        }

                        if (tpHits.length < recent.length * 0.1 && wins.length > 0) {
                            const newTp = Math.max((snap.take_profit_pct || 25) * 0.8, 5);
                            suggestions.push({
                                msg: `Very few TP hits (${tpHits.length}/${recent.length}). Consider lowering take profit to ${newTp.toFixed(1)}% to lock gains.`,
                                apply: {'take_profit_pct': newTp.toFixed(1)}
                            });
                        }

                        if (avgDuration < 30 && recent.length > 10) {
                            suggestions.push({
                                msg: `Average hold time very short (${avgDuration.toFixed(0)}s). Consider increasing min_hold_secs to reduce noise exits.`,
                                apply: {'min_hold_secs': Math.max((snap.min_hold_secs || 5) + 5, 10).toString()}
                            });
                        }

                        if (avgDuration > 7200 && recent.length > 5) {
                            const newMax = Math.max(avgDuration * 0.7, 1800);
                            suggestions.push({
                                msg: `Average hold time long (${(avgDuration/60).toFixed(0)}min). Consider reducing max_hold to ${(newMax/60).toFixed(0)}min.`,
                                apply: {'max_hold_secs': newMax.toFixed(0)}
                            });
                        }

                        if (losses.length > 0 && avgLossPct > avgWinPct * 2) {
                            suggestions.push({
                                msg: `Average loss (${avgLossPct.toFixed(2)}%) is ${(avgLossPct/avgWinPct).toFixed(1)}x average win. Tighten risk per trade.`,
                                apply: {'risk_per_trade': Math.max((snap.risk_per_trade || 2) * 0.7, 0.5).toFixed(1)}
                            });
                        }

                        const winRate = wins.length / recent.length;
                        if (winRate < 0.35 && recent.length >= 10) {
                            const newSignal = Math.min((snap.entry_signal_strength_min || 40) + 10, 90);
                            suggestions.push({
                                msg: `Win rate low (${(winRate*100).toFixed(0)}%). Raise signal minimum to ${newSignal} for higher quality entries.`,
                                apply: {'entry_signal_strength_min': newSignal.toString()}
                            });
                        }

                        if (!suggestions.length) {
                            resultsEl.innerHTML = '<span class="hft-health-badge ok">No issues detected. Current parameters look well-tuned for recent trades.</span>';
                            return;
                        }

                        resultsEl.innerHTML = suggestions.map((s, i) =>
                            `<div class="hft-suggest-card">
                                <span class="hft-suggest-msg">${s.msg}</span>
                                <button class="hft-suggest-apply" data-suggest-idx="${i}">APPLY</button>
                            </div>`
                        ).join('');

                        resultsEl.querySelectorAll('.hft-suggest-apply').forEach(btn => {
                            btn.addEventListener('click', async () => {
                                const idx = parseInt(btn.dataset.suggestIdx);
                                const sug = suggestions[idx];
                                if (!sug?.apply) return;
                                await captureSnapshotToDisk();
                                try {
                                    await invoke('batch_update_settings', { settings: sug.apply });
                                    await loadHftSettings();
                                    btn.textContent = 'APPLIED';
                                    btn.disabled = true;
                                    btn.style.borderColor = '#00FF88';
                                    btn.style.color = '#00FF88';
                                    hftToast('Suggestion applied', '#00FF88');
                                } catch(e) { hftToast('Apply failed: ' + normalizeError(e), '#FF4444'); }
                            });
                        });
                    } catch(e) {
                        resultsEl.innerHTML = '<span style="color:#FF4444;">Analysis failed: ' + (e.message || e) + '</span>';
                    }
                });
            }

            const _etRegimes = ['trending', 'trending_aggro', 'mean_rev', 'mean_rev_aggro', 'chaotic', 'unknown', 'unknown_aggro'];
            const _etFields = ['long', 'short', 'conf'];
            function _etInputId(regime, fld) {
                return 'et-' + regime.replace(/_/g, '-') + '-' + fld;
            }
            function _loadEntryThresholds(snap) {
                const modeEl = $el('et-mode');
                const mode = modeEl ? modeEl.value : 'scalp';
                const map = mode === 'scalp' ? snap.entry_thresholds_scalp : snap.entry_thresholds_normal;
                if (!map) return;
                for (const regime of _etRegimes) {
                    const t = map[regime];
                    if (!t) continue;
                    const longEl = $el(_etInputId(regime, 'long'));
                    const shortEl = $el(_etInputId(regime, 'short'));
                    const confEl = $el(_etInputId(regime, 'conf'));
                    if (longEl) longEl.value = t.long_threshold;
                    if (shortEl) shortEl.value = t.short_threshold;
                    if (confEl) confEl.value = t.conf_threshold;
                }
            }

            function _restoreTaGates(snap) {
                const gates = snap.ta_gates || {};
                const gateMap = {
                    'ta-gate-rsi': gates.rsi_gate || 'Off',
                    'ta-gate-ema': gates.ema_alignment_gate || 'Off',
                    'ta-gate-adx': gates.adx_gate || 'Off',
                    'ta-gate-bb': gates.bb_squeeze_gate || 'Off',
                    'ta-gate-macd': gates.macd_gate || 'Off',
                    'ta-gate-stoch': gates.stoch_rsi_gate || 'Off',
                };
                Object.entries(gateMap).forEach(([id, mode]) => {
                    const el = $el(id);
                    if (el) el.value = mode.toLowerCase();
                });
                const penEl = $el('ta-gate-penalty');
                const penVal = $el('ta-gate-penalty-val');
                if (penEl) { penEl.value = snap.ta_gate_soft_penalty || 15; if (penVal) penVal.textContent = penEl.value; }
            }

            async function loadHftSettings() {
                try {
                    const snap = await invoke('get_settings_snapshot');
                    const map = {
                        'hft-signal-min': snap.entry_signal_strength_min || 40,
                        'hft-ofi-thresh': snap.min_ofi_threshold || 5,
                        'hft-max-spread': snap.max_spread_bps || 50,
                        'hft-cooldown': snap.entry_cooldown_secs || 30,
                        'hft-trail': (snap.trailing_stop_pct || 1.5) * 10,
                        'hft-tp': (snap.take_profit_pct || 2.5) * 10,
                        'hft-max-hold': snap.max_hold_secs || 3600,
                        'hft-risk': (snap.risk_per_trade || 2.0) * 10,
                        'hft-max-pos': snap.max_open_positions || 1,
                        'hft-max-daily': snap.max_daily_trades || 50,
                        'hft-daily-loss': (snap.daily_loss_limit_pct || 3.0) * 10,
                    };
                    Object.entries(map).forEach(([id, val]) => {
                        const el = $el(id);
                        if (el) { el.value = val; el.dispatchEvent(new Event('input')); }
                    });
                    const corrEl = $el('hft-max-corr');
                    const corrValEl = $el('hft-max-corr-val');
                    if (corrEl) {
                        corrEl.value = Math.round((snap.max_correlation || 0.70) * 100);
                        if (corrValEl) corrValEl.textContent = (snap.max_correlation || 0.70).toFixed(2);
                    }
                    const urlEl = $el('hft-discord-url');
                    if (urlEl) urlEl.value = snap.discord_webhook_url || '';
                    const fillEl = $el('hft-alert-fill');
                    if (fillEl) fillEl.checked = snap.alert_on_fill !== false;
                    const breakerEl = $el('hft-alert-breaker');
                    if (breakerEl) breakerEl.checked = snap.alert_on_breaker !== false;
                    const dailyEl = $el('hft-alert-daily');
                    if (dailyEl) dailyEl.checked = snap.alert_on_daily_summary === true;
                    // Telegram
                    const tgTokenEl = $el('hft-tg-token');
                    if (tgTokenEl) tgTokenEl.value = snap.telegram_bot_token || '';
                    const tgChatEl = $el('hft-tg-chatid');
                    if (tgChatEl) tgChatEl.value = snap.telegram_chat_id || '';
                    const tgEnabledEl = $el('hft-tg-enabled');
                    if (tgEnabledEl) tgEnabledEl.checked = snap.telegram_enabled === true;
                    // ATR
                    const atrStopEl = $el('hft-atr-stop');
                    if (atrStopEl) { atrStopEl.value = Math.round((snap.atr_stop_multiplier || 1.5) * 10); atrStopEl.dispatchEvent(new Event('input')); }
                    const atrTakeEl = $el('hft-atr-take');
                    if (atrTakeEl) { atrTakeEl.value = Math.round((snap.atr_take_multiplier || 3.0) * 10); atrTakeEl.dispatchEvent(new Event('input')); }
                    const partialPctEl = $el('hft-partial-pct');
                    if (partialPctEl) { partialPctEl.value = snap.partial_take_pct || 50; partialPctEl.dispatchEvent(new Event('input')); }
                    const partialEnEl = $el('hft-partial-enabled');
                    if (partialEnEl) partialEnEl.checked = snap.partial_take_enabled !== false;
                    // LLM Sentiment
                    const llmKeyEl = $el('hft-llm-key');
                    if (llmKeyEl) llmKeyEl.value = snap.llm_api_key || '';
                    const llmProvEl = $el('hft-llm-provider');
                    if (llmProvEl) llmProvEl.value = snap.llm_provider || 'openai';
                    const llmEnEl = $el('hft-llm-enabled');
                    if (llmEnEl) llmEnEl.checked = snap.llm_sentiment_enabled === true;
                    // Options Hedging
                    const hedgeOtmEl = $el('hft-hedge-otm');
                    if (hedgeOtmEl) { hedgeOtmEl.value = snap.hedge_otm_pct || 5; hedgeOtmEl.dispatchEvent(new Event('input')); }
                    const hedgePremEl = $el('hft-hedge-premium');
                    if (hedgePremEl) { hedgePremEl.value = Math.round((snap.hedge_max_premium_pct || 1.0) * 10); hedgePremEl.dispatchEvent(new Event('input')); }
                    const optionsEnEl = $el('hft-options-enabled');
                    if (optionsEnEl) optionsEnEl.checked = snap.options_hedging_enabled === true;
                    // Web Dashboard
                    const webEnEl = $el('hft-web-enabled');
                    if (webEnEl) webEnEl.checked = snap.web_dashboard_enabled === true;
                    const webPortEl = $el('hft-web-port');
                    if (webPortEl) webPortEl.value = snap.web_dashboard_port || 7777;

                    // Advanced trading params (Phases 1-9 + N4)
                    const _smSnap = snap.score_modifiers || {};
                    const _cbSnap = snap.classical_bonuses || {};
                    const _nestedFieldMap = {
                        sm_chaos_kill_penalty: _smSnap.chaos_kill_penalty, sm_chaos_kill_fallback: _smSnap.chaos_kill_fallback,
                        sm_chaos_conf_mult_mean_rev: _smSnap.chaos_conf_mult_mean_rev, sm_chaos_conf_mult_chaotic: _smSnap.chaos_conf_mult_chaotic,
                        sm_chaos_conf_mult_unknown: _smSnap.chaos_conf_mult_unknown, sm_regime_mult_mean_rev: _smSnap.regime_mult_mean_rev,
                        sm_regime_mult_chaotic: _smSnap.regime_mult_chaotic, sm_regime_mult_unknown: _smSnap.regime_mult_unknown,
                        sm_vpin_penalty_high: _smSnap.vpin_penalty_high, sm_vpin_penalty_medium: _smSnap.vpin_penalty_medium,
                        sm_fvg_bullish_bonus: _smSnap.fvg_bullish_bonus, sm_fvg_bearish_penalty: _smSnap.fvg_bearish_penalty,
                        sm_rvol_tier1_threshold: _smSnap.rvol_tier1_threshold, sm_rvol_tier1_bonus: _smSnap.rvol_tier1_bonus,
                        sm_rvol_tier2_threshold: _smSnap.rvol_tier2_threshold, sm_rvol_tier2_bonus: _smSnap.rvol_tier2_bonus,
                        sm_stoch_rsi_overbought: _smSnap.stoch_rsi_overbought, sm_stoch_rsi_oversold: _smSnap.stoch_rsi_oversold,
                        sm_bb_squeeze_width: _smSnap.bb_squeeze_width, sm_entry_label_strong: _smSnap.entry_label_strong,
                        sm_entry_label_mild: _smSnap.entry_label_mild,
                        cb_rsi_bonus: _cbSnap.rsi_bonus, cb_ema_alignment_bonus: _cbSnap.ema_alignment_bonus,
                        cb_adx_bonus: _cbSnap.adx_bonus, cb_bb_squeeze_bonus: _cbSnap.bb_squeeze_bonus,
                        cb_vwap_bonus: _cbSnap.vwap_bonus, cb_cap: _cbSnap.cap,
                    };
                    for (const s of hftAdvancedSliders) {
                        const el = $el(s.id);
                        if (!el) continue;
                        const snapVal = s.field in _nestedFieldMap ? _nestedFieldMap[s.field] : snap[s.field];
                        if (snapVal != null) {
                            el.value = s.div > 1 ? Math.round(snapVal * s.div) : snapVal;
                            el.dispatchEvent(new Event('input'));
                        }
                    }

                    // Adaptive exits toggle
                    const adaptiveEl = $el('hft-disable-adaptive');
                    if (adaptiveEl && snap.disable_adaptive_exits) adaptiveEl.classList.add('on');

                    // Multi-asset
                    const multiToggleEl = $el('hft-multi-asset-toggle');
                    if (multiToggleEl && snap.multi_symbol_enabled) multiToggleEl.classList.add('on');
                    const maxConcEl = $el('hft-max-concurrent');
                    if (maxConcEl && snap.max_concurrent_assets) {
                        maxConcEl.value = snap.max_concurrent_assets;
                        maxConcEl.dispatchEvent(new Event('input'));
                    }

                    // TA Entry Gates
                    _restoreTaGates(snap);

                    // Portfolio Risk Gates (S9)
                    const varGateEl = $el('hft-var-gate');
                    if (varGateEl) varGateEl.checked = snap.enable_var_gate === true;
                    const varMaxEl2 = $el('hft-var-max');
                    if (varMaxEl2) {
                        varMaxEl2.value = Math.round((snap.max_portfolio_var_pct || 2.0) * 10);
                        varMaxEl2.dispatchEvent(new Event('input'));
                    }
                    const varLookEl2 = $el('hft-var-lookback');
                    if (varLookEl2) {
                        varLookEl2.value = snap.var_lookback_periods || 20;
                        varLookEl2.dispatchEvent(new Event('input'));
                    }
                    const hhiGateEl = $el('hft-hhi-gate');
                    if (hhiGateEl) hhiGateEl.checked = snap.enable_concentration_gate === true;
                    const hhiMaxEl2 = $el('hft-hhi-max');
                    if (hhiMaxEl2) {
                        hhiMaxEl2.value = Math.round((snap.max_concentration_hhi || 0.4) * 100);
                        hhiMaxEl2.dispatchEvent(new Event('input'));
                    }

                    // Safety Gates (S04.1) restore
                    const sgVarFail = $el('sg-var-fail-closed');
                    if (sgVarFail) sgVarFail.checked = snap.var_fail_closed !== false;
                    const sgAssetHard = $el('sg-asset-check-hard');
                    if (sgAssetHard) sgAssetHard.checked = snap.asset_check_hard_gate !== false;
                    const sgBlFirst = $el('sg-blackout-first');
                    if (sgBlFirst) {
                        sgBlFirst.value = snap.blackout_first_mins ?? 5;
                        sgBlFirst.dispatchEvent(new Event('input'));
                    }
                    const sgBlLast = $el('sg-blackout-last');
                    if (sgBlLast) {
                        sgBlLast.value = snap.blackout_last_mins ?? 5;
                        sgBlLast.dispatchEvent(new Event('input'));
                    }
                    const sgExempt = $el('sg-blackout-exempt');
                    if (sgExempt) sgExempt.value = (snap.blackout_exempt_symbols || []).join(', ');
                    const sgDvol = $el('sg-min-dvol');
                    if (sgDvol) {
                        const dvolM = (snap.scan_params?.min_dollar_volume || 5000000) / 1000000;
                        sgDvol.value = Math.round(dvolM * 10);
                        sgDvol.dispatchEvent(new Event('input'));
                    }
                    // S15 Trading Edge restore
                    const s15ChkDecay = $el('chk-max-hold-decay');
                    if (s15ChkDecay) { const b = s15ChkDecay.querySelector('.chk-box'); if (b) { b.classList.toggle('checked', snap.max_hold_decay_enabled !== false); } }
                    const s15Ds = $el('s15-decay-start');
                    if (s15Ds) { s15Ds.value = Math.round((snap.max_hold_decay_start_pct || 0.5) * 100); s15Ds.dispatchEvent(new Event('input')); }
                    const s15Dt = $el('s15-decay-tighten');
                    if (s15Dt) { s15Dt.value = Math.round((snap.max_hold_decay_trail_tighten || 0.003) * 1000); s15Dt.dispatchEvent(new Event('input')); }
                    const s15ChkMtf = $el('chk-mtf-gate');
                    if (s15ChkMtf) { const b = s15ChkMtf.querySelector('.chk-box'); if (b) { b.classList.toggle('checked', snap.mtf_gate_enabled !== false); } }
                    const s15Mm = $el('s15-mtf-mode');
                    if (s15Mm) s15Mm.value = snap.mtf_gate_mode || 'soft';
                    const s15Mp = $el('s15-mtf-penalty');
                    if (s15Mp) { s15Mp.value = snap.mtf_gate_penalty || 15; s15Mp.dispatchEvent(new Event('input')); }
                    const s15ChkHhi = $el('chk-hhi-manual');
                    if (s15ChkHhi) { const b = s15ChkHhi.querySelector('.chk-box'); if (b) { b.classList.toggle('checked', snap.hhi_gate_manual_orders !== false); } }
                    const s15Hl = $el('s15-hhi-limit');
                    if (s15Hl) { s15Hl.value = Math.round((snap.max_concentration_hhi_manual || 0.35) * 100); s15Hl.dispatchEvent(new Event('input')); }
                    const s15ChkTier = $el('chk-tiered-scan');
                    if (s15ChkTier) { const b = s15ChkTier.querySelector('.chk-box'); if (b) { b.classList.toggle('checked', snap.tiered_scan_enabled !== false); } }

                    // N4: Entry Threshold Matrix + Classical Bonuses visibility
                    _loadEntryThresholds(snap);
                    const cbCard = $el('classical-bonuses-card');
                    const cbStatus = $el('cb-status');
                    if (cbCard && cbStatus) {
                        const isOn = snap.use_classical_filters === true;
                        cbStatus.textContent = isOn ? '(active)' : '(enable Classical Filters above)';
                        cbCard.style.opacity = isOn ? '1' : '0.5';
                    }

                    // Apply locked preset
                    const lock = localStorage.getItem('titan_preset_lock');
                    if (lock === 'safe') {
                        const safeLockEl = $el('hft-safe-lock');
                        if (safeLockEl) safeLockEl.classList.add('on');
                    } else if (lock === 'scalp') {
                        const scalpLockEl = $el('hft-scalp-lock');
                        if (scalpLockEl) scalpLockEl.classList.add('on');
                    }
                } catch (e) { TitanError.report('perf:hft-settings', e); }
            }

            async function wireReplayTab() {
                try {
                    const raw = await invoke('get_replay_symbols');
                    const symbols = Array.isArray(raw) ? raw : [];
                    const sel = $el('replay-symbol');
                    if (sel && symbols.length > 0) {
                        sel.innerHTML = symbols.map(s => `<option value="${escapeHtml(s)}">${escapeHtml(s)}</option>`).join('');
                    } else if (sel) {
                        sel.innerHTML = '<option value="">No trades in journal</option>';
                    }
                } catch(e) { TitanError.report('perf:replay-symbols', e); }

                const today = new Date();
                const monthAgo = new Date(today);
                monthAgo.setDate(monthAgo.getDate() - 30);
                const fromEl = $el('replay-from');
                const toEl = $el('replay-to');
                if (fromEl) fromEl.value = monthAgo.toISOString().split('T')[0];
                if (toEl) toEl.value = today.toISOString().split('T')[0];

                const btn = $el('replay-btn');
                if (btn) btn.addEventListener('click', async () => {
                    const symbol = $el('replay-symbol')?.value;
                    const tf = $el('replay-tf')?.value || '5Min';
                    const dateFrom = $el('replay-from')?.value || '';
                    const dateTo = $el('replay-to')?.value || '';
                    if (!symbol) return;

                    btn.textContent = 'Loading...';
                    try {
                        const data = await invoke('get_replay_data', { symbol, timeframe: tf, dateFrom, dateTo });
                        renderReplayChart(data);
                    } catch(e) { titanLogAdd('Replay error: ' + normalizeError(e)); }
                    btn.textContent = 'REPLAY';
                });
            }

            function renderReplayChart(data) {
                const container = $el('replay-chart-container');
                if (!container) return;
                container.innerHTML = '';

                const chart = LightweightCharts.createChart(container, {
                    width: container.clientWidth,
                    height: 400,
                    layout: { background: { color: '#0a0a0a' }, textColor: '#888' },
                    grid: { vertLines: { color: '#1a1a1a' }, horzLines: { color: '#1a1a1a' } },
                    timeScale: { timeVisible: true },
                });

                const candleSeries = chart.addCandlestickSeries({
                    upColor: '#00BFFF', downColor: '#FF2B2B',
                    borderUpColor: '#00BFFF', borderDownColor: '#FF2B2B',
                    wickUpColor: '#00BFFF', wickDownColor: '#FF2B2B',
                });

                if (data.candles && data.candles.length > 0) {
                    candleSeries.setData(data.candles.map(c => ({
                        time: c.time, open: c.open, high: c.high, low: c.low, close: c.close
                    })));
                }

                if (data.trades && data.trades.length > 0) {
                    const markers = data.trades.map(t => ({
                        time: t.time,
                        position: t.side.toLowerCase() === 'buy' ? 'belowBar' : 'aboveBar',
                        color: sideColor(t.side),
                        shape: t.side.toLowerCase() === 'buy' ? 'arrowUp' : 'arrowDown',
                        text: `${t.side.toUpperCase()} $${t.price.toFixed(2)} ${t.pnl !== 0 ? '($' + t.pnl.toFixed(2) + ')' : ''}`,
                    }));
                    candleSeries.setMarkers(markers);

                    const wins = data.trades.filter(t => t.pnl > 0).length;
                    const totalPnl = data.trades.reduce((s, t) => s + t.pnl, 0);
                    const avgPnl = totalPnl / data.trades.length;
                    const winRate = data.trades.length > 0 ? (wins / data.trades.length * 100) : 0;

                    const el = (id, v) => { const e = $el(id); if (e) e.textContent = v; };
                    el('replay-stat-trades', data.trades.length);
                    el('replay-stat-wr', winRate.toFixed(1) + '%');
                    el('replay-stat-pnl', '$' + totalPnl.toFixed(2));
                    el('replay-stat-avg', '$' + avgPnl.toFixed(2));

                    const tbody = $el('replay-trades-body');
                    if (tbody) {
                        tbody.innerHTML = data.trades.map(t => {
                            const color = pnlColor(t.pnl);
                            const dt = new Date(t.time * 1000);
                            const ts = dt.toISOString().replace('T', ' ').slice(0, 16);
                            return `<tr><td style="padding:4px 6px;">${ts}</td><td style="padding:4px 6px;">${escapeHtml(t.side.toUpperCase())}</td><td style="text-align:right; padding:4px 6px;">$${t.price.toFixed(2)}</td><td style="text-align:right; padding:4px 6px; color:${color};">$${t.pnl.toFixed(2)}</td><td style="padding:4px 6px;">${escapeHtml(t.reason)}</td></tr>`;
                        }).join('');
                    }
                }
                chart.timeScale().fitContent();
            }

            async function loadCorrelationMatrix() {
                const body = $el('corr-matrix-body');
                if (!body) return;
                body.innerHTML = '<div style="color:#555; text-align:center;">Loading...</div>';
                try {
                    const data = await invoke('get_correlation_matrix');
                    if (!data || data.length === 0) {
                        body.innerHTML = '<div style="color:#555; text-align:center; padding:10px;">No correlation data (need 2+ symbols in HFT targets)</div>';
                        return;
                    }
                    let html = '<table style="width:100%; border-collapse:collapse;">';
                    html += '<tr style="border-bottom:1px solid #333;"><th style="text-align:left; padding:3px 6px; color:#888;">Pair</th><th style="text-align:right; padding:3px 6px; color:#888;">Corr</th><th style="text-align:center; padding:3px 6px; color:#888;">Risk</th></tr>';
                    const maxCorr = parseFloat($el('hft-max-corr')?.value || 70) / 100;
                    data.forEach(e => {
                        const abs = Math.abs(e.correlation);
                        const color = abs > maxCorr ? '#FF4444' : abs > 0.5 ? '#FFAA00' : '#00BFFF';
                        const risk = abs > maxCorr ? 'BLOCK' : abs > 0.5 ? 'WARN' : 'OK';
                        html += `<tr style="border-bottom:1px solid #222;"><td style="padding:3px 6px; color:#ccc;">${escapeHtml(e.symbol_a)}/${escapeHtml(e.symbol_b)}</td><td style="text-align:right; padding:3px 6px; color:${color}; font-weight:bold;">${e.correlation.toFixed(2)}</td><td style="text-align:center; padding:3px 6px; color:${color}; font-size:9px;">${risk}</td></tr>`;
                    });
                    html += '</table>';
                    body.innerHTML = html;
                } catch (e) {
                    body.innerHTML = '<div style="color:#FF4444; text-align:center; padding:10px;">Error: ' + escapeHtml(normalizeError(e)) + '</div>';
                }
            }

            async function loadPortfolioPerformance() {
                const body = $el('perf-table-body');
                if (!body) return;
                body.innerHTML = '<div style="color:#555; text-align:center;">Loading...</div>';
                try {
                    const data = await invoke('get_portfolio_performance');
                    if (!data || data.length === 0) {
                        body.innerHTML = '<div style="color:#555; text-align:center; padding:10px;">No trade history yet</div>';
                        return;
                    }
                    let html = '<table style="width:100%; border-collapse:collapse;">';
                    html += '<tr style="border-bottom:1px solid #333;"><th style="text-align:left; padding:3px 4px; color:#888;">Symbol</th><th style="text-align:right; padding:3px 4px; color:#888;">Trades</th><th style="text-align:right; padding:3px 4px; color:#888;">WR%</th><th style="text-align:right; padding:3px 4px; color:#888;">PnL</th><th style="text-align:right; padding:3px 4px; color:#888;">Risk</th><th style="text-align:center; padding:3px 4px; color:#888;">Tag</th></tr>';
                    data.forEach(p => {
                        const pnlClr = pnlColor(p.total_pnl);
                        const tagColors = { STAR: '#00BFFF', OK: '#88CC88', NEW: '#5599FF', WEAK: '#FFAA00', UNDERPERFORMER: '#FF4444' };
                        const tagColor = tagColors[p.tag] || '#888';
                        html += `<tr style="border-bottom:1px solid #222;"><td style="padding:3px 4px; color:#ccc;">${escapeHtml(p.symbol)}</td><td style="text-align:right; padding:3px 4px; color:#aaa;">${p.trades}</td><td style="text-align:right; padding:3px 4px; color:${p.win_rate >= 50 ? '#00BFFF' : '#FFAA00'};">${p.win_rate.toFixed(1)}</td><td style="text-align:right; padding:3px 4px; color:${pnlClr};">$${p.total_pnl.toFixed(2)}</td><td style="text-align:right; padding:3px 4px; color:#ccc;">${p.risk_multiplier.toFixed(1)}x</td><td style="text-align:center; padding:3px 4px; color:${tagColor}; font-size:9px; font-weight:bold;">${escapeHtml(p.tag)}</td></tr>`;
                    });
                    html += '</table>';
                    body.innerHTML = html;
                } catch (e) {
                    body.innerHTML = '<div style="color:#FF4444; text-align:center; padding:10px;">Error: ' + escapeHtml(normalizeError(e)) + '</div>';
                }
            }

            function wireCorrAndPerf() {
                const btnCorr = $el('btn-refresh-corr');
                if (btnCorr) btnCorr.addEventListener('click', loadCorrelationMatrix);

                const btnPerf = $el('btn-refresh-perf');
                if (btnPerf) btnPerf.addEventListener('click', loadPortfolioPerformance);

                const btnRebalance = $el('btn-rebalance');
                if (btnRebalance) btnRebalance.addEventListener('click', async () => {
                    try {
                        const action = await invoke('rebalance_portfolio');
                        let msg = 'Rebalance: ';
                        if (action.remove.length) msg += 'Removed: ' + action.remove.join(', ') + '. ';
                        if (action.promote.length) msg += 'Stars: ' + action.promote.join(', ') + '. ';
                        if (action.demote.length) msg += 'Weak: ' + action.demote.join(', ') + '. ';
                        if (!action.remove.length && !action.promote.length && !action.demote.length) msg += 'No changes needed.';
                        titanLogAdd(msg);
                        loadPortfolioPerformance();
                    } catch (e) {
                        titanLogAdd('Rebalance failed: ' + normalizeError(e));
                    }
                });

                const corrSlider = $el('hft-max-corr');
                const corrVal = $el('hft-max-corr-val');
                if (corrSlider && corrVal) {
                    corrSlider.addEventListener('input', () => {
                        corrVal.textContent = (parseFloat(corrSlider.value) / 100).toFixed(2);
                    });
                }
            }

            TitanBus.on('tabSwitch', function (index) {
                if (index === 4 && !_hftSettingsLoaded) {
                    _hftSettingsLoaded = true;
                    loadHftSettings();
                }
            });

            document.addEventListener('DOMContentLoaded', () => {
                wireHftEngine();
                wireCorrAndPerf();
            });
        })();
