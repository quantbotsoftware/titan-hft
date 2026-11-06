#!/bin/bash
set -euo pipefail

PROJECT="$(cd "$(dirname "$0")" && pwd)"
BINARY="titan-hft-v7-reloaded"
APP_ID="com.titan.hft.v7reloaded"
INSTALL_PATH="${HOME}/.local/bin/${BINARY}"
DESKTOP_FILE="${HOME}/.local/share/applications/${BINARY}.desktop"
SYSTEM_INSTALL_PATH="/usr/bin/${BINARY}"
SYSTEM_DESKTOP_FILE="/usr/share/applications/${BINARY}.desktop"
LOG_DIR="${HOME}/HFT_Bot/logs"
LOG_FILE="${LOG_DIR}/titan_$(date +%Y%m%d_%H%M%S).log"
BUILD_START=$SECONDS

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

TOTAL_STEPS=13

step_banner() {
    echo -e "${CYAN}[${1}/${TOTAL_STEPS}]${NC} ${2}"
}

step_ok() {
    echo -e "${GREEN}✓ ${1}${NC}"
}

step_fail() {
    echo -e "${RED}✗ ${1}${NC}" >&2
}

require_cmd() {
    local c="$1"
    if ! command -v "$c" &>/dev/null; then
        step_fail "Required command '${c}' not found. Please install it first."
        exit 1
    fi
}

require_cmd cargo
require_cmd rsync
require_cmd node
require_cmd python3

validate_first_party_js() {
    local count=0
    local f
    while IFS= read -r -d '' f; do
        [[ "${f}" == *"/lightweight-charts.js" ]] && continue
        node --check "${f}"
        count=$((count + 1))
    done < <(find "${PROJECT}/src" "${PROJECT}/src-tauri" \
        -type f -name '*.js' \
        ! -path '*/node_modules/*' \
        ! -path '*/dist/*' \
        ! -path '*/target/*' \
        ! -path '*/.git/*' \
        -print0)
    echo "${count}"
}

echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  TITAN HFT — VALIDATE, BUILD & DEPLOY ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
echo -e "${YELLOW}Start:${NC} $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# ─── Step 1: First-party JS syntax ───────────────────────────
step_banner 1 "Validating first-party JavaScript (node --check)..."
JS_COUNT="$(validate_first_party_js)"
step_ok "${JS_COUNT} JS file(s) syntax-checked"

# ─── Step 2: DOM regression guard ────────────────────────────
step_banner 2 "DOM regression guard (no direct getElementById in migrated modules)..."
bash "${PROJECT}/guard_no_direct_getelementbyid.sh"
step_ok "DOM guard passed"

# ─── Step 3: Static UI smoke (headless Chrome + CDP) ─────────
if [[ "${TITAN_SKIP_STATIC_UI_SMOKE:-}" == "1" ]]; then
    echo -e "${YELLOW}[3/${TOTAL_STEPS}] Skipping static UI smoke (TITAN_SKIP_STATIC_UI_SMOKE=1)${NC}"
else
    step_banner 3 "Static UI smoke (headless Chrome, CDP)..."
    export PROJECT
    python3 - <<'PY'
import asyncio
import json
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

try:
    import requests
    import websockets
except ImportError as e:
    print(
        "Static UI smoke FAILED: Python modules 'requests' and/or 'websockets' are missing "
        f"({e}). Install with: python3 -m pip install requests websockets\n"
        "To skip this step intentionally, set: TITAN_SKIP_STATIC_UI_SMOKE=1",
        file=sys.stderr,
    )
    sys.exit(1)

PROJECT = os.environ.get("PROJECT", "")
if not PROJECT or not os.path.isdir(PROJECT):
    print("Static UI smoke FAILED: PROJECT not set or invalid.", file=sys.stderr)
    sys.exit(1)

ALLOW_CONSOLE = "[TITAN data:invalid-response]"


