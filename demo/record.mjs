// Record the demo's app scenes from the real, running app (no mock-ups). Chrome is driven by
// puppeteer-core and captured frame by frame (deterministic stop-motion at 30 fps), so every
// movement and every still moment lands in the video exactly as timed.
// Usage (app served at http://localhost:8765/app/):
//   TOOLS=<dir with node_modules/puppeteer-core> OUT=<dir> node demo/record.mjs
import { createRequire } from "node:module";
import { copyFileSync, mkdirSync, rmSync } from "node:fs";
import path from "node:path";

const require = createRequire(path.join(process.env.TOOLS, "package.json"));
const puppeteer = require("puppeteer-core");
const OUT = process.env.OUT;
const URL = "http://localhost:8765/app/#";
const FPS = 30;

const browser = await puppeteer.launch({
  executablePath: "C:/Program Files/Google/Chrome/Application/chrome.exe",
  // 1280x720 CSS px at 3x = 3840x2160 (4K) frames: the compositor zooms into regions without losing sharpness.
  defaultViewport: { width: 1280, height: 720, deviceScaleFactor: 3 },
  args: ["--hide-scrollbars", "--window-size=1280,720"],
});
const page = await browser.newPage();

let dir = "", n = 0, last = "";
/** Capture the current screen and hold it for `frames` frames. */
async function snap(frames = 1) {
  last = path.join(dir, `f${String(n).padStart(5, "0")}.jpg`);
  await page.screenshot({ path: last, type: "jpeg", quality: 95 });
  n++;
  for (let k = 1; k < frames; k++) copyFileSync(last, path.join(dir, `f${String(n++).padStart(5, "0")}.jpg`));
}
const hold = (seconds) => snap(Math.max(1, Math.round(seconds * FPS)));

/** A visible fingertip so viewers see where the astronaut touches. */
async function touchIndicator() {
  await page.evaluate(() => {
    const d = document.createElement("div");
    d.id = "finger";
    d.style.cssText = "position:fixed;z-index:9999;width:58px;height:58px;margin:-29px 0 0 -29px;border-radius:50%;border:4px solid rgba(255,255,255,.92);background:rgba(255,255,255,.2);pointer-events:none;left:-100px;top:-100px";
    document.body.append(d);
    addEventListener("pointermove", (e) => { d.style.left = `${e.clientX}px`; d.style.top = `${e.clientY}px`; }, true);
    addEventListener("pointerdown", () => { d.style.transform = "scale(.8)"; }, true);
    addEventListener("pointerup", () => { d.style.transform = "scale(1)"; }, true);
  });
}

async function open(hash) {
  await page.goto(URL + hash + "&guide=off", { waitUntil: "load" });
  await page.waitForFunction(() => document.documentElement.dataset.ready === "true", { timeout: 60000 });
  await page.reload({ waitUntil: "load" }); // hash routing applies on load
  await page.waitForFunction(() => document.documentElement.dataset.ready === "true", { timeout: 60000 });
  await touchIndicator();
}

async function rect(sel, i = 0) {
  return page.evaluate((s, k) => {
    const r = document.querySelectorAll(s)[k].getBoundingClientRect();
    return { x: r.left + r.width / 2, y: r.top + r.height / 2, l: r.left, w: r.width };
  }, sel, i);
}

async function moveTo(x, y, steps = 8) {
  const from = await page.evaluate(() => { const f = document.getElementById("finger"); return { x: parseFloat(f.style.left) || 640, y: parseFloat(f.style.top) || 600 }; });
  for (let k = 1; k <= steps; k++) {
    const t = k / steps, e = t * t * (3 - 2 * t);
    await page.mouse.move(from.x + (x - from.x) * e, from.y + (y - from.y) * e);
    await snap();
  }
}

/** Bring an element into the visible band above the replay bar, with an eased scroll. */
async function ensureVisible(sel, i = 0) {
  const r = await rect(sel, i);
  if (r.y > 150 && r.y < 470) return;
  const top = await page.evaluate(() => document.querySelector("main").scrollTop);
  await scrollTo(Math.max(0, top + r.y - 300), 10);
}

