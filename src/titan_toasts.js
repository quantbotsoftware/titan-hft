/* ═══════════════════════════════════════════════════════════════════
   TITAN TOASTS -- Unified notification system
   Replaces: alert(), confirm(), hftToast, signalAlertToast, newMoverToast
   ═══════════════════════════════════════════════════════════════════ */
const TitanToasts = (function () {
    'use strict';

    const MAX_VISIBLE = 5;
    const CONTAINER_ID = 'titan-toast-container';
    const CONFIRM_OVERLAY_ID = 'titan-confirm-overlay';

    const DEFAULTS = {
        success: { duration: 3000, icon: '\u2714' },
        error:   { duration: 6000, icon: '\u2716' },
        warning: { duration: 5000, icon: '\u26A0' },
        info:    { duration: 3000, icon: '\u2139' },
    };

    let _container = null;

    function _ensureContainer() {
        if (_container && document.body.contains(_container)) return _container;
        _container = document.createElement('div');
        _container.id = CONTAINER_ID;
        document.body.appendChild(_container);
        return _container;
    }

    function show(message, type, durationMs) {
        type = type || 'info';
        const cfg = DEFAULTS[type] || DEFAULTS.info;
        const dur = durationMs != null ? durationMs : cfg.duration;

        const container = _ensureContainer();

        const el = document.createElement('div');
        el.className = 'titan-toast titan-toast-' + type;

        const iconSpan = document.createElement('span');
        iconSpan.className = 'titan-toast-icon';
        iconSpan.textContent = cfg.icon;
        el.appendChild(iconSpan);

        const msgSpan = document.createElement('span');
        msgSpan.className = 'titan-toast-msg';
        msgSpan.textContent = message;
        el.appendChild(msgSpan);

        const closeBtn = document.createElement('span');
        closeBtn.className = 'titan-toast-close';
        closeBtn.textContent = '\u00D7';
        closeBtn.addEventListener('click', function () { _dismiss(el); });
        el.appendChild(closeBtn);

        container.appendChild(el);
        void el.offsetHeight;
        el.classList.add('titan-toast-show');

        const children = container.querySelectorAll('.titan-toast');
        if (children.length > MAX_VISIBLE) {
            _dismiss(children[0]);
        }

        if (dur > 0) {
            el._timer = setTimeout(function () { _dismiss(el); }, dur);
        }

        return el;
    }

    function _dismiss(el) {
        if (!el || el._dismissed) return;
        el._dismissed = true;
        clearTimeout(el._timer);
        el.classList.remove('titan-toast-show');
        el.classList.add('titan-toast-hide');
        setTimeout(function () { el.remove(); }, 300);
    }

    function confirm(message, opts) {
        opts = opts || {};
        return new Promise(function (resolve) {
            let settled = false;
            const overlay = document.createElement('div');
            overlay.id = CONFIRM_OVERLAY_ID;
            overlay.className = 'titan-confirm-overlay';

            const box = document.createElement('div');
            box.className = 'titan-confirm-box';

            const msgEl = document.createElement('div');
            msgEl.className = 'titan-confirm-msg';
            msgEl.textContent = message;
            box.appendChild(msgEl);

            const actions = document.createElement('div');
            actions.className = 'titan-confirm-actions';

            const yesBtn = document.createElement('button');
            yesBtn.className = 'titan-confirm-btn titan-confirm-yes';
            yesBtn.textContent = opts.yesLabel || 'CONFIRM';

            const noBtn = document.createElement('button');
            noBtn.className = 'titan-confirm-btn titan-confirm-no';
            noBtn.textContent = opts.noLabel || 'CANCEL';

            function onKeydown(e) {
                if (e.key === 'Escape') {
                    e.preventDefault();
                    finish(false);
                } else if (e.key === 'Enter') {
                    e.preventDefault();
                    finish(true);
                }
            }

            function finish(val) {
                if (settled) return;
                settled = true;
                document.removeEventListener('keydown', onKeydown);
                overlay.remove();
                resolve(val);
            }

            yesBtn.addEventListener('click', function () { finish(true); });
            noBtn.addEventListener('click', function () { finish(false); });

            actions.appendChild(yesBtn);
            actions.appendChild(noBtn);
            box.appendChild(actions);
            overlay.appendChild(box);

            overlay.addEventListener('click', function (e) {
                if (e.target === overlay) finish(false);
            });

            document.addEventListener('keydown', onKeydown);
            document.body.appendChild(overlay);
            yesBtn.focus();
        });
    }

    return { show: show, confirm: confirm };
})();

function showCyberToast(message, type, durationMs) {
    return TitanToasts.show(message, type, durationMs);
}

function cyberConfirm(message, opts) {
    return TitanToasts.confirm(message, opts);
}
