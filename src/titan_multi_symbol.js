        (function () {
            function wireMultiSymbol() {
                const chk = $el('chk-multi-symbol-central');
                if (!chk) return;
                const box = chk.querySelector('.chk-box');
                chk.addEventListener('click', async () => {
                    const active = box.classList.toggle('checked');
                    box.textContent = active ? '✓' : '';
                    await invoke('update_settings', { field: 'multi_symbol_enabled', value: String(active) });
                    titanLogAdd(active ? 'Multi-symbol autopilot ENABLED' : 'Multi-symbol autopilot DISABLED');
                    updateMultiStatus();
                });
            }

            async function updateMultiStatus() {
                try {
                    const data = await invoke('get_multi_status');
                    const chk = $el('chk-multi-symbol-central');
                    if (chk) {
                        const box = chk.querySelector('.chk-box');
                        if (data.enabled) { box.classList.add('checked'); box.textContent = '✓'; }
                        else { box.classList.remove('checked'); box.textContent = ''; }
                    }
                    const el = $el('multi-positions-central');
                    if (el && data.positions && data.positions.length > 0) {
                        el.innerHTML = data.positions.map(p => {
                            const color = pnlColor(p.pnl);
                            return `<div style="display:flex; justify-content:space-between; padding:2px 0; border-bottom:1px solid #1a1a2e;">
                                <span style="color:#fff;">${escapeHtml(p.symbol)}</span>
                                <span style="color:${color};">$${p.pnl.toFixed(2)}</span>
                            </div>`;
                        }).join('');
                    } else if (el) {
                        el.innerHTML = '<div style="color:#555; font-size:10px;">Nicio pozitie activa</div>';
                    }
                } catch (e) { TitanError.report('multi:status-poll', e); }
            }

            document.addEventListener('DOMContentLoaded', () => {
                wireMultiSymbol();
                TitanTimers.register('multi:status', updateMultiStatus, 15000);
                setTimeout(updateMultiStatus, 3000);
            });
        })();
