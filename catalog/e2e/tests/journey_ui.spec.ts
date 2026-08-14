import { readFileSync } from 'node:fs';
import { test, expect, type Page } from '@playwright/test';

interface SitlRcAction {
  op: 'sitl_rc';
  chan5: number;
  chan6: number;
  chan7: number;
  chan8: number;
}

type UiAction =
  | { op: 'open'; path: string }
  | { op: 'expect'; text: string }
  | { op: 'click'; text: string }
  | { op: 'click_if_visible'; text: string }
  | { op: 'wait_text'; text: string; timeout_ms: number }
  | { op: 'expect_iframe' }
  | SitlRcAction
  | { op: 'sleep'; ms: number };

interface UiJourneyPlan {
  journey_id: string;
  sitl_frame: string | null;
  actions: UiAction[];
}

const BENIGN_FAILURE_PATTERNS: RegExp[] = [
  /\/bag\/v1\.0\/get\/(major_tom|vehicle\.(image_path|logo_image_path)|wizard)(?:\?|$)/,
  /\/file-browser\/api\/rawsystem_root\/root\/\.majortom\/token\.key/,
  /\/userdata\/metadata_override\.json(?:\?|$)/,
  /\/userdata\/modeloverrides\/.+\.glb(?:\?|$)/,
  /\/mavlink2rest\/mavlink(?:\?|$)/,
  /\/mavlink2rest\/ws\/mavlink\?filter=/,
];

function isBenign(urlOrText: string): boolean {
  return BENIGN_FAILURE_PATTERNS.some((pattern) => pattern.test(urlOrText));
}

function loadPlans(): UiJourneyPlan[] {
  const path = process.env.UI_PLAN;
  if (!path) {
    throw new Error('UI_PLAN is required');
  }
  return JSON.parse(readFileSync(path, 'utf8')) as UiJourneyPlan[];
}

async function overlayKind(page: Page): Promise<'wizard' | 'tour' | null> {
  if (await page.getByRole('button', { name: 'Skip Wizard', exact: true }).isVisible().catch(() => false)) {
    return 'wizard';
  }
  if (await page.getByText('Skip Wizard', { exact: true }).isVisible().catch(() => false)) {
    return 'wizard';
  }
  if (await page.getByRole('button', { name: 'Skip tour', exact: true }).isVisible().catch(() => false)) {
    return 'tour';
  }
  if (await page.locator('.v-tour .v-step').isVisible().catch(() => false)) {
    return 'tour';
  }
  return null;
}

async function dismissOverlays(page: Page, waitMs = 15_000): Promise<void> {
  const deadline = Date.now() + waitMs;
  while (Date.now() < deadline) {
    const kind = await overlayKind(page);
    if (kind === 'wizard') {
      const skip = page.getByRole('button', { name: 'Skip Wizard', exact: true })
        .or(page.getByText('Skip Wizard', { exact: true }));
      await skip.first().click({ timeout: 10_000 });
      await page.getByRole('button', { name: 'Abort wizard', exact: true }).click({ timeout: 10_000 });
      await page.getByRole('button', { name: 'Close', exact: true }).click({ timeout: 10_000 });
      continue;
    }
    if (kind === 'tour') {
      const skipTour = page.getByRole('button', { name: 'Skip tour', exact: true });
      if (await skipTour.isVisible().catch(() => false)) {
        await skipTour.click();
      } else {
        await page.keyboard.press('Escape');
      }
      await page.waitForTimeout(300);
      continue;
    }
    return;
  }
  if (await overlayKind(page)) {
    throw new Error('first-boot overlay still open after dismiss');
  }
}

async function waitAndDismiss(page: Page, appearMs = 12_000): Promise<void> {
  await Promise.race([
    page.getByRole('button', { name: 'Skip tour', exact: true }).waitFor({ state: 'visible', timeout: appearMs }),
    page.getByRole('button', { name: 'Skip Wizard', exact: true }).waitFor({ state: 'visible', timeout: appearMs }),
    page.getByText('Skip Wizard', { exact: true }).waitFor({ state: 'visible', timeout: appearMs }),
  ]).catch(() => {});
  await dismissOverlays(page);
}

