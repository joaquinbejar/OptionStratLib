// Reads the accessibility tree Chrome builds for an SVG file (#923), to
// check what the outlines SVG exposes: the chart's name and description
// from <title>/<desc>, and each label's name from its aria-label.
//
// Starts its own headless Chrome (BROWSER_PATH) in a process group of its
// own, with a fresh profile and no network, asks it over the DevTools
// protocol for the full tree (Accessibility.getFullAXTree) and prints the
// unignored nodes as JSON. Every step has a deadline. Whatever happens, it
// then stops that browser: Browser.close, then SIGTERM and finally SIGKILL
// to its process group, each with a bounded wait, until no process of the
// group is left; and it removes the profile. It never signals any other
// process. Needs Node 22+ (WebSocket).
//
// Usage: node accessibility_tree.mjs <svg> > tree.json
//        node accessibility_tree.mjs --self-test
//
// OSL_A11Y_TIMEOUT_MS bounds the whole read (default 30000); the shutdown
// that follows takes at most three more GRACE_MS waits.

import { spawn } from "node:child_process";
import {
  chmodSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { setTimeout as sleep } from "node:timers/promises";

const TIMEOUT_MS = Number(process.env.OSL_A11Y_TIMEOUT_MS ?? 30000);
const CALL_TIMEOUT_MS = 10000;
const GRACE_MS = 2000;
const PROFILE_PREFIX = "osl-a11y-";

/** Rejects after `ms` with `what` timed out, unless `promise` settles first. */
function within(promise, ms, what) {
  let timer;
  const expired = new Promise((_, reject) => {
    timer = setTimeout(() => reject(new Error(`${what} timed out after ${ms} ms`)), ms);
  });
  return Promise.race([promise, expired]).finally(() => clearTimeout(timer));
}

/** A DevTools connection whose calls time out and fail once it closes. */
function connect(url) {
  const socket = new WebSocket(url);
  let next = 0;
  let closed = null;
  const pending = new Map();
  const listeners = [];
  const failAll = (reason) => {
    closed = closed ?? reason;
    for (const { reject } of pending.values()) reject(closed);
    pending.clear();
  };
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
  socket.addEventListener("close", () => failAll(new Error("DevTools connection closed")));
  socket.addEventListener("error", () => failAll(new Error("DevTools connection failed")));
  const opened = new Promise((ok, reject) => {
    socket.addEventListener("open", ok, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  return {
    opened: within(opened, CALL_TIMEOUT_MS, "DevTools connection"),
    send(method, params = {}, sessionId) {
      if (closed) return Promise.reject(closed);
      next += 1;
      const id = next;
      const reply = new Promise((ok, reject) => pending.set(id, { resolve: ok, reject }));
      socket.send(JSON.stringify({ id, method, params, sessionId }));
      return within(reply, CALL_TIMEOUT_MS, method);
    },
    on(listener) {
      listeners.push(listener);
    },
    close() {
      failAll(new Error("DevTools connection closed"));
      socket.close();
    },
  };
}

/** The browser this script owns: its process group and its profile. */
function launch(browser) {
  const profile = mkdtempSync(join(tmpdir(), PROFILE_PREFIX));
  const child = spawn(
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
    // A group of its own, so the browser and every helper it starts can be
    // signalled together, and nothing else.
    { stdio: "ignore", detached: true },
  );
  const owned = { child, profile, exited: false, spawnError: null };
  child.once("exit", () => {
    owned.exited = true;
  });
  child.once("error", (error) => {
    owned.spawnError = error;
    if (child.pid === undefined) owned.exited = true;
  });
  return owned;
}

function groupAlive(pid) {
  try {
    process.kill(-pid, 0);
    return true;
  } catch {
    return false;
  }
}

/** Stops the owned browser's whole group, bounded; true once it is gone. */
async function stop(owned) {
  const { pid } = owned.child;
  if (pid === undefined) return true;
  for (const signal of [null, "SIGTERM", "SIGKILL"]) {
    if (signal) {
      try {
        process.kill(-pid, signal);
      } catch {
        // The group is already gone.
      }
    }
    const deadline = Date.now() + GRACE_MS;
    while (Date.now() < deadline) {
      if (owned.exited && !groupAlive(pid)) return true;
      await sleep(50);
    }
  }
  return owned.exited && !groupAlive(pid);
}

async function endpoint(owned) {
  const file = join(owned.profile, "DevToolsActivePort");
  for (;;) {
    if (owned.spawnError) throw new Error(`cannot start the browser: ${owned.spawnError.message}`);
    if (owned.exited) throw new Error("the browser exited before opening a DevTools port");
    if (existsSync(file)) {
      const [port, path] = readFileSync(file, "utf8").trim().split("\n");
      if (port && path) return `ws://127.0.0.1:${port}${path}`;
    }
    await sleep(100);
  }
}

async function readTree(owned, svg) {
  const cdp = connect(await endpoint(owned));
  try {
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
    await within(loaded, CALL_TIMEOUT_MS, "page load");
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
    await cdp.send("Browser.close").catch(() => {});
    return shown;
  } finally {
    cdp.close();
  }
}

async function main(svg) {
  const browser =
    process.env.BROWSER_PATH ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
  const owned = launch(browser);
  let status = 0;
  try {
    const nodes = await within(readTree(owned, svg), TIMEOUT_MS, "reading the accessibility tree");
    process.stdout.write(`${JSON.stringify({ svg, chrome: browser, nodes }, null, 2)}\n`);
  } catch (error) {
    console.error(`accessibility_tree: ${error.message}`);
    status = 1;
  } finally {
    if (!(await stop(owned))) {
      console.error(`accessibility_tree: process group ${owned.child.pid} outlived SIGKILL`);
      status = 1;
    }
    rmSync(owned.profile, { recursive: true, force: true });
  }
  return status;
}

/**
 * Runs this script against stand-ins that misbehave, with no real browser:
 * a "browser" whose group ignores SIGTERM and never opens a DevTools port,
 * and a browser path that does not exist. Each run must fail within its
 * deadline, leave no process of the stand-in's group and no profile.
 */
async function selfTest() {
  const script = fileURLToPath(import.meta.url);
  const scratch = mkdtempSync(join(tmpdir(), "osl-a11y-self-test-"));
  const fake = join(scratch, "fake-browser.cjs");
  const pids = join(scratch, "pids");
  writeFileSync(
    fake,
    `#!${process.execPath}
process.on("SIGTERM", () => {});
const { spawn } = require("node:child_process");
const helper = spawn(process.execPath, ["-e", "process.on('SIGTERM', () => {}); setInterval(() => {}, 1000);"], { stdio: "ignore" });
require("node:fs").writeFileSync(${JSON.stringify(pids)}, process.pid + " " + helper.pid);
setInterval(() => {}, 1000);
`,
  );
  chmodSync(fake, 0o755);
  const alive = (pid) => {
    try {
      process.kill(pid, 0);
      return true;
    } catch {
      return false;
    }
  };
  const run = async (name, browserPath, budgetMs) => {
    const temp = mkdtempSync(join(scratch, "tmp-"));
    const started = Date.now();
    const child = spawn(process.execPath, [script, join(scratch, "missing.svg")], {
      env: { ...process.env, BROWSER_PATH: browserPath, TMPDIR: temp, OSL_A11Y_TIMEOUT_MS: "1500" },
      stdio: ["ignore", "ignore", "pipe"],
      detached: true,
    });
    let stderr = "";
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    const code = await within(
      new Promise((ok) => child.once("exit", ok)),
      budgetMs,
      `${name} run`,
    ).catch((error) => {
      // The harness itself hung: stop its group (ours) and fail.
      try {
        process.kill(-child.pid, "SIGKILL");
      } catch {}
      return error.message;
    });
    const elapsed = Date.now() - started;
    const leftovers = readdirSync(temp).filter((entry) => entry.startsWith(PROFILE_PREFIX));
    const problems = [];
    if (code !== 1) problems.push(`exit ${code}, expected 1`);
    if (leftovers.length) problems.push(`profile left: ${leftovers}`);
    return { name, elapsed, stderr: stderr.trim(), problems };
  };

  const results = [];
  const ignoring = await run("browser ignoring SIGTERM", fake, 20000);
  if (existsSync(pids)) {
    const [browserPid, helperPid] = readFileSync(pids, "utf8").split(" ").map(Number);
    for (const pid of [browserPid, helperPid]) {
      if (alive(pid)) {
        ignoring.problems.push(`stand-in process ${pid} still alive`);
        process.kill(pid, "SIGKILL");
      }
    }
  } else {
    ignoring.problems.push("the stand-in browser never started");
  }
  results.push(ignoring);
  results.push(await run("missing browser", join(scratch, "no-such-browser"), 10000));
  rmSync(scratch, { recursive: true, force: true });

  let failed = 0;
  for (const { name, elapsed, stderr, problems } of results) {
    const verdict = problems.length ? `FAIL: ${problems.join("; ")}` : "ok";
    console.error(`self-test ${name}: ${verdict} (${elapsed} ms; ${stderr})`);
    failed += problems.length ? 1 : 0;
  }
  console.error(failed ? "self-test: FAILED" : "self-test: OK");
  return failed ? 1 : 0;
}

const argument = process.argv[2];
if (!argument) {
  console.error("usage: node accessibility_tree.mjs <svg> | --self-test");
  process.exit(2);
}
process.exit(argument === "--self-test" ? await selfTest() : await main(argument));
