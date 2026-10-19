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
// group is left; and it removes the profile. SIGINT, SIGTERM or SIGHUP to
// this script (Ctrl-C reaches it, not the browser's own group) run the same
// cleanup once and exit with 128 + the signal number. Only SIGKILL to this
// script, which nothing can handle, would leave the browser behind. It never
// signals any other process. Needs Node 22+ (WebSocket).
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

/** Exit codes for the signals that run the cleanup: 128 + signal number. */
const HANDLED_SIGNALS = [
  ["SIGHUP", 129],
  ["SIGINT", 130],
  ["SIGTERM", 143],
];

async function main(svg) {
  const browser =
    process.env.BROWSER_PATH ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
  let owned = null;
  let cleaning = null;
  // Idempotent: the normal path and any signal share one bounded cleanup.
  const cleanUp = () => {
    cleaning ??= (async () => {
      if (owned === null) return true;
      const gone = await stop(owned);
      rmSync(owned.profile, { recursive: true, force: true });
      if (!gone) {
        console.error(`accessibility_tree: process group ${owned.child.pid} outlived SIGKILL`);
      }
      return gone;
    })();
    return cleaning;
  };
  for (const [signal, code] of HANDLED_SIGNALS) {
    process.on(signal, () => {
      console.error(`accessibility_tree: ${signal}, stopping the browser`);
      cleanUp().finally(() => process.exit(code));
    });
  }
  owned = launch(browser);
  let status = 0;
  try {
    const nodes = await within(readTree(owned, svg), TIMEOUT_MS, "reading the accessibility tree");
    process.stdout.write(`${JSON.stringify({ svg, chrome: browser, nodes }, null, 2)}\n`);
  } catch (error) {
    console.error(`accessibility_tree: ${error.message}`);
    status = 1;
  } finally {
    if (!(await cleanUp())) status = 1;
  }
  return status;
}

/**
 * Runs this script against stand-ins that misbehave, with no real browser:
 * a "browser" whose group ignores SIGTERM and never opens a DevTools port,
 * and a browser path that does not exist. Each run must fail within its
 * deadline, leave no process of the stand-in's group and no profile. The
 * stand-in also runs while this script itself gets SIGTERM, and SIGINT as a
 * terminal's Ctrl-C sends it, with the same requirements and exit code
 * 128 + signal.
 */
async function selfTest() {
  const script = fileURLToPath(import.meta.url);
  const scratch = mkdtempSync(join(tmpdir(), "osl-a11y-self-test-"));
  const fake = join(scratch, "fake-browser.cjs");
  writeFileSync(
    fake,
    `#!${process.execPath}
process.on("SIGTERM", () => {});
const { spawn } = require("node:child_process");
const helper = spawn(process.execPath, ["-e", "process.on('SIGTERM', () => {}); setInterval(() => {}, 1000);"], { stdio: "ignore" });
require("node:fs").writeFileSync(process.env.OSL_FAKE_PIDS, process.pid + " " + helper.pid);
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
  // One run of this script; `signal`, when given, goes to this script's
  // process only, once the stand-in browser has started.
  const run = async (name, browserPath, { budgetMs, timeoutMs = 1500, signal, expect = 1 }) => {
    const temp = mkdtempSync(join(scratch, "tmp-"));
    const pids = join(temp, "pids");
    const started = Date.now();
    const child = spawn(process.execPath, [script, join(scratch, "missing.svg")], {
      env: {
        ...process.env,
        BROWSER_PATH: browserPath,
        TMPDIR: temp,
        OSL_A11Y_TIMEOUT_MS: String(timeoutMs),
        OSL_FAKE_PIDS: pids,
      },
      stdio: ["ignore", "ignore", "pipe"],
      detached: true,
    });
    let stderr = "";
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    const exited = new Promise((ok) => child.once("exit", (code, by) => ok(code ?? by)));
    const problems = [];
    if (signal) {
      const deadline = Date.now() + 10000;
      while (!existsSync(pids) && Date.now() < deadline) await sleep(50);
      if (existsSync(pids)) {
        await sleep(200);
        process.kill(child.pid, signal);
      } else {
        problems.push("the stand-in browser never started");
      }
    }
    const code = await within(exited, budgetMs, `${name} run`).catch((error) => {
      // The harness itself hung: stop its group (ours) and fail.
      try {
        process.kill(-child.pid, "SIGKILL");
      } catch {}
      return error.message;
    });
    const elapsed = Date.now() - started;
    if (code !== expect) problems.push(`exit ${code}, expected ${expect}`);
    const leftovers = readdirSync(temp).filter((entry) => entry.startsWith(PROFILE_PREFIX));
    if (leftovers.length) problems.push(`profile left: ${leftovers}`);
    if (existsSync(pids)) {
      for (const pid of readFileSync(pids, "utf8").split(" ").map(Number)) {
        if (alive(pid)) {
          problems.push(`stand-in process ${pid} still alive`);
          process.kill(pid, "SIGKILL");
        }
      }
    } else if (browserPath === fake) {
      problems.push("the stand-in browser never started");
    }
    return { name, elapsed, stderr: stderr.trim().replaceAll("\n", " | "), problems };
  };

  const results = [
    await run("browser ignoring SIGTERM", fake, { budgetMs: 20000 }),
    await run("missing browser", join(scratch, "no-such-browser"), { budgetMs: 10000 }),
    await run("SIGTERM to this script", fake, {
      budgetMs: 20000,
      timeoutMs: 60000,
      signal: "SIGTERM",
      expect: 143,
    }),
    await run("SIGINT (Ctrl-C) to this script", fake, {
      budgetMs: 20000,
      timeoutMs: 60000,
      signal: "SIGINT",
      expect: 130,
    }),
  ];
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