async function spaGoto(page: Page, path: string): Promise<void> {
  if (!page.url().startsWith('http')) {
    const base = process.env.BLUEOS_BASE ?? 'http://192.168.0.177';
    const bagWizard = page
      .waitForResponse((res) => res.url().includes('/bag/v1.0/get/wizard'), { timeout: 15_000 })
      .catch(() => null);
    const response = await page.goto(base, { waitUntil: 'domcontentloaded' });
    expect(response, `navigation to ${base}`).not.toBeNull();
    expect(response?.status(), `HTTP status for ${base}`).toBeLessThan(400);
    await bagWizard;
  }
  await expect(page.locator('#app')).toBeVisible({ timeout: 30_000 });
  await waitAndDismiss(page);
  if (path === '/') {
    return;
  }
  await page.evaluate((target) => {
    const rootEl = document.querySelector('#app') as HTMLElement & {
      __vue__?: { $router: { push: (p: string) => unknown } };
    };
    const router = rootEl?.__vue__?.$router;
    if (!router) {
      throw new Error('Vue router not available on #app');
    }
    void router.push(target);
  }, path);
  await page.waitForURL(`**${path}**`, { timeout: 30_000 });
  await expect(page.locator('#app')).toBeVisible();
  await dismissOverlays(page, 8_000);
}

async function postSitlRc(page: Page, rc: SitlRcAction): Promise<void> {
  const status = await page.evaluate(async (override) => {
    const headers = { 'Content-Type': 'application/json' };
    const post = (message: unknown) => fetch('/mavlink2rest/mavlink', {
      method: 'POST',
      headers,
      body: JSON.stringify({
        header: { system_id: 255, component_id: 1, sequence: 0 },
        message,
      }),
    });
    const servos = [override.chan5, override.chan6, override.chan7, override.chan8];
    let last = 0;
    for (let i = 0; i < servos.length; i += 1) {
      const response = await post({
        type: 'COMMAND_LONG',
        param1: i + 5,
        param2: servos[i],
        param3: 0,
        param4: 0,
        param5: 0,
        param6: 0,
        param7: 0,
        command: { type: 'MAV_CMD_DO_SET_SERVO' },
        target_system: 1,
        target_component: 1,
        confirmation: 0,
      });
      last = response.status;
    }
    await post({
      type: 'RC_CHANNELS_OVERRIDE',
      target_system: 1,
      target_component: 1,
      chan1_raw: 1500,
      chan2_raw: 1500,
      chan3_raw: 1500,
      chan4_raw: 1500,
      chan5_raw: override.chan5,
      chan6_raw: override.chan6,
      chan7_raw: override.chan7,
      chan8_raw: override.chan8,
    });
    return last;
  }, rc);
  expect(status, 'DO_SET_SERVO').toBeLessThan(400);
}

async function clickText(page: Page, text: string): Promise<void> {
  if (await overlayKind(page)) {
    await dismissOverlays(page, 10_000);
  }
  const inDialog = page.getByRole('dialog').getByRole('button', { name: text, exact: true });
  if (await inDialog.isVisible().catch(() => false)) {
    await inDialog.scrollIntoViewIfNeeded();
    await inDialog.click({ timeout: 30_000 });
    return;
  }
  const button = page.getByRole('button', { name: text, exact: true });
  if ((await button.count()) > 0) {
    await button.first().scrollIntoViewIfNeeded();
    await button.first().click({ timeout: 30_000 });
    return;
  }
  const byText = page.getByText(text, { exact: true }).last();
  await byText.scrollIntoViewIfNeeded();
  await byText.click({ timeout: 30_000 });
}

async function clickIfVisible(page: Page, text: string): Promise<void> {
  const button = page.getByRole('button', { name: text, exact: true });
  if (await button.first().isVisible().catch(() => false)) {
    await button.first().click({ timeout: 10_000 });
  }
}

async function waitCompassDone(page: Page, timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let last = 'no dialog';
  while (Date.now() < deadline) {
    const failed = page.getByText(/Calibration failed/i).first();
    if (await failed.isVisible().catch(() => false)) {
      throw new Error((await failed.innerText()).slice(0, 300));
    }
    if (await page.getByRole('button', { name: 'Dismiss', exact: true }).isVisible().catch(() => false)) {
      return;
    }
    if (await page.getByText('Calibration finished', { exact: true }).isVisible().catch(() => false)) {
      return;
    }
    const dialog = page.getByRole('dialog');
    const rows = await dialog.locator('tbody tr').count().catch(() => 0);
    const body = (await dialog.innerText().catch(() => '')).replace(/\s+/g, ' ');
    last = `rows=${rows} ${body.slice(0, 180)}`;
    if (rows >= 3 && /9\d%/.test(body)) {
      console.log(`compass reports ready (${last})`);
      return;
    }
    await page.waitForTimeout(1000);
  }
  throw new Error(`compass cal did not finish (${last})`);
}