async function drag(sel, i, from, to) {
  await ensureVisible(sel, i);
  const t = await rect(sel, i);
  await moveTo(t.l + t.w * from, t.y, 7);
  await page.mouse.down();
  await snap(2);
  for (let k = 1; k <= 12; k++) {
    const e = k / 12;
    await page.mouse.move(t.l + t.w * (from + (to - from) * e), t.y);
    await snap();
  }
  await page.mouse.up();
  await snap(3);
}

async function scrollTo(top, steps = 14) {
  const start = await page.evaluate(() => document.querySelector("main").scrollTop);
  for (let k = 1; k <= steps; k++) {
    const t = k / steps, e = t * t * (3 - 2 * t);
    await page.evaluate((y) => document.querySelector("main").scrollTo(0, y), start + (top - start) * e);
    await snap();
  }
}

async function scene(name, fn) {
  dir = path.join(OUT, name);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  n = 0;
  await fn();
  console.log(`recorded ${name}: ${n} frames = ${(n / FPS).toFixed(2)} s`);
}

// S3: the evening check-in, rated first (fork the replay at day 125, then rate day 126).
await open("view=today&day=125&light=evening");
await page.evaluate(() => [...document.querySelectorAll("button")].find((b) => b.textContent === "Continue from this day").click());
await page.waitForFunction(() => document.querySelector("h1")?.textContent?.startsWith("Evening check"), { timeout: 60000 });
await touchIndicator();
await page.mouse.move(700, 600);
await scene("s3_checkin", async () => {
  await hold(0.3);
  await drag(".slider__track", 0, 0.5, 0.34);
  await drag(".slider__track", 1, 0.5, 0.41);
  await drag(".slider__track", 2, 0.5, 0.63);
  await ensureVisible("button.primary");
  const save = await rect("button.primary");
  await moveTo(save.x, save.y, 7);
  await page.mouse.down();
  await page.mouse.up();
  await new Promise((r) => setTimeout(r, 900)); // kernel B comparison, then the result
  await page.evaluate(() => document.querySelector("main").scrollTo(0, 0));
  await page.evaluate(() => { document.getElementById("finger").style.display = "none"; });
  await hold(1.4);
});

// S4: the mission as a time-lapse: the corridor grows, change is noticed, a new normal is set.
await open("view=course&day=20&light=evening");
await page.evaluate(() => document.querySelector("main").scrollTo(0, 265));
await scene("s4_course", async () => {
  for (let d = 20; d <= 150; d += 1) {
    await page.evaluate((day) => { const s = document.querySelector(".scrub"); s.value = String(day); s.dispatchEvent(new Event("input")); }, d);
    await page.evaluate(() => document.querySelector("main").scrollTo(0, 265));
    await snap();
  }
  await hold(1.0);
});

// S5: every note explains itself.
await open("view=today&day=110&light=evening&why=open");
await scene("s5_why", async () => {
  await hold(0.6);
  // Scroll the first note to the top of the view so its open "Why this note?" panel is on screen.
  const target = await page.evaluate(() => {
    const m = document.querySelector("main"), n = document.querySelector(".msg--caution, .msg--warning");
    return m.scrollTop + n.getBoundingClientRect().top - m.getBoundingClientRect().top - 90;
  });
  await scrollTo(target, 30);
  await hold(1.6);
});

// S6: the sharing log, with the safety notice re-routed to the CMO on board during loss of signal.
await open("view=sharing&day=150&light=evening");
await page.mouse.move(640, 640);
await scene("s6_sharing", async () => {
  await hold(0.4);
  await scrollTo(250, 22);
  const row = await page.evaluate(() => {
    const li = [...document.querySelectorAll(".log__row")].find((r) => r.textContent.includes("safety notice"));
    const b = li.getBoundingClientRect();
    return { x: b.right - 140, y: b.top + b.height / 2 };
  });
  await moveTo(row.x, row.y, 12);
  await hold(2.0);
});

await browser.close();
