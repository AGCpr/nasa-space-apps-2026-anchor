// Render ANCHOR's app icons (app/icons) to PNG with headless Chrome.
// Usage: TOOLS=<dir with node_modules/puppeteer-core> node app/scripts/icons.mjs app/icons
import { createRequire } from "node:module";
const require = createRequire(process.env.TOOLS + "/package.json");
const puppeteer = require("puppeteer-core");
const out = process.argv[2];
const mark = (rx, inset) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="${rx}" fill="#0d0904"/><g transform="translate(16 16) scale(${inset}) translate(-16 -16)"><path d="M5 21c5-8 8 2 11-5s7-3 11-8" stroke="#ffb858" stroke-width="3" fill="none" stroke-linecap="round"/></g></svg>`;
const b = await puppeteer.launch({ executablePath: "C:/Program Files/Google/Chrome/Application/chrome.exe" });
const p = await b.newPage();
// Home-screen icons are masked by the OS, so they are drawn full-bleed; maskable keeps the mark in the 80 % safe zone.
for (const [file, size, rx, inset] of [["icon-192.png", 192, 0, 1], ["icon-512.png", 512, 0, 1], ["icon-maskable-512.png", 512, 0, 0.72], ["apple-touch-icon.png", 180, 0, 0.9]]) {
  await p.setViewport({ width: size, height: size });
  await p.setContent(`<html><body style="margin:0;background:#0d0904">${mark(rx, inset).replace("<svg ", `<svg width="${size}" height="${size}" `)}</body></html>`);
  await p.screenshot({ path: `${out}/${file}`, omitBackground: false });
}
await b.close();
console.log("icons written");