async function waitText(page: Page, text: string, timeoutMs: number): Promise<void> {
  if (text === 'Dismiss') {
    await waitCompassDone(page, timeoutMs);
    return;
  }
  const wanted = page.getByText(text).first();
  const failed = page.getByText(/Calibration failed/i).first();
  if (/failed/i.test(text)) {
    await wanted.waitFor({ state: 'visible', timeout: timeoutMs });
    return;
  }
  const failedP = failed.waitFor({ state: 'visible', timeout: timeoutMs }).then(
    async () => ({ kind: 'fail' as const, text: (await failed.innerText()).slice(0, 300) }),
    () => ({ kind: 'none' as const }),
  );
  const wantedP = wanted.waitFor({ state: 'visible', timeout: timeoutMs }).then(
    () => ({ kind: 'ok' as const }),
    (err: unknown) => ({ kind: 'timeout' as const, err }),
  );
  const first = await Promise.race([wantedP, failedP]);
  if (first.kind === 'ok') {
    return;
  }
  if (first.kind === 'fail') {
    throw new Error(first.text);
  }
  const [wantedR, failedR] = await Promise.all([wantedP, failedP]);
  if (wantedR.kind === 'ok') {
    return;
  }
  if (failedR.kind === 'fail') {
    throw new Error(failedR.text);
  }
  throw wantedR.err;
}

function attitudeFromPwm(chan: number): number {
  return ((chan - 1500) / 500) * Math.PI;
}

function wrapPi(rad: number): number {
  let x = rad;
  while (x > Math.PI) x -= 2 * Math.PI;
  while (x < -Math.PI) x += 2 * Math.PI;
  return x;
}

function poseReached(
  att: { roll: number; pitch: number },
  wantRoll: number,
  wantPitch: number,
): boolean {
  if (Math.abs(wantPitch) > 2.8 || Math.abs(wantRoll) > 2.8) {
    return Math.abs(Math.abs(att.pitch) - Math.PI) < 0.35
      || Math.abs(Math.abs(att.roll) - Math.PI) < 0.35;
  }
  if (Math.abs(wantPitch) > 1.2) {
    return Math.abs(Math.abs(att.pitch) - Math.abs(wantPitch)) < 0.3;
  }
  if (Math.abs(wantRoll) > 1.2) {
    return Math.abs(Math.abs(att.roll) - Math.abs(wantRoll)) < 0.3;
  }
  return Math.abs(wrapPi(att.roll - wantRoll)) < 0.2
    && Math.abs(wrapPi(att.pitch - wantPitch)) < 0.2;
}

async function fetchAttitude(page: Page): Promise<{
  roll: number;
  pitch: number;
  rollspeed: number;
  pitchspeed: number;
  yawspeed: number;
} | null> {
  return page.evaluate(async () => {
    const response = await fetch('/mavlink2rest/v1/mavlink/vehicles/1/components/1/messages/ATTITUDE');
    const body = await response.json();
    const message = body?.message;
    if (!message) {
      return null;
    }
    return {
      roll: message.roll,
      pitch: message.pitch,
      rollspeed: message.rollspeed,
      pitchspeed: message.pitchspeed,
      yawspeed: message.yawspeed,
    };
  });
}

