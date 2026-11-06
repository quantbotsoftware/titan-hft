// ═══════════════════════════════════════════════════════════════
//  S16: Titan Init — Module Entrypoint
//  Orchestrates initialization order for all Titan modules
// ═══════════════════════════════════════════════════════════════
(function titanInit() {
    'use strict';

    const INIT_ORDER = [
        { name: 'core',       fn: () => typeof TitanDebug !== 'undefined' && TitanDebug.log('init', 'Core ready') },
        { name: 'tabs',       fn: () => typeof bindTabs === 'function' && bindTabs() },
        { name: 'sidebar',    fn: () => typeof initSidebar === 'function' && initSidebar() },
        { name: 'chart',      fn: () => typeof initChart === 'function' && initChart() },
        { name: 'keyboard',   fn: () => typeof TitanKeyboard !== 'undefined' },
        { name: 'audio',      fn: () => typeof TitanAudio !== 'undefined' },
        { name: 'toasts',     fn: () => typeof TitanToast !== 'undefined' },
    ];

    window._titanInitComplete = false;
    window._titanInitTimestamp = Date.now();

    function runInit() {
        for (const step of INIT_ORDER) {
            try {
                step.fn();
            } catch (e) {
                if (typeof TitanDebug !== 'undefined') {
                    TitanDebug.log('init', 'Init step ' + step.name + ' failed:', e);
                }
            }
        }
        window._titanInitComplete = true;

        if (typeof TitanDebug !== 'undefined') {
            const elapsed = Date.now() - window._titanInitTimestamp;
            TitanDebug.log('init', 'Titan initialized in ' + elapsed + 'ms');
        }
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', runInit);
    } else {
        runInit();
    }
})();
