const { chromium } = require("playwright-extra");
const StealthPlugin = require("puppeteer-extra-plugin-stealth");
const fs = require("fs");
const path = require("path");

chromium.use(StealthPlugin());

const args = process.argv.slice(2);
const contest = args[0];
const headless = args.includes("--headless");
const doLogin = args.includes("--login");
const watchMode = args.includes("--watch");
const leaderboardMode = args.includes("--leaderboard");
let intervalSecs = 15;
const i = args.indexOf("--interval");
if (i !== -1 && args[i + 1]) intervalSecs = parseInt(args[i + 1], 10) || 15;
const user = process.env.HKWATCH_USERNAME || "";
const pass = process.env.HKWATCH_PASSWORD || "";

if (!contest) {
  console.error("usage: node fetch.js <contest-slug> [--headless] [--login] [--watch] [--interval <secs>]");
  process.exit(2);
}

const BASE = `https://www.hackerrank.com/contests/${contest}`;

async function login(page, username, password) {
  await page.goto("https://www.hackerrank.com/auth/login", {
    waitUntil: "domcontentloaded",
    timeout: 60000,
  });
  await page.waitForTimeout(2500);

  for (const label of ["Accept All Cookies", "Accept All"]) {
    const btn = page.getByRole("button", { name: label }).first();
    if (await btn.isVisible().catch(() => false)) {
      await btn.click().catch(() => {});
      await page.waitForTimeout(500);
      break;
    }
  }

  await page.fill('input[name="username"]', username);
  await page.fill('input[name="password"]', password);

  const submit = page.locator('button[type="submit"]').first();
  if (await submit.isVisible().catch(() => false)) {
    await submit.click();
  } else {
    await page.getByRole("button", { name: /log in/i }).first().click();
  }

  await page
    .waitForURL((url) => !url.pathname.includes("/auth/login"), { timeout: 30000 })
    .catch(() => {});
  await page.waitForTimeout(2000);
  await page
    .goto("https://www.hackerrank.com/dashboard", { waitUntil: "domcontentloaded", timeout: 60000 })
    .catch(() => {});
  console.error(`[login] now at: ${page.url()} title="${await page.title()}"`);
}

async function downloadImages(ctx, slug, body_html) {
  const imageUrls = (() => {
    // resolve <img src> from the statement HTML
    const re = /<img[^>]+src=["']([^"']+)["']/gi;
    const out = [];
    let m;
    while ((m = re.exec(body_html)) !== null) {
      out.push(m[1]);
    }
    return out;
  })();

  const dir = path.join("challenges", slug);
  fs.mkdirSync(dir, { recursive: true });

  const saved = [];
  let idx = 0;
  for (let raw of imageUrls) {
    let url = raw;
    if (url.startsWith("//")) url = "https:" + url;
    else if (url.startsWith("/")) url = "https://www.hackerrank.com" + url;
    idx += 1;
    try {
      const resp = await ctx.request.get(url);
      if (!resp.ok()) {
        console.error(`[img ${slug}] HTTP ${resp.status()} for ${url}`);
        continue;
      }
      const ct = resp.headers()["content-type"] || "";
      const ext = ct.includes("png")
        ? "png"
        : ct.includes("jpeg") || ct.includes("jpg")
        ? "jpg"
        : ct.includes("gif")
        ? "gif"
        : "png";
      const file = path.join(dir, `statement_${idx}.${ext}`);
      fs.writeFileSync(file, await resp.body());
      saved.push(file);
      console.error(`[img ${slug}] saved ${file} (${url})`);
    } catch (e) {
      console.error(`[img ${slug}] failed ${url}: ${e.message}`);
    }
  }
  return saved;
}

