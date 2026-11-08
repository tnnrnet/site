import { existsSync } from "node:fs";
import { devices, defineConfig } from "@playwright/test";

/**
 * End-to-end tests against a running site server. They don't start one:
 * `bin/verify e2e` starts the built server on a spare port and sets BASE_URL
 * and POST_PATH; by hand, point BASE_URL at any server (default: the dev
 * server) and POST_PATH at one of its posts.
 *
 *   BASE_URL=http://127.0.0.1:3000 POST_PATH=/posts/my-post npx playwright test
 */
const baseURL = process.env.BASE_URL ?? "http://127.0.0.1:3000";

/**
 * Claude Code cloud sessions preinstall one Chromium at /opt/pw-browsers for
 * the global `playwright` package. The pinned @playwright/test matches it
 * today (1.63), but the image can move ahead of package-lock.json, so launch
 * that binary directly when present (or whatever PLAYWRIGHT_CHROMIUM_PATH
 * names). Elsewhere Playwright uses its own browser from
 * `npx playwright install chromium`.
 */
const cloudChromium = "/opt/pw-browsers/chromium";
const executablePath =
  process.env.PLAYWRIGHT_CHROMIUM_PATH ||
  (existsSync(cloudChromium) ? cloudChromium : undefined);

export default defineConfig({
  testDir: "./tests",
  timeout: 30 * 1000,
  expect: { timeout: 5000 },
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  use: {
    baseURL,
    trace: "retain-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        launchOptions: executablePath ? { executablePath } : {},
      },
    },
  ],
});