def pick_chrome():
    env_bin = os.environ.get("TITAN_CHROME_BIN", "").strip()
    candidates = [env_bin] if env_bin else []
    candidates += [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
    ]
    seen = set()
    for name in candidates:
        if not name or name in seen:
            continue
        seen.add(name)
        if os.path.isfile(name) and os.access(name, os.X_OK):
            return name
        w = shutil.which(name)
        if w:
            return w
    return None


def free_port():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def console_text(params):
    parts = []
    for a in params.get("args", []) or []:
        t = a.get("type")
        if t == "string":
            parts.append(a.get("value", ""))
        elif t == "number":
            parts.append(str(a.get("value", "")))
        elif t == "boolean":
            parts.append(str(a.get("value", "")))
        elif t == "object" and a.get("description"):
            parts.append(a["description"])
        elif t == "undefined":
            parts.append("undefined")
        else:
            parts.append(json.dumps(a, default=str))
    return " ".join(parts)


class CdpSession:
    def __init__(self, ws, loop):
        self.ws = ws
        self.loop = loop
        self._seq = 0
        self._pending = {}
        self.violations = []

    def _allow_console_line(self, text):
        t = (text or "").strip()
        return ALLOW_CONSOLE in t

    def on_event(self, msg):
        m = msg.get("method")
        if m == "Runtime.exceptionThrown":
            d = msg.get("params", {}).get("exceptionDetails", {})
            desc = d.get("exception", {}).get("description") or d.get("text") or json.dumps(d)
            if not self._allow_console_line(desc):
                self.violations.append(f"Runtime.exceptionThrown: {desc}")
        elif m == "Runtime.consoleAPICalled":
            p = msg.get("params", {})
            if p.get("type") != "error":
                return
            text = console_text(p)
            if not self._allow_console_line(text):
                self.violations.append(f"console.error: {text}")

    async def pump(self):
        try:
            async for raw in self.ws:
                msg = json.loads(raw)
                mid = msg.get("id")
                if mid is not None and mid in self._pending:
                    self._pending.pop(mid).set_result(msg)
                else:
                    self.on_event(msg)
        except asyncio.CancelledError:
            raise

    async def cmd(self, method, params=None, timeout=60.0):
        self._seq += 1
        mid = self._seq
        fut = self.loop.create_future()
        self._pending[mid] = fut
        await self.ws.send(
            json.dumps({"id": mid, "method": method, "params": params or {}})
        )
        return await asyncio.wait_for(fut, timeout=timeout)

    async def eval_js(self, expression, await_promise=False, timeout=60.0):
        r = await self.cmd(
            "Runtime.evaluate",
            {
                "expression": expression,
                "awaitPromise": await_promise,
                "returnByValue": True,
            },
            timeout=timeout,
        )
        if r.get("error"):
            raise RuntimeError(r["error"])
        outer = r.get("result") or {}
        if "exceptionDetails" in outer:
            raise RuntimeError(json.dumps(outer["exceptionDetails"]))
        res = outer.get("result", {})
        if res.get("subtype") == "error" or res.get("className") == "Error":
            raise RuntimeError(res.get("description", str(res)))
        return res.get("value")


async def dispatch_key_combo(session, ctrl=False, shift=False):
    mod = 0
    if ctrl or shift:
        mod = 2 * int(ctrl) + 8 * int(shift)
    for typ in ("keyDown", "keyUp"):
        await session.cmd(
            "Input.dispatchKeyEvent",
            {
                "type": typ,
                "modifiers": mod,
                "key": "Enter",
                "code": "Enter",
                "windowsVirtualKeyCode": 13,
                "nativeVirtualKeyCode": 13,
            },
        )


async def wait_for_pred(session, js_expr, timeout=15.0, interval=0.1):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        v = await session.eval_js(js_expr)
        if v:
            return True
        await asyncio.sleep(interval)
    return False


