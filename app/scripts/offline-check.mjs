// Check of the offline claim (N1): open the app once, cut the network, reload, and require that the
// app starts and replays the mission. Exit code 1 on failure.
// Usage: TOOLS=<dir with node_modules/puppeteer-core> node app/scripts/offline-check.mjs [url]
import { createRequire } from "node:module";
const require = createRequire(process.env.TOOLS + "/package.json");
const puppeteer = require("puppeteer-core");
const url = process.argv[2] ?? "http://localhost:8765/app/";
const b = await puppeteer.launch({ executablePath: "C:/Program Files/Google/Chrome/Application/chrome.exe" });
const p = await b.newPage();
await p.goto(url, { waitUntil: "load" });
await p.waitForFunction(() => document.documentElement.dataset.ready === "true", { timeout: 60000 });
await p.evaluate(() => navigator.serviceWorker.ready);
await p.reload({ waitUntil: "load" }); // first controlled load
await p.setOfflineMode(true);
await p.reload({ waitUntil: "load" });
const ok = await p.waitForFunction(() => document.documentElement.dataset.ready === "true", { timeout: 60000 }).then(() => true, () => false);
const day = await p.evaluate(() => document.querySelector(".pill .num")?.textContent ?? "");
await b.close();
console.log(ok ? `offline: app started with no network (${day.trim()})` : "offline: FAILED to start without network");
process.exit(ok ? 0 : 1);
