import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test, expect } from '@playwright/test';

interface SmokePage {
  page_id: string;
  path: string;
  name: string;
  landmarks?: string[];
  skip_reason?: string;
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
    test.skip(Boolean(page.skip_reason), page.skip_reason ?? '');
    const unexpected: string[] = [];
    const serverErrors: string[] = [];

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
      if (status >= 500) {
        serverErrors.push(`${status} ${res.request().method()} ${url}`);
      }
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

    const skipWizard = browserPage.getByRole('button', { name: 'Skip Wizard', exact: true });
    if (await skipWizard.isVisible().catch(() => false)) {
      await skipWizard.click();
      const abort = browserPage.getByRole('button', { name: 'Abort wizard', exact: true });
      if (await abort.isVisible().catch(() => false)) {
        await abort.click();
      }
      const close = browserPage.getByRole('button', { name: 'Close', exact: true });
      if (await close.isVisible().catch(() => false)) {
        await close.click();
      }
    }
    const skipTour = browserPage.getByRole('button', { name: 'Skip tour', exact: true });
    if (await skipTour.isVisible().catch(() => false)) {
      await skipTour.click();
    }

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

    const landmarks = page.landmarks ?? [];
    if (landmarks.length > 0) {
      let loc = browserPage.getByText(landmarks[0], { exact: false });
      for (const text of landmarks.slice(1)) {
        loc = loc.or(browserPage.getByText(text, { exact: false }));
      }
      loc = loc.or(browserPage.locator('iframe'));
      await expect(loc.first(), `landmark ${landmarks.join('|')} (or iframe) on ${page.page_id}`).toBeVisible({
        timeout: 15_000,
      });
    }

    expect(serverErrors, `[${page.page_id}] non-benign HTTP 5xx`).toEqual([]);
    if (unexpected.length > 0) {
      console.log(`[${page.page_id}] unexpected 4xx/console (soft):`, [...new Set(unexpected)]);
    }
  });
}