async function fetchChallenges(page, ctx) {
  const list = await page.evaluate(async (c) => {
    const res = await fetch(`/rest/contests/${c}/challenges?offset=0&limit=100&track_login=true`, {
      credentials: "include",
    });
    const data = await res.json();
    return (data.models || []).map((m) => ({
      slug: m.slug,
      name: m.name,
      solved: !!m.solved,
    }));
  }, contest);

  const challenges = [];
  for (const ch of list) {
    const detail = await page.evaluate(
      async ({ c, slug }) => {
        const res = await fetch(`/rest/contests/${c}/challenges/${slug}`, { credentials: "include" });
        if (!res.ok) return null;
        const data = await res.json();
        const m = data.model || {};
        return {
          slug: m.slug,
          name: m.name,
          body_html: m.body_html || "",
          problem_statement: m.problem_statement || "",
          input_format: m.input_format || "",
          output_format: m.output_format || "",
          constraints: m.constraints || "",
          url: `https://www.hackerrank.com/contests/${c}/challenges/${m.slug}`,
          solved: !!m.solved,
        };
      },
      { c: contest, slug: ch.slug }
    );
    if (!detail) continue;
    detail.images = await downloadImages(ctx, detail.slug, detail.body_html);
    challenges.push(detail);
  }
  return challenges;
}

async function poll(page, ctx, pollNo) {
  const title = await page.title();
  if (title === "Access Denied") {
    console.error(`[poll ${pollNo}] Access Denied, skipping`);
    return;
  }
  const challenges = await fetchChallenges(page, ctx);
  let leaderboard = [];
  try {
    leaderboard = await page.evaluate(async (c) => {
      const res = await fetch(`/rest/contests/${c}/leaderboard?offset=0&limit=100&include_practice=true`, {
        credentials: "include",
      });
      if (!res.ok) return [];
      const d = await res.json();
      return (d.models || []).map((m) => ({
        hacker: m.hacker,
        rank: m.rank,
        score: m.score,
        time_taken: m.time_taken,
      }));
    }, contest);
  } catch (e) {
    console.error(`[poll ${pollNo}] leaderboard error: ${e.message}`);
  }
  const result = {
    contest,
    fetched_at: new Date().toISOString(),
    poll: pollNo,
    challenges,
    leaderboard,
  };
  console.log(JSON.stringify(result));
}

async function main() {
  const browser = await chromium.launch({
    headless,
    args: ["--disable-blink-features=AutomationControlled"],
  });
  const ctx = await browser.newContext({
    locale: "en-US",
    viewport: { width: 1440, height: 900 },
  });
  const page = await ctx.newPage();

  const cleanup = async () => {
    try { await browser.close(); } catch {}
    process.exit(0);
  };
  process.on("SIGINT", cleanup);
  process.on("SIGTERM", cleanup);

  if (doLogin) {
    if (!user || !pass) {
      throw new Error("HKWATCH_USERNAME and HKWATCH_PASSWORD env vars are required for --login");
    }
    await login(page, user, pass);
  }

  await page.goto(`${BASE}/challenges`, { waitUntil: "domcontentloaded", timeout: 60000 });
  await page.waitForTimeout(2000);

  if (leaderboardMode) {
    const models = [];
    let offset = 0;
    let total = 0;
    do {
      const part = await page.evaluate(
        async ({ c, off }) => {
          const res = await fetch(`/rest/contests/${c}/leaderboard?offset=${off}&limit=100&include_practice=true`, {
            credentials: "include",
          });
          if (!res.ok) return { total: 0, models: [], status: res.status };
          const d = await res.json();
          return { total: d.total || 0, models: d.models || [], status: res.status };
        },
        { c: contest, off: offset }
      );
      total = part.total;
      models.push(...part.models);
      offset += 100;
    } while (models.length < total && offset < 2000);
    console.log(
      JSON.stringify({ contest, fetched_at: new Date().toISOString(), leaderboard: models }, null, 2)
    );
    await browser.close();
    return;
  }

  if (!watchMode) {
    await poll(page, ctx, 0);
    await browser.close();
    return;
  }

  let pollNo = 1;
  let busy = false;
  let first = true;

  const tick = async () => {
    if (busy) return;
    busy = true;
    try {
      if (first) {
        first = false;
      } else {
        console.error(`[poll ${pollNo}] reloading ${BASE}/challenges`);
        await page.reload({ waitUntil: "domcontentloaded", timeout: 60000 });
        await page.waitForTimeout(2000);
      }
      await poll(page, ctx, pollNo);
      pollNo += 1;
    } catch (e) {
      console.error(`[poll] error: ${e.message}`);
    } finally {
      busy = false;
    }
  };

  await tick();
  setInterval(tick, intervalSecs * 1000);
  console.error(`[watch] polling every ${intervalSecs}s, browser stays open`);
}

main().catch((e) => {
  console.error("FETCH_ERROR: " + e.message);
  process.exit(1);
});
