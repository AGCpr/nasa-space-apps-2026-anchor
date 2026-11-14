export function h(tag, props = {}, ...children) {
    const el = document.createElement(tag);
    for (const [k, v] of Object.entries(props)) {
        if (v === null || v === undefined || v === false)
            continue;
        if (typeof v === "function")
            el.addEventListener(k.slice(2), v);
        else if (v === true)
            el.setAttribute(k, "");
        else
            el.setAttribute(k, String(v));
    }
    for (const c of children)
        if (c !== null && c !== undefined && c !== false)
            el.append(c);
    return el;
}
const NS = "http://www.w3.org/2000/svg";
export function s(tag, attrs = {}, ...children) {
    const el = document.createElementNS(NS, tag);
    for (const [k, v] of Object.entries(attrs))
        el.setAttribute(k, String(v));
    for (const c of children)
        el.append(c);
    return el;
}
/** Register glyphs: shape and label carry the meaning, colour only repeats it (R2). */
const GLYPH = {
    warning: { label: "Safety line", path: "M12 2 22 12 12 22 2 12Z" },
    caution: { label: "Worth a look", path: "M12 3 22 21H2Z" },
    advisory: { label: "Note", path: "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18Z" },
    info: { label: "Update", path: "M4 11h16v2H4Z" },
};
export function glyph(register) {
    return s("svg", { viewBox: "0 0 24 24", width: 28, height: 28, "aria-hidden": "true", class: `glyph glyph--${register}` }, s("path", { d: GLYPH[register].path }));
}
export function card(m, extra) {
    return h("article", { class: `msg msg--${m.register}` }, 
    // Register inline on the heading line, never a label stacked above it; warning and caution
    // keep their word beside the shape (R2).
    h("header", { class: "msg__head" }, glyph(m.register), h("h2", { class: "msg__title" }, m.register === "warning" || m.register === "caution" ? h("span", { class: "msg__reg" }, `${GLYPH[m.register].label}: `) : null, m.title)), h("p", {}, m.text), h("p", { class: "msg__step" }, h("strong", {}, m.owner === "You" ? "What you can do: " : `Who acts: ${m.owner}. `), `${m.action[0].toUpperCase()}${m.action.slice(1)}.`), m.why
        ? h("details", { class: "why" }, h("summary", {}, "Why this note?"), h("ul", {}, ...m.why.lines.map((l) => h("li", {}, l))), m.why.area ? h("p", { class: "why__area" }, "NASA research area: ", h("a", { href: m.why.area.url, target: "_blank", rel: "noopener" }, m.why.area.name), ".") : null)
        : null, extra ?? null);
}
export function segmented(name, options, current, onPick, cls = "") {
    return h("div", { class: `seg ${cls}`, role: "radiogroup", "aria-label": name }, ...options.map((o) => h("button", { type: "button", role: "radio", "aria-checked": o.value === current ? "true" : "false", onclick: () => onPick(o.value) }, o.label)));
}
/**
 * Pop-up button (HIG pop-up-buttons.md): a flat list of mutually exclusive options, used where a
 * segmented control would exceed HIG's "about five to seven segments". Native select, so the
 * platform supplies the menu, keyboard and screen-reader behaviour.
 */
export function popUp(name, groups, current, onPick) {
    return h("label", { class: "popup" }, h("span", { class: "popup__label" }, name), h("select", { class: "popup__select", onchange: (e) => onPick(e.target.value) }, ...groups.filter((g) => g.options.length).map((g) => h("optgroup", { label: g.label }, ...g.options.map((o) => h("option", { value: o.value, selected: o.value === current }, o.label))))));
}
/** Stepper for a measured value: large minus and plus targets around the value (R7), no typing. */
export function stepper(opts) {
    const show = (v) => `${v.toFixed(opts.decimals)} ${opts.unit}`;
    return h("div", { class: "stepper", role: "group", "aria-labelledby": `${opts.id}-l` }, h("span", { id: `${opts.id}-l`, class: "stepper__label" }, opts.label), h("button", { type: "button", class: "stepper__btn", "aria-label": `Lower ${opts.label}`, disabled: opts.disabled, onclick: () => opts.onChange(opts.value - opts.step) }, "−"), h("output", { class: "stepper__value", "aria-live": "polite" }, show(opts.value)), h("button", { type: "button", class: "stepper__btn", "aria-label": `Raise ${opts.label}`, disabled: opts.disabled, onclick: () => opts.onChange(opts.value + opts.step) }, "+"));
}
/**
 * Bipolar slider (R9, R15): the whole track takes the press, the value follows the finger and
 * is committed on release. Untouched, it shows "not rated" rather than a default of 50.
 */
export function bipolar(opts) {
    let value = opts.value;
    const fill = h("div", { class: "slider__fill" });
    const thumb = h("div", { class: "slider__thumb" });
    const readout = h("span", { class: "slider__readout" });
    const track = h("div", {
        class: "slider__track",
        role: "slider",
        tabindex: opts.disabled ? -1 : 0,
        "aria-labelledby": `${opts.id}-label`,
        "aria-valuemin": 0,
        "aria-valuemax": 1000,
        "aria-disabled": opts.disabled ? "true" : null,
    }, h("div", { class: "slider__mid", "aria-hidden": "true" }), fill, thumb);
    const paint = () => {
        const rated = value !== null;
        track.dataset.rated = String(rated);
        if (rated) {
            const p = value / 10;
            const [a, b] = p < 50 ? [p, 50] : [50, p];
            fill.style.left = `${a}%`;
            fill.style.width = `${b - a}%`;
            thumb.style.left = `${p}%`;
            track.setAttribute("aria-valuenow", String(value));
            const word = p < 40 ? opts.low : p > 60 ? opts.high : opts.mid;
            track.setAttribute("aria-valuetext", `${word}, ${Math.round(p)} of 100`);
            readout.textContent = word;
        }
        else {
            track.removeAttribute("aria-valuenow");
            track.setAttribute("aria-valuetext", opts.notRated);
            readout.textContent = opts.notRated;
        }
    };
    const at = (e) => {
        const r = track.getBoundingClientRect();
        return Math.round(Math.min(1, Math.max(0, (e.clientX - r.left) / r.width)) * 1000);
    };
    if (!opts.disabled) {
        track.addEventListener("pointerdown", (e) => {
            track.setPointerCapture(e.pointerId);
            value = at(e);
            paint();
        });
        track.addEventListener("pointermove", (e) => {
            if (!track.hasPointerCapture(e.pointerId))
                return;
            value = at(e);
            paint();
        });
        track.addEventListener("pointerup", (e) => {
            if (!track.hasPointerCapture(e.pointerId))
                return;
            track.releasePointerCapture(e.pointerId);
            opts.onCommit(value);
        });
        track.addEventListener("keydown", (e) => {
            const step = { ArrowLeft: -10, ArrowDown: -10, ArrowRight: 10, ArrowUp: 10, PageDown: -100, PageUp: 100 }[e.key];
            if (step === undefined)
                return;
            e.preventDefault();
            value = Math.min(1000, Math.max(0, (value ?? 500) + step));
            paint();
            opts.onCommit(value);
        });
    }
    paint();
    return h("div", { class: "slider" }, h("div", { class: "slider__top" }, h("span", { id: `${opts.id}-label`, class: "slider__label" }, opts.label), readout), track, h("div", { class: "slider__ends", "aria-hidden": "true" }, h("span", {}, opts.low), h("span", {}, opts.mid), h("span", {}, opts.high)));
}
