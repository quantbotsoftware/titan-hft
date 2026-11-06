// ╔══════════════════════════════════════════════════════════════════╗
// ║  TITAN NEXUS — GPU Multi-Agent Consensus Kernel                ║
// ║  150,000+ Autonomous Trading Agents (Parallel Compute)         ║
// ║                                                                ║
// ║  Strategy Types:                                               ║
// ║    0 = Momentum   (buys when velocity > risk_dna)              ║
// ║    1 = Mean-Rev   (buys when velocity < -risk_dna)             ║
// ║    2 = Breakout   (buys when volatility > risk_dna * 2)        ║
// ║                                                                ║
// ║  Output: Atomic weighted BUY/SELL consensus (8 bytes)          ║
// ╚══════════════════════════════════════════════════════════════════╝

// Agent struct — MUST match Rust #[repr(C)] layout exactly (32 bytes)
struct Agent {
    risk_dna: f32,            // Offset  0: Risk threshold (0.0 → 1.0)
    strategy_type: u32,       // Offset  4: 0=momentum, 1=mean-rev, 2=breakout
    capital_0: f32,           // Offset  8: Position size
    capital_1: f32,           // Offset 12: Cash reserves
    action_state: u32,        // Offset 16: 0=HOLD, 1=BUY, 2=SELL
    _pad0: u32,               // Offset 20: Padding
    _pad1: u32,               // Offset 24: Padding
    _pad2: u32,               // Offset 28: Padding
    // Total: 32 bytes ✓ (std430 aligned)
};

// Market state uniform — pushed per tick from CPU
struct MarketState {
    current_price: f32,
    price_velocity: f32,
    volatility: f32,
    tick_count: u32,
    prev_price: f32,          // For fitness feedback (agent evolution)
    _ms_pad0: u32,
    _ms_pad1: u32,
    _ms_pad2: u32,
};

// Atomic intent counters — GPU-side reduction (only 8 bytes read back)
struct IntentCounters {
    buy_weight: atomic<u32>,
    sell_weight: atomic<u32>,
};

@group(0) @binding(0) var<storage, read_write> agents: array<Agent>;
@group(0) @binding(1) var<uniform> market: MarketState;
@group(0) @binding(2) var<storage, read_write> intent: IntentCounters;
@group(0) @binding(3) var<uniform> agent_count: u32;

// Simple hash for per-agent jitter (avoids all agents acting identically)
fn hash_u32(x: u32) -> f32 {
    var h = x;
    h = h ^ (h >> 16u);
    h = h * 0x45d9f3bu;
    h = h ^ (h >> 16u);
    return f32(h & 0xFFFFu) / 65535.0;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= agent_count) {
        return;
    }

    let agent = agents[idx];
    let risk = agent.risk_dna;
    let strategy = agent.strategy_type;
    let cash = agent.capital_1;

    // Respawn near-bankrupt agents with fresh DNA (prevents pool degradation)
    if (cash < 0.05) {
        let new_dna = hash_u32(idx ^ market.tick_count ^ 0xDEAD) * 0.49 + 0.01;
        agents[idx].risk_dna = new_dna;
        agents[idx].capital_1 = 0.5 + hash_u32(idx ^ market.tick_count ^ 0xBEEF) * 1.5;
        agents[idx].capital_0 = 0.0;
        agents[idx].action_state = 0u;
        return;
    }

    // Per-agent noise for diversity (prevents herding)
    let jitter = hash_u32(idx ^ market.tick_count) * 0.1;
    let adjusted_risk = risk + jitter;

    let vel = market.price_velocity;
    let vol = market.volatility;

    // Confidence = how much capital the agent is willing to commit
    // Scaled to integer for atomicAdd; capped at 1000 to prevent overflow
    // with 150K agents (worst case: 150000 * 1000 = 150M, well within u32 max 4.29B)
    let base_confidence = u32(clamp(cash * 10.0, 1.0, 1000.0));

    var action: u32 = 0u; // 0 = HOLD

    // ═══════════════════════════════════
    //  STRATEGY DISPATCH
    // ═══════════════════════════════════
    switch (strategy) {
        // ── MOMENTUM STRATEGY ──
        // Follows the trend: buys when price is accelerating up
        case 0u: {
            if (vel > adjusted_risk) {
                action = 1u; // BUY
                atomicAdd(&intent.buy_weight, base_confidence);
            } else if (vel < -adjusted_risk) {
                action = 2u; // SELL
                atomicAdd(&intent.sell_weight, base_confidence);
            }
        }

        // ── MEAN-REVERSION STRATEGY ──
        // Contrarian: buys the dip, sells the rip
        case 1u: {
            if (vel < -adjusted_risk) {
                action = 1u; // BUY (oversold bounce)
                atomicAdd(&intent.buy_weight, base_confidence);
            } else if (vel > adjusted_risk) {
                action = 2u; // SELL (overbought reversal)
                atomicAdd(&intent.sell_weight, base_confidence);
            }
        }

        // ── BREAKOUT STRATEGY ──
        // Volatility expansion: enters when market is moving big
        case 2u: {
            let vol_threshold = adjusted_risk * 2.0;
            if (vol > vol_threshold) {
                // Direction determined by velocity sign
                if (vel > 0.0) {
                    action = 1u; // BUY breakout
                    atomicAdd(&intent.buy_weight, base_confidence);
                } else {
                    action = 2u; // SELL breakdown
                    atomicAdd(&intent.sell_weight, base_confidence);
                }
            }
        }

        default: {
            // Unknown strategy type — HOLD
        }
    }

    // AGENT EVOLUTION — Reward correct predictions, punish wrong ones
    // Reads prev_price to determine if last tick's action was correct
    let prev_action = agent.action_state;
    let price_delta = market.current_price - market.prev_price;

    if (prev_action == 1u && price_delta > 0.0) || (prev_action == 2u && price_delta < 0.0) {
        // Correct prediction — grow capital (capped at 5.0)
        agents[idx].capital_1 = min(agents[idx].capital_1 * 1.002, 5.0);
    } else if (prev_action != 0u) {
        // Wrong prediction — shrink capital
        agents[idx].capital_1 = agents[idx].capital_1 * 0.998;
    }

    // Update agent state (visible next tick)
    agents[idx].action_state = action;
}