async def run_smoke():
    chrome = pick_chrome()
    if not chrome:
        print(
            "Static UI smoke FAILED: no Chrome/Chromium binary found. "
            "Set TITAN_CHROME_BIN or install google-chrome / chromium.\n"
            "To skip intentionally: TITAN_SKIP_STATIC_UI_SMOKE=1",
            file=sys.stderr,
        )
        sys.exit(1)

    http_port = free_port()
    dbg_port = free_port()

    class Handler(SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=os.path.join(PROJECT, "src"), **kwargs)

        def log_message(self, fmt, *args):
            pass

    httpd = ThreadingHTTPServer(("127.0.0.1", http_port), Handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{http_port}/index.html"

    user_data_dir = tempfile.mkdtemp(prefix="titan_static_ui_smoke_chrome_")
    proc = None
    try:
        proc = subprocess.Popen(
            [
                chrome,
                f"--user-data-dir={user_data_dir}",
                f"--remote-debugging-port={dbg_port}",
                "--headless=new",
                "--no-sandbox",
                "--disable-dev-shm-usage",
                "--disable-gpu",
                "about:blank",
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        ws_url = None
        for _ in range(80):
            try:
                tabs = requests.get(
                    f"http://127.0.0.1:{dbg_port}/json/list", timeout=0.5
                ).json()
                for t in tabs:
                    if t.get("type") == "page" and t.get("webSocketDebuggerUrl"):
                        ws_url = t["webSocketDebuggerUrl"]
                        break
            except Exception:
                pass
            if ws_url:
                break
            time.sleep(0.1)
        if not ws_url:
            print("Static UI smoke FAILED: could not attach to Chrome DevTools.", file=sys.stderr)
            sys.exit(1)

        loop = asyncio.get_running_loop()
        async with websockets.connect(ws_url, max_size=2**25) as ws:
            session = CdpSession(ws, loop)
            pump = asyncio.create_task(session.pump())
            try:
                await session.cmd("Runtime.enable")
                await session.cmd("Log.enable")
                await session.cmd("Page.enable")
                await session.cmd(
                    "Page.addScriptToEvaluateOnNewDocument",
                    {
                        "source": """
window.__titan_smoke = { errs: [] };
window.addEventListener('error', function (e) {
  window.__titan_smoke.errs.push('window.error:' + (e && e.message));
});
window.addEventListener('unhandledrejection', function (e) {
  window.__titan_smoke.errs.push('unhandledrejection:' + e.reason);
});
"""
                    },
                )
                await session.cmd("Page.navigate", {"url": base})
                await asyncio.sleep(0.3)
                if not await wait_for_pred(
                    session,
                    "document.readyState === 'complete'",
                    timeout=30.0,
                ):
                    raise RuntimeError("page did not reach complete state")
                await asyncio.sleep(3.0)

                await session.eval_js(
                    """
(() => {
  const p = document.getElementById('manual-trade-panel');
  if (p) p.open = true;
  const sym = document.getElementById('mt-symbol');
  const usd = document.getElementById('mt-usd');
  if (sym) sym.value = 'AAPL';
  if (usd) usd.value = '100';
  if (typeof setSymbol === 'function') setSymbol('AAPL');
})()
"""
                )

                await session.eval_js("document.body && document.body.focus && document.body.focus()")
                await dispatch_key_combo(session, ctrl=True, shift=False)
                ok = await wait_for_pred(
                    session,
                    "Array.from(document.querySelectorAll('.titan-toast-msg')).some("
                    "el => (el.textContent || '').indexOf('BUY AAPL:') >= 0)",
                    timeout=10.0,
                )
                if not ok:
                    raise RuntimeError("quick buy toast missing 'BUY AAPL:'")

                await dispatch_key_combo(session, ctrl=True, shift=True)
                ok = await wait_for_pred(
                    session,
                    "Array.from(document.querySelectorAll('.titan-toast-msg')).some("
                    "el => (el.textContent || '').indexOf('SELL AAPL:') >= 0)",
                    timeout=10.0,
                )
                if not ok:
                    raise RuntimeError("quick sell toast missing 'SELL AAPL:'")

                await session.eval_js(
                    "typeof switchTab === 'function' && switchTab(4)", await_promise=False
                )
                await asyncio.sleep(0.5)

                kill_sel = "document.getElementById('btn-emergency-kill')"
                await session.eval_js(
                    f"""
(async () => {{
  const b = {kill_sel};
  if (!b) throw new Error('btn-emergency-kill missing');
  b.click();
}})()
""",
                    await_promise=True,
                )
                await asyncio.sleep(0.2)
                has_ov = await session.eval_js(
                    "!!document.getElementById('titan-confirm-overlay')"
                )
                if not has_ov:
                    raise RuntimeError("KILL ALL did not open cyberConfirm overlay")
                await session.eval_js(
                    """
(() => {
  const n = document.querySelector('#titan-confirm-overlay .titan-confirm-no');
  if (n) n.click();
})()
"""
                )
                await asyncio.sleep(0.2)
                if await session.eval_js("!!document.getElementById('titan-confirm-overlay')"):
                    raise RuntimeError("KILL ALL cancel did not close overlay")
                txt = await session.eval_js(
                    f"{kill_sel} && {kill_sel}.textContent"
                )
                dis = await session.eval_js(f"{kill_sel} && {kill_sel}.disabled")
                if (txt or "") != "KILL ALL" or dis:
                    raise RuntimeError("KILL ALL button did not reset after cancel")

                await session.eval_js(
                    f"""
(async () => {{
  const b = {kill_sel};
  b.click();
}})()
""",
                    await_promise=True,
                )
                await asyncio.sleep(0.2)
                await session.eval_js(
                    """
(() => {
  const y = document.querySelector('#titan-confirm-overlay .titan-confirm-yes');
  if (y) y.click();
})()
"""
                )
                await asyncio.sleep(1.0)
                if await session.eval_js("!!document.getElementById('titan-confirm-overlay')"):
                    raise RuntimeError("KILL ALL confirm did not close overlay")
                txt = await session.eval_js(f"{kill_sel} && {kill_sel}.textContent")
                dis = await session.eval_js(f"{kill_sel} && {kill_sel}.disabled")
                if (txt or "") != "KILL ALL" or dis:
                    raise RuntimeError("KILL ALL button did not reset after confirm")

                ca = await session.eval_js(
                    """
(async () => {
  const p = cyberConfirm('smoke-cancel-path');
  await new Promise(r => setTimeout(r, 30));
  const n = document.querySelector('#titan-confirm-overlay .titan-confirm-no');
  if (!n) throw new Error('no cancel btn');
  n.click();
  const v = await p;
  return v === false;
})()
""",
                    await_promise=True,
                )
                if not ca:
                    raise RuntimeError("cyberConfirm cancel path failed")
                if await session.eval_js("!!document.getElementById('titan-confirm-overlay')"):
                    raise RuntimeError("cyberConfirm overlay still open after cancel")

                cy = await session.eval_js(
                    """
(async () => {
  const p = cyberConfirm('smoke-confirm-path');
  await new Promise(r => setTimeout(r, 30));
  const y = document.querySelector('#titan-confirm-overlay .titan-confirm-yes');
  if (!y) throw new Error('no yes btn');
  y.click();
  const v = await p;
  return v === true;
})()
""",
                    await_promise=True,
                )
                if not cy:
                    raise RuntimeError("cyberConfirm confirm path failed")

                await session.eval_js(
                    """
(() => {
  const p = document.getElementById('manual-trade-panel');
  if (p) p.open = true;
  const sym = document.getElementById('mt-symbol');
  const usd = document.getElementById('mt-usd');
  if (sym) sym.value = 'AAPL';
  if (usd) usd.value = '100';
})()
"""
                )
                await session.eval_js(
                    """
(async () => {
  const btn = document.getElementById('btn-mt-execute');
  if (!btn) throw new Error('btn-mt-execute missing');
  btn.click();
})()
""",
                    await_promise=True,
                )
                await asyncio.sleep(0.3)
                if not await session.eval_js(
                    "!!document.querySelector('.mt-confirm-overlay')"
                ):
                    raise RuntimeError("Manual trade confirm modal did not open")
                await session.eval_js(
                    """
(() => {
  const c = document.querySelector('.mt-confirm-cancel');
  if (c) c.click();
})()
"""
                )
                await asyncio.sleep(0.2)
                if await session.eval_js("!!document.querySelector('.mt-confirm-overlay')"):
                    raise RuntimeError("Manual trade modal still open after cancel")
                lbl = await session.eval_js(
                    "(() => { const b=document.getElementById('btn-mt-execute'); "
                    "return b ? b.textContent : ''; })()"
                )
                dis = await session.eval_js(
                    "(() => { const b=document.getElementById('btn-mt-execute'); "
                    "return b ? b.disabled : true; })()"
                )
                if dis:
                    raise RuntimeError("Manual trade execute button stayed disabled after cancel")
                if (lbl or "") != "ENGAGE TRADE":
                    raise RuntimeError(
                        "Manual trade execute button label wrong after cancel: " + str(lbl)
                    )

                await session.eval_js(
                    """
(async () => {
  const btn = document.getElementById('btn-mt-execute');
  btn.click();
})()
""",
                    await_promise=True,
                )
                await asyncio.sleep(0.3)
                await session.eval_js(
                    """
(() => {
  const ok = document.querySelector('.mt-confirm-ok');
  if (ok) ok.click();
})()
"""
                )
                await asyncio.sleep(2.0)
                if await session.eval_js("!!document.querySelector('.mt-confirm-overlay')"):
                    raise RuntimeError("Manual trade modal still open after confirm")
                lbl = await session.eval_js(
                    "(() => { const b=document.getElementById('btn-mt-execute'); "
                    "return b ? b.textContent : ''; })()"
                )
                dis = await session.eval_js(
                    "(() => { const b=document.getElementById('btn-mt-execute'); "
                    "return b ? b.disabled : true; })()"
                )
                if dis:
                    raise RuntimeError("Manual trade execute button stayed disabled after confirm")
                if (lbl or "") != "ENGAGE TRADE":
                    raise RuntimeError(
                        "Manual trade execute button did not reset to ENGAGE TRADE: " + str(lbl)
                    )

                inj = await session.eval_js(
                    "window.__titan_smoke && window.__titan_smoke.errs ? window.__titan_smoke.errs : []"
                )
                if inj:
                    raise RuntimeError("window errors/rejections: " + json.dumps(inj))

                if session.violations:
                    raise RuntimeError(
                        "CDP violations:\n" + "\n".join(session.violations)
                    )

            finally:
                pump.cancel()
                try:
                    await pump
                except asyncio.CancelledError:
                    pass
    finally:
        httpd.shutdown()
        if proc is not None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
        shutil.rmtree(user_data_dir, ignore_errors=True)


def main():
    try:
        asyncio.run(run_smoke())
    except RuntimeError as e:
        print(f"Static UI smoke FAILED: {e}", file=sys.stderr)
        print(
            "Fix the UI regression or set TITAN_SKIP_STATIC_UI_SMOKE=1 to skip intentionally.",
            file=sys.stderr,
        )
        sys.exit(1)
    print("Static UI smoke: OK")


main()
PY
    step_ok "Static UI smoke passed"
fi

# ─── Step 4: Rust tests (workspace) ──────────────────────────
step_banner 4 "cargo test --workspace..."
cd "${PROJECT}/src-tauri"
if ! cargo test --workspace 2>&1; then
    step_fail "cargo test FAILED — aborting deploy"
    exit 1
fi
step_ok "All workspace tests passed"

# ─── Step 5: Clippy (workspace, deny warnings) ─────────────
step_banner 5 "cargo clippy --workspace (deny warnings)..."
if ! cargo clippy --workspace -- -D warnings 2>&1; then
    step_fail "cargo clippy FAILED — aborting deploy"
    exit 1
fi
step_ok "Clippy clean (workspace)"

# ─── Step 6: Optional benchmarks (not default correctness gating) ─────────────
# Optional performance validation only. Default ./build.sh skips this; failures
# here abort the pipeline only when benchmarks are explicitly enabled via env.
if [[ "${TITAN_RUN_FULL_BENCH:-}" == "1" ]]; then
    step_banner 6 "cargo bench (full suite, TITAN_RUN_FULL_BENCH=1)..."
    if ! cargo bench 2>&1; then
        step_fail "cargo bench FAILED — aborting deploy"
        exit 1
    fi
    step_ok "Full benchmark suite completed"
elif [[ "${TITAN_RUN_BENCH:-}" == "1" ]]; then
    step_banner 6 "cargo bench --bench bench_check_entry (TITAN_RUN_BENCH=1)..."
    if ! cargo bench --bench bench_check_entry 2>&1; then
        step_fail "cargo bench FAILED — aborting deploy"
        exit 1
    fi
    step_ok "Focused benchmark (bench_check_entry) completed"
else
    echo -e "${YELLOW}[6/${TOTAL_STEPS}] Skipping optional benchmarks (set TITAN_RUN_BENCH=1 for focused bench, or TITAN_RUN_FULL_BENCH=1 for full suite)${NC}"
fi

# ─── Step 7: Sync src/ → dist/ (required before Tauri bundle) ─
step_banner 7 "Syncing src/ → dist/ (frontend bundle input)..."
mkdir -p "${PROJECT}/dist"
rsync -a --delete "${PROJECT}/src/" "${PROJECT}/dist/"
step_ok "dist/ synced"

# ─── Step 8: Release build ───────────────────────────────────
step_banner 8 "Building release binary (custom-protocol)..."
if ! cargo build --release --features custom-protocol 2>&1; then
    step_fail "cargo build FAILED"
    exit 1
fi
step_ok "Release build successful"

BINARY_SIZE=$(du -h "${PROJECT}/src-tauri/target/release/titan_hft_rust" | cut -f1)
echo -e "${GREEN}  Binary size: ${BINARY_SIZE}${NC}"

# ─── Step 9: Clear Webview cache ─────────────────────────────
step_banner 9 "Clearing Webview cache..."
rm -rf "${HOME}/.local/share/${APP_ID}" 2>/dev/null || true
rm -rf "${HOME}/.cache/${BINARY}" 2>/dev/null || true
step_ok "Webview cache cleared"

# ─── Step 10: Stop running instance ───────────────────────────
step_banner 10 "Stopping running instance..."
killall "${BINARY}" 2>/dev/null || true
sleep 1
step_ok "Stop signal sent (if running)"

# ─── Step 11: Deploy binary + desktop entries ───────────────
step_banner 11 "Deploying to ${INSTALL_PATH}..."
mkdir -p "$(dirname "${INSTALL_PATH}")"

if [ -f "${INSTALL_PATH}" ]; then
    cp "${INSTALL_PATH}" "${INSTALL_PATH}.bak" 2>/dev/null || true
fi
cp "${PROJECT}/src-tauri/target/release/titan_hft_rust" "${INSTALL_PATH}"
chmod +x "${INSTALL_PATH}"
step_ok "Deployed to ${INSTALL_PATH}"

if sudo cp "${INSTALL_PATH}" "${SYSTEM_INSTALL_PATH}"; then
    step_ok "Also deployed to ${SYSTEM_INSTALL_PATH}"
else
    echo -e "${YELLOW}⚠ Could not update ${SYSTEM_INSTALL_PATH} (sudo required).${NC}"
fi

mkdir -p "$(dirname "${DESKTOP_FILE}")"
cat > "${DESKTOP_FILE}" << DESKTOP_EOF
[Desktop Entry]
Categories=Finance;Development;
Comment=Titan HFT Bot — Sentient Hybrid Architecture
Exec=${INSTALL_PATH}
Path=${PROJECT}
Icon=${BINARY}
Name=Titan HFT (SHARK V7)
Terminal=false
Type=Application
StartupWMClass=${BINARY}
DESKTOP_EOF

chmod 644 "${DESKTOP_FILE}"
step_ok "Desktop shortcut updated → ${DESKTOP_FILE}"

if sudo cp "${DESKTOP_FILE}" "${SYSTEM_DESKTOP_FILE}"; then
    step_ok "System launcher synced → ${SYSTEM_DESKTOP_FILE}"
else
    echo -e "${YELLOW}⚠ Could not update ${SYSTEM_DESKTOP_FILE} (sudo required).${NC}"
fi

update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
if sudo update-desktop-database "/usr/share/applications" 2>/dev/null; then
    step_ok "Desktop database refreshed (system)"
fi

if command -v xdg-desktop-menu &>/dev/null; then
    xdg-desktop-menu forceupdate 2>/dev/null || true
fi

ICON_SRC="${PROJECT}/src-tauri/icons"
if [ -d "${ICON_SRC}" ]; then
    echo -e "${CYAN}  Installing icons...${NC}"
    for size_file in 32x32 128x128; do
        if [ -f "${ICON_SRC}/${size_file}.png" ]; then
            mkdir -p "${HOME}/.local/share/icons/hicolor/${size_file}/apps"
            cp "${ICON_SRC}/${size_file}.png" "${HOME}/.local/share/icons/hicolor/${size_file}/apps/${BINARY}.png"
            if sudo mkdir -p "/usr/share/icons/hicolor/${size_file}/apps" 2>/dev/null; then
                sudo cp "${ICON_SRC}/${size_file}.png" "/usr/share/icons/hicolor/${size_file}/apps/${BINARY}.png" 2>/dev/null || true
            fi
        fi
    done
    if [ -f "${ICON_SRC}/icon.png" ]; then
        mkdir -p "${HOME}/.local/share/icons/hicolor/512x512/apps"
        cp "${ICON_SRC}/icon.png" "${HOME}/.local/share/icons/hicolor/512x512/apps/${BINARY}.png"
        if sudo mkdir -p "/usr/share/icons/hicolor/512x512/apps" 2>/dev/null; then
            sudo cp "${ICON_SRC}/icon.png" "/usr/share/icons/hicolor/512x512/apps/${BINARY}.png" 2>/dev/null || true
        fi
    fi
    step_ok "Icons installed (32, 128, 512)"
fi

if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
    sudo gtk-update-icon-cache -f -t "/usr/share/icons/hicolor" 2>/dev/null || true
fi

# ─── Step 12: Launch ─────────────────────────────────────────
step_banner 12 "Launching ${BINARY}..."
mkdir -p "${LOG_DIR}"
nohup "${INSTALL_PATH}" >> "${LOG_FILE}" 2>&1 &
APP_PID=$!
disown

sleep 3
if kill -0 "$APP_PID" 2>/dev/null; then
    echo -e "${GREEN}✓ ${BINARY} launched (PID: ${APP_PID})${NC}"
    echo -e "${CYAN}  Logs: ${LOG_FILE}${NC}"
else
    step_fail "${BINARY} crashed on startup! Check logs: ${LOG_FILE}"
    tail -20 "${LOG_FILE}" 2>/dev/null
    exit 1
fi

# ─── Step 13: Success summary (final) ────────────────────────
BUILD_DURATION=$(( SECONDS - BUILD_START ))
step_banner 13 "Done"
echo ""
echo -e "${CYAN}╔══════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   VALIDATE · BUILD · DEPLOY · LAUNCH   ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════╝${NC}"
echo -e "${GREEN}Duration:${NC} ${BUILD_DURATION}s"
echo -e "${GREEN}Binary:${NC}   ${BINARY_SIZE}"
echo -e "${GREEN}End:${NC}      $(date '+%Y-%m-%d %H:%M:%S')"
echo ""
