import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test, expect } from '@playwright/test';

interface SmokePage {
  page_id: string;
  path: string;
  name: string;
}

/**
 * Bench-Pi / default-install noise the BlueOS SPA emits on every page load.
 * These are not page-entry failures: missing optional userdata, unset bag keys,
 * MajorTom token absent, and mavlink2rest churn without a live FC stream.
 */
const BENIGN_FAILURE_PATTERNS: RegExp[] = [
  /\/bag\/v1\.0\/get\/(major_tom|vehicle\.(image_path|logo_image_path))(?:\?|$)/,
  /\/file-browser\/api\/rawsystem_root\/root\/\.majortom\/token\.key/,
  /\/userdata\/metadata_override\.json(?:\?|$)/,
  /\/userdata\/modeloverrides\/.+\.glb(?:\?|$)/,
  /\/mavlink2rest\/mavlink(?:\?|$)/,
  /\/mavlink2rest\/ws\/mavlink\?filter=/,
];

const pagesPath = join(__dirname, '..', 'pages.json');
const pages: SmokePage[] = JSON.parse(readFileSync(pagesPath, 'utf8'));

function isBenign(urlOrText: string): boolean {
  return BENIGN_FAILURE_PATTERNS.some((pattern) => pattern.test(urlOrText));
}

for (const page of pages) {
  test(`page loads: ${page.name} (${page.page_id})`, async ({ page: browserPage, baseURL }) => {
    const unexpected: string[] = [];

    browserPage.on('response', (res) => {
      const status = res.status();
      if (status < 400) {
        return;
      }
      const url = res.url();
      if (isBenign(url)) {
        return;
      }
      unexpected.push(`${status} ${res.request().method()} ${url}`);
    });

    browserPage.on('requestfailed', (req) => {
      const url = req.url();
      if (isBenign(url)) {
        return;
      }
      unexpected.push(`FAIL ${req.failure()?.errorText ?? '?'} ${req.method()} ${url}`);
    });

    browserPage.on('console', (msg) => {
      if (msg.type() !== 'error') {
        return;
      }
      const text = msg.text();
      // Chromium's generic "Failed to load resource" has no URL — covered by response listener.
      if (
        text.startsWith('Failed to load resource:') ||
        (text.includes('WebSocket connection') && isBenign(text))
      ) {
        return;
      }
      unexpected.push(`CONSOLE ${text}`);
    });

    const root = baseURL ?? process.env.BLUEOS_BASE ?? 'http://192.168.0.177';
    const rootResponse = await browserPage.goto(root, { waitUntil: 'domcontentloaded' });
    expect(rootResponse, `navigation to ${root}`).not.toBeNull();
    expect(rootResponse?.status(), `HTTP status for ${root}`).toBeLessThan(400);
    await expect(browserPage.locator('#app')).toBeVisible();

    // BlueOS nginx serves the SPA only at / (deep links 404); use client-side routing.
    await browserPage.evaluate((path) => {
      const rootEl = document.querySelector('#app') as HTMLElement & {
        __vue__?: { $router: { push: (p: string) => unknown } };
      };
      const router = rootEl?.__vue__?.$router;
      if (!router) {
        throw new Error('Vue router not available on #app');
      }
      void router.push(path);
    }, page.path);
    await browserPage.waitForURL(`**${page.path}**`);

    await expect(browserPage.locator('body')).toBeVisible();
    await expect(browserPage.locator('#app')).toBeVisible();

    // Brief settle so late bag/mavlink probes are classified, not missed as "unexpected".
    await browserPage.waitForTimeout(1500);

    if (unexpected.length > 0) {
      console.log(`[${page.page_id}] unexpected network/console errors (soft):`, [...new Set(unexpected)]);
    }
  });
}
