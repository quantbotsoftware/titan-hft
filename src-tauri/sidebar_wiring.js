
// --- TITAN HFT: SIDEBAR WIRING (INJECTED) ---

(function () {
    console.log("⚙️ INITIALIZING SIDEBAR SETTINGS MODULE...");

    // Helper to invoke Rust command
    async function sendSetting(field, value) {
        try {
            await invoke('update_settings', { field, value: String(value) });
            // console.log(`Sent: ${field} = ${value}`);
        } catch (e) {
            console.error(`Failed to update ${field}:`, e);
        }
    }

    // Wiring Map: Text -> Rust Field
    const toggleMap = {
        'ACTIVEAZĂ LOOP': 'loop_active',
        'INTELEGENȚĂ FINANCIARĂ': 'omni_scan_active',
        'TRANZACȚIONARE EXTENDED': 'extended_hours',
        'AUTO-PILOT': 'auto_pilot',
        'AGGRESSIVE MODE': 'aggressive_mode',
        'ALLOW SHORT SELLING': 'allow_short',
        'FORCE MANUAL': 'force_manual'
    };

    // 1. Wire Toggles (find div by sibling span text)
    const allSpans = document.getElementsByTagName('span');
    for (let span of allSpans) {
        const text = span.textContent;
        // Check if text matches any key
        for (let key in toggleMap) {
            if (text.includes(key)) {
                // Found a toggle label. The checkbox div is usually the previous sibling.
                const chk = span.previousElementSibling;
                if (chk && chk.classList.contains('chk-box')) {
                    // console.log(`Wiring Toggle: ${key}`);

                    // Click Handler
                    span.parentElement.style.cursor = 'pointer';
                    span.parentElement.onclick = (e) => {
                        // Toggle Class
                        chk.classList.toggle('checked');
                        const isActive = chk.classList.contains('checked');

                        // Send to Rust
                        sendSetting(toggleMap[key], isActive);

                        // UI Feedback
                        // console.log(`${key} -> ${isActive}`);
                    };
                }
            }
        }
    }

    // 2. Wire Sliders
    // They are generic <input type="range">. We need to identify them by their preceding label.
    // "VITEZĂ REFRESH" -> Next input
    // "RISK PER TRADE" -> Next input
    // "Zile Backtest" -> Next input

    const allInputs = document.getElementsByTagName('input');
    for (let i = 0; i < allInputs.length; i++) {
        const inp = allInputs[i];
        if (inp.type !== 'range') continue;

        // Look at previous element for label
        const prev = inp.previousElementSibling;
        if (prev && prev.classList.contains('st-label')) {
            const labelText = prev.textContent;
            let field = "";
            let scaler = 1;

            if (labelText.includes('REFRESH')) field = "refresh_rate";
            if (labelText.includes('RISK')) field = "risk_per_trade";
            // if(labelText.includes('Zile')) field = "backtest_days";

            if (field) {
                // console.log(`Wiring Slider: ${field}`);
                inp.onchange = (e) => {
                    const val = e.target.value;
                    // Update the number display (next sibling usually)
                    const next = inp.nextElementSibling;
                    if (next) next.innerText = val;

                    sendSetting(field, val);
                };
            }
        }
    }

    // 3. FORCE MANUAL RESOLUTION (Custom Check)
    // There is an input div? "3m"
    // Leave for later if needed, complex structure.

})();