async function waitSitlPose(page: Page, rc: SitlRcAction, timeoutMs = 25_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let last = 'no ATTITUDE';
  if (rc.chan5 >= 1200 && rc.chan5 < 1300) {
    while (Date.now() < deadline) {
      const servo5 = await page.evaluate(async () => {
        const response = await fetch('/mavlink2rest/v1/mavlink/vehicles/1/components/1/messages/SERVO_OUTPUT_RAW');
        const body = await response.json();
        return body?.message?.servo5_raw ?? null;
      });
      last = `SERVO5=${servo5}`;
      if (typeof servo5 === 'number' && servo5 >= 1200 && servo5 < 1300) {
        console.log(`SERVO5=${servo5} mag-dance`);
        return;
      }
      await page.waitForTimeout(250);
    }
    throw new Error(`SITL mag-dance PWM not applied (${last} want 1200-1299)`);
  }
  if (rc.chan5 >= 1300) {
    return;
  }
  const wantRoll = attitudeFromPwm(rc.chan6);
  const wantPitch = attitudeFromPwm(rc.chan7);
  const holdAttitude = rc.chan5 >= 1100 && rc.chan5 < 1200;
  while (Date.now() < deadline) {
    const att = await fetchAttitude(page);
    if (att) {
      const moving = Math.abs(att.rollspeed) > 0.08
        || Math.abs(att.pitchspeed) > 0.08
        || Math.abs(att.yawspeed) > 0.08;
      const oriented = !holdAttitude || poseReached(att, wantRoll, wantPitch);
      last = `roll=${att.roll.toFixed(2)} pitch=${att.pitch.toFixed(2)} moving=${moving}`;
      if (oriented && !moving) {
        return;
      }
    }
    await page.waitForTimeout(250);
  }
  throw new Error(`SITL pose not settled (${last} want roll=${wantRoll.toFixed(2)} pitch=${wantPitch.toFixed(2)})`);
}

async function runPlan(page: Page, plan: UiJourneyPlan): Promise<void> {
  let holdTimer: ReturnType<typeof setInterval> | null = null;

  const holdRc = async (rc: SitlRcAction | null): Promise<void> => {
    if (holdTimer) {
      clearInterval(holdTimer);
      holdTimer = null;
    }
    if (!rc) {
      return;
    }
    await postSitlRc(page, rc);
    const servo5 = await page.evaluate(async () => {
      const response = await fetch('/mavlink2rest/v1/mavlink/vehicles/1/components/1/messages/SERVO_OUTPUT_RAW');
      const body = await response.json();
      return body?.message?.servo5_raw ?? null;
    });
    console.log(`SERVO5=${servo5} want=${rc.chan5}`);
    holdTimer = setInterval(() => {
      void postSitlRc(page, rc).catch(() => {});
    }, 400);
    await waitSitlPose(page, rc);
  };

  try {
    for (const action of plan.actions) {
      switch (action.op) {
        case 'open':
          await spaGoto(page, action.path);
          break;
        case 'expect': {
          const deadline = Date.now() + 30_000;
          let found = false;
          while (Date.now() < deadline) {
            if (await page.getByText(action.text).first().isVisible().catch(() => false)) {
              found = true;
              break;
            }
            for (const frame of page.frames()) {
              if (await frame.getByText(action.text).first().isVisible().catch(() => false)) {
                found = true;
                break;
              }
            }
            if (found) {
              break;
            }
            await page.waitForTimeout(250);
          }
          expect(found, `expect "${action.text}" in page or iframe`).toBe(true);
          break;
        }
        case 'expect_iframe':
          await expect(page.locator('iframe').first()).toBeVisible({ timeout: 30_000 });
          break;
        case 'click':
          await clickText(page, action.text);
          break;
        case 'click_if_visible':
          await clickIfVisible(page, action.text);
          break;
        case 'wait_text':
          await waitText(page, action.text, action.timeout_ms);
          break;
        case 'sitl_rc':
          await holdRc(action);
          break;
        case 'sleep':
          await page.waitForTimeout(action.ms);
          break;
        default:
          throw new Error(`unknown action: ${JSON.stringify(action)}`);
      }
    }
  } finally {
    if (holdTimer) {
      clearInterval(holdTimer);
    }
  }
}

const plans: UiJourneyPlan[] = process.env.UI_PLAN ? loadPlans() : [];

if (plans.length === 0) {
  test('ui: skipped without UI_PLAN', () => {
    test.skip(true, 'UI_PLAN not set');
  });
}

for (const plan of plans) {
  test(`ui: ${plan.journey_id}`, async ({ page }) => {
    test.setTimeout(360_000);
    page.on('response', (res) => {
      if (res.status() >= 400 && !isBenign(res.url())) {
        console.log(`[${plan.journey_id}] HTTP ${res.status()} ${res.url()}`);
      }
    });
    try {
      await runPlan(page, plan);
      console.log(`UI_RESULT ${plan.journey_id} PASS`);
    } catch (err) {
      const snippet = await page.locator('body').innerText().catch(() => '');
      console.log(`UI_PAGE ${plan.journey_id} ${snippet.slice(0, 2500).replace(/\s+/g, ' ')}`);
      console.log(`UI_RESULT ${plan.journey_id} FAIL ${String(err).split('\n')[0]}`);
      throw err;
    }
  });
}
