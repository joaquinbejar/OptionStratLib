// Reads the accessibility tree Chrome builds for an SVG file (#923), to
// check what the outlines SVG exposes: the chart's name and description
// from <title>/<desc>, and each label's name from its aria-label.
//
// Starts its own headless Chrome (BROWSER_PATH) with a fresh profile and no
// network, asks it over the DevTools protocol for the full tree
// (Accessibility.getFullAXTree), prints the unignored nodes as JSON, then
// closes that browser and removes the profile. Needs Node 22+ (WebSocket).
//
// Usage: node accessibility_tree.mjs <svg> > tree.json

import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { setTimeout as sleep } from "node:timers/promises";

const svg = process.argv[2];
if (!svg) {
  console.error("usage: node accessibility_tree.mjs <svg>");
  process.exit(2);
}
const browser =
  process.env.BROWSER_PATH ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const profile = mkdtempSync(join(tmpdir(), "osl-a11y-"));
const chrome = spawn(
  browser,
  [
    "--headless=new",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-sync",
    "--force-renderer-accessibility",
    "--host-resolver-rules=MAP * ~NOTFOUND",
    "--proxy-server=127.0.0.1:9",
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    "about:blank",
  ],
  { stdio: "ignore" },
);
const exited = new Promise((done) => chrome.once("exit", done));

async function endpoint() {
  const file = join(profile, "DevToolsActivePort");
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (existsSync(file)) {
      const [port, path] = readFileSync(file, "utf8").trim().split("\n");
      if (port && path) return `ws://127.0.0.1:${port}${path}`;
    }
    await sleep(100);
  }
  throw new Error("Chrome did not open a DevTools port");
}

function connect(url) {
  const socket = new WebSocket(url);
  let next = 0;
  const pending = new Map();
  const listeners = [];
  socket.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    if (message.id !== undefined && pending.has(message.id)) {
      const { resolve: ok, reject } = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) reject(new Error(JSON.stringify(message.error)));
      else ok(message.result);
    } else {
      for (const listener of listeners) listener(message);
    }
  });
  const opened = new Promise((ok, reject) => {
    socket.addEventListener("open", ok, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  return {
    opened,
    send(method, params = {}, sessionId) {
      next += 1;
      const id = next;
      socket.send(JSON.stringify({ id, method, params, sessionId }));
      return new Promise((ok, reject) => pending.set(id, { resolve: ok, reject }));
    },
    on(listener) {
      listeners.push(listener);
    },
    close() {
      socket.close();
    },
  };
}

let status = 0;
try {
  const cdp = connect(await endpoint());
  await cdp.opened;
  const { targetId } = await cdp.send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await cdp.send("Target.attachToTarget", { targetId, flatten: true });
  await cdp.send("Page.enable", {}, sessionId);
  const loaded = new Promise((ok) =>
    cdp.on((message) => {
      if (message.sessionId === sessionId && message.method === "Page.loadEventFired") ok();
    }),
  );
  await cdp.send("Page.navigate", { url: pathToFileURL(resolve(svg)).href }, sessionId);
  await loaded;
  await cdp.send("Accessibility.enable", {}, sessionId);
  const { nodes } = await cdp.send("Accessibility.getFullAXTree", {}, sessionId);
  const value = (property) => property?.value ?? null;
  const shown = nodes
    .filter((node) => !node.ignored)
    .map((node) => ({
      role: value(node.role),
      name: value(node.name),
      description: value(node.description),
    }));
  process.stdout.write(`${JSON.stringify({ svg, chrome: browser, nodes: shown }, null, 2)}\n`);
  await cdp.send("Browser.close").catch(() => {});
  cdp.close();
} catch (error) {
  console.error(`accessibility_tree: ${error.message}`);
  status = 1;
} finally {
  // Only the browser this script started, and only if it outlived
  // Browser.close.
  const closed = await Promise.race([exited.then(() => true), sleep(5000).then(() => false)]);
  if (!closed) {
    chrome.kill("SIGTERM");
    await exited;
  }
  rmSync(profile, { recursive: true, force: true });
}
process.exit(status);
