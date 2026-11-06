(function () {
    'use strict';

    const STORAGE_KEY = 'titan_audio_enabled';
    const TITLE_ORIGINAL = document.title;

    let _audioCtx = null;
    let _enabled = true;
    let _volume = 0.35;
    let _initialized = false;
    let _flashTimer = null;
    let _glowTimer = null;

    function _loadPref() {
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            if (raw !== null) _enabled = raw === 'true';
        } catch (_) { /* storage unavailable */ }
    }

    function _savePref() {
        try { localStorage.setItem(STORAGE_KEY, String(_enabled)); } catch (_) {}
    }

    function _ensureCtx() {
        if (_audioCtx && _audioCtx.state !== 'closed') {
            if (_audioCtx.state === 'suspended') _audioCtx.resume().catch(() => {});
            return _audioCtx;
        }
        const Ctor = window.AudioContext || window.webkitAudioContext;
        if (!Ctor) return null;
        try {
            _audioCtx = new Ctor();
            return _audioCtx;
        } catch (_) { return null; }
    }

    function _initOnGesture() {
        if (_initialized) return;
        _initialized = true;
        _ensureCtx();
    }

    document.addEventListener('click', _initOnGesture, { once: true });
    document.addEventListener('keydown', _initOnGesture, { once: true });

    function _osc(ctx, type, freq, startT, dur, vol) {
        const o = ctx.createOscillator();
        const g = ctx.createGain();
        o.type = type;
        o.frequency.value = freq;
        g.gain.setValueAtTime(0.0001, startT);
        g.gain.exponentialRampToValueAtTime(vol, startT + 0.01);
        g.gain.exponentialRampToValueAtTime(0.0001, startT + dur);
        o.connect(g).connect(ctx.destination);
        o.start(startT);
        o.stop(startT + dur + 0.01);
    }

    function _playFillEntry(ctx, now, vol) {
        _osc(ctx, 'sine', 520, now, 0.06, vol);
        _osc(ctx, 'sine', 780, now + 0.07, 0.08, vol * 0.9);
        _osc(ctx, 'sine', 1040, now + 0.15, 0.06, vol * 0.7);
    }

    function _playFillProfit(ctx, now, vol) {
        _osc(ctx, 'sine', 880, now, 0.08, vol);
        _osc(ctx, 'sine', 1320, now + 0.1, 0.08, vol * 0.9);
        _osc(ctx, 'triangle', 1760, now + 0.2, 0.12, vol * 0.6);
    }

    function _playFillLoss(ctx, now, vol) {
        _osc(ctx, 'sawtooth', 220, now, 0.15, vol * 0.5);
        _osc(ctx, 'sawtooth', 180, now + 0.15, 0.2, vol * 0.4);
    }

    function _playBreaker(ctx, now, vol) {
        for (let i = 0; i < 3; i++) {
            const t = now + i * 0.35;
            _osc(ctx, 'square', 600, t, 0.12, vol * 0.4);
            _osc(ctx, 'square', 750, t + 0.12, 0.08, vol * 0.3);
        }
    }

    function _playFlashCrash(ctx, now, vol) {
        _osc(ctx, 'sawtooth', 440, now, 0.2, vol * 0.6);
        _osc(ctx, 'sawtooth', 330, now + 0.25, 0.3, vol * 0.5);
        _osc(ctx, 'square', 220, now + 0.6, 0.4, vol * 0.4);
    }

    function _playSignal(ctx, now, vol) {
        _osc(ctx, 'sine', 1000, now, 0.1, vol * 0.7);
        _osc(ctx, 'sine', 1200, now + 0.12, 0.08, vol * 0.5);
    }

    function _playWatchdog(ctx, now, vol) {
        _osc(ctx, 'triangle', 400, now, 0.15, vol * 0.5);
        _osc(ctx, 'triangle', 500, now + 0.2, 0.15, vol * 0.4);
        _osc(ctx, 'triangle', 400, now + 0.4, 0.15, vol * 0.3);
    }

    function _playStStalking(ctx, now, vol) {
        _osc(ctx, 'sine', 600, now, 0.08, vol * 0.4);
        _osc(ctx, 'sine', 800, now + 0.09, 0.06, vol * 0.35);
    }

    function _playStEntry(ctx, now, vol) {
        _osc(ctx, 'sine', 880, now, 0.05, vol * 0.7);
        _osc(ctx, 'sine', 1100, now + 0.06, 0.05, vol * 0.65);
        _osc(ctx, 'sine', 880, now + 0.12, 0.05, vol * 0.5);
        _osc(ctx, 'sine', 1100, now + 0.18, 0.05, vol * 0.45);
    }

    function _playStExit(ctx, now, vol) {
        _osc(ctx, 'sine', 900, now, 0.1, vol * 0.6);
        _osc(ctx, 'sine', 700, now + 0.12, 0.1, vol * 0.5);
        _osc(ctx, 'sine', 500, now + 0.24, 0.15, vol * 0.4);
    }

    function _playStTrail(ctx, now, vol) {
        _osc(ctx, 'triangle', 1200, now, 0.04, vol * 0.5);
        _osc(ctx, 'triangle', 1400, now + 0.05, 0.06, vol * 0.4);
    }

    function _playStChangepoint(ctx, now, vol) {
        for (let i = 0; i < 4; i++) {
            const f = 300 + Math.random() * 600;
            _osc(ctx, 'sawtooth', f, now + i * 0.04, 0.03, vol * 0.25);
        }
        _osc(ctx, 'square', 200, now + 0.18, 0.08, vol * 0.3);
    }

    function _playStKill(ctx, now, vol) {
        _osc(ctx, 'square', 800, now, 0.08, vol * 0.5);
        _osc(ctx, 'square', 600, now + 0.1, 0.08, vol * 0.45);
        _osc(ctx, 'square', 800, now + 0.2, 0.08, vol * 0.4);
    }

    const _sounds = {
        'fill-entry': _playFillEntry,
        'fill-exit-profit': _playFillProfit,
        'fill-exit-loss': _playFillLoss,
        'breaker': _playBreaker,
        'flash-crash': _playFlashCrash,
        'signal': _playSignal,
        'watchdog': _playWatchdog,
        'st_stalking': _playStStalking,
        'st_entry': _playStEntry,
        'st_exit': _playStExit,
        'st_trail': _playStTrail,
        'st_changepoint': _playStChangepoint,
        'st_kill': _playStKill,
    };

    function play(eventType) {
        if (!_enabled) return;
        const fn = _sounds[eventType];
        if (!fn) return;
        const ctx = _ensureCtx();
        if (!ctx) return;
        try {
            fn(ctx, ctx.currentTime, _volume);
        } catch (e) {
            if (typeof TitanError !== 'undefined') TitanError.report('audio:play', e);
        }
    }

    function flashTitle(message, durationMs) {
        const dur = durationMs || 3000;
        let tick = 0;
        clearInterval(_flashTimer);
        _flashTimer = setInterval(() => {
            document.title = tick % 2 === 0 ? message : TITLE_ORIGINAL;
            tick++;
        }, 500);
        setTimeout(() => {
            clearInterval(_flashTimer);
            _flashTimer = null;
            document.title = TITLE_ORIGINAL;
        }, dur);
    }

    function showScreenGlow(color, durationMs) {
        let overlay = $el('titan-screen-glow');
        if (!overlay) {
            overlay = document.createElement('div');
            overlay.id = 'titan-screen-glow';
            overlay.className = 'titan-screen-glow';
            document.body.appendChild(overlay);
        }
        overlay.style.boxShadow = `inset 0 0 80px 30px ${color}`;
        overlay.classList.add('active');
        clearTimeout(_glowTimer);
        _glowTimer = setTimeout(() => {
            overlay.classList.remove('active');
        }, durationMs || 2000);
    }

    function setEnabled(val) {
        _enabled = !!val;
        _savePref();
        const btn = $el('btn-titan-audio-toggle');
        if (btn) {
            btn.textContent = _enabled ? 'AUDIO: ON' : 'AUDIO: OFF';
            btn.classList.toggle('on', _enabled);
            btn.classList.toggle('off', !_enabled);
        }
    }

    function isEnabled() { return _enabled; }

    function toggle() { setEnabled(!_enabled); }

    function setVolume(v) {
        _volume = Math.max(0.05, Math.min(1.0, Number(v) || 0.35));
    }

    function wireEvents() {
        const btn = $el('btn-titan-audio-toggle');
        if (btn) {
            btn.textContent = _enabled ? 'AUDIO: ON' : 'AUDIO: OFF';
            btn.classList.toggle('on', _enabled);
            btn.classList.toggle('off', !_enabled);
        }

        if (!window.__TAURI__) return;
        const { listen } = window.__TAURI__.event;

        listen('trade-fill', (event) => {
            const f = event.payload || {};
            const side = (f.side || '').toUpperCase();
            play('fill-entry');
            flashTitle('FILL: ' + side + ' ' + (f.symbol || '?'), 2500);
        });

        listen('breaker-triggered', (event) => {
            const b = event.payload || {};
            const level = (b.level || '').toUpperCase();
            if (level === 'NORMAL') return;
            play('breaker');
            flashTitle('BREAKER ' + level, 5000);
            if (level === 'PANIC' || level === 'HALT') {
                showScreenGlow('rgba(255, 43, 43, 0.35)', 4000);
            } else if (level === 'CAUTION') {
                showScreenGlow('rgba(255, 215, 0, 0.25)', 2500);
            }
        });

        listen('flash-crash', (event) => {
            const d = event.payload || {};
            play('flash-crash');
            flashTitle('FLASH CRASH: ' + (d.symbol || '?'), 6000);
            showScreenGlow('rgba(255, 0, 100, 0.4)', 5000);
        });

        listen('watchdog-alert', (event) => {
            const dead = (event.payload || {}).dead_loops || [];
            if (dead.length > 0) {
                play('watchdog');
                flashTitle('WATCHDOG: ' + dead.length + ' DEAD', 4000);
            }
        });
    }

    _loadPref();

    window.TitanAudio = {
        play,
        flashTitle,
        showScreenGlow,
        setEnabled,
        isEnabled,
        toggle,
        setVolume,
        wireEvents,
    };
})();
