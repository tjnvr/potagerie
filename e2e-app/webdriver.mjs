// A small W3C WebDriver client, enough for the smoke tests of the real app
// (tauri-driver on port 4444). No dependency: plain fetch.

const ELEMENT = "element-6066-11e4-a52f-4f94d47db1c9";

export class WebDriverError extends Error {}

async function request(url, method, body) {
  const response = await fetch(url, {
    method,
    headers: { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const json = await response.json().catch(() => ({}));
  if (!response.ok || json.value?.error) {
    const v = json.value ?? {};
    throw new WebDriverError(`${method} ${url}: ${v.error ?? response.status} ${v.message ?? ""}`.trim());
  }
  return json.value;
}

/** Polls `fn` until it returns a truthy value (or stops throwing), for at most `timeout` ms. */
export async function until(fn, what, timeout = 30_000) {
  const end = Date.now() + timeout;
  let last;
  for (;;) {
    try {
      const value = await fn();
      if (value) return value;
    } catch (e) {
      last = e;
    }
    if (Date.now() > end) throw new Error(`Timed out waiting for ${what}${last ? `: ${last.message}` : ""}`);
    await new Promise((r) => setTimeout(r, 250));
  }
}

export class Session {
  constructor(base, id) {
    this.base = base;
    this.url = `${base}/session/${id}`;
  }

  /** Launches `application` through tauri-driver: one session is one run of the app. */
  static async start(base, application) {
    const value = await request(`${base}/session`, "POST", {
      capabilities: { alwaysMatch: { browserName: "wry", "tauri:options": { application } } },
    });
    return new Session(base, value.sessionId);
  }

  /** Closes the app. */
  async quit() {
    await request(this.url, "DELETE");
  }

  cmd(method, path, body) {
    return request(`${this.url}${path}`, method, body);
  }

  async findAll(xpath) {
    const found = await this.cmd("POST", "/elements", { using: "xpath", value: xpath });
    return found.map((e) => e[ELEMENT]);
  }

  /** Waits for the first element matching `xpath`. */
  async find(xpath, timeout) {
    return until(async () => (await this.findAll(xpath))[0], xpath, timeout);
  }

  async click(xpath) {
    const id = await this.find(xpath);
    await until(() => this.cmd("POST", `/element/${id}/click`, {}).then(() => true), `click on ${xpath}`, 10_000);
  }

  text(id) {
    return this.cmd("GET", `/element/${id}/text`);
  }

  /** Waits until the page shows `text`. */
  async waitText(text, timeout) {
    await until(() => this.execute("return document.body.innerText.includes(arguments[0])", [text]), `"${text}"`, timeout);
  }

  execute(script, args = []) {
    return this.cmd("POST", "/execute/sync", { script, args });
  }

  /** A real mouse click at the centre of an element, even under another one. */
  async clickAt(id) {
    await this.cmd("POST", "/actions", {
      actions: [
        {
          type: "pointer",
          id: "mouse",
          parameters: { pointerType: "mouse" },
          actions: [
            { type: "pointerMove", duration: 0, origin: { [ELEMENT]: id }, x: 0, y: 0 },
            { type: "pointerDown", button: 0 },
            { type: "pointerUp", button: 0 },
          ],
        },
      ],
    });
    await this.cmd("DELETE", "/actions");
  }

  /** Types one key (WebDriver code, e.g. "" for Enter) on the focused element. */
  async key(value) {
    await this.cmd("POST", "/actions", {
      actions: [{ type: "key", id: "keyboard", actions: [{ type: "keyDown", value }, { type: "keyUp", value }] }],
    });
    await this.cmd("DELETE", "/actions");
  }

  async screenshot() {
    return Buffer.from(await this.cmd("GET", "/screenshot"), "base64");
  }

  source() {
    return this.cmd("GET", "/source");
  }
}
