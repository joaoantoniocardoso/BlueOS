import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test, expect } from '@playwright/test';

interface SmokePage {
  page_id: string;
  path: string;
  name: string;
}

const pagesPath = join(__dirname, '..', 'pages.json');
const pages: SmokePage[] = JSON.parse(readFileSync(pagesPath, 'utf8'));

for (const page of pages) {
  test(`page loads: ${page.name} (${page.page_id})`, async ({ page: browserPage, baseURL }) => {
    const consoleErrors: string[] = [];
    browserPage.on('console', (msg) => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    const root = baseURL ?? process.env.BLUEOS_BASE ?? 'http://192.168.0.177';
    const rootResponse = await browserPage.goto(root, { waitUntil: 'domcontentloaded' });
    expect(rootResponse, `navigation to ${root}`).not.toBeNull();
    expect(rootResponse?.status(), `HTTP status for ${root}`).toBeLessThan(400);
    await expect(browserPage.locator('#app')).toBeVisible();

    // BlueOS nginx serves the SPA only at / (deep links 404); use client-side routing.
    await browserPage.evaluate((path) => {
      const root = document.querySelector('#app') as HTMLElement & { __vue__?: { $router: { push: (p: string) => unknown } } };
      const router = root?.__vue__?.$router;
      if (!router) {
        throw new Error('Vue router not available on #app');
      }
      void router.push(path);
    }, page.path);
    await browserPage.waitForURL(`**${page.path}**`);

    await expect(browserPage.locator('body')).toBeVisible();
    await expect(browserPage.locator('#app')).toBeVisible();

    if (consoleErrors.length > 0) {
      console.log(`[${page.page_id}] console errors (soft):`, consoleErrors);
    }
  });
}
