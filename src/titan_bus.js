const TitanBus = {
    _handlers: {},
    on(event, fn) {
        (this._handlers[event] ??= []).push(fn);
    },
    once(event, fn) {
        const wrapper = (data) => { this.off(event, wrapper); fn(data); };
        this.on(event, wrapper);
    },
    off(event, fn) {
        if (!this._handlers[event]) return;
        this._handlers[event] = this._handlers[event].filter(h => h !== fn);
    },
    emit(event, data) {
        (this._handlers[event] ?? []).forEach(fn => {
            try { fn(data); } catch(e) { TitanError.report(`bus:${event}`, e); }
        });
    }
};
window.TitanBus = TitanBus;
