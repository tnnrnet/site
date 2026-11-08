import { test, expect, type Page } from "@playwright/test";

// A published post, e.g. "/posts/my-post". bin/verify passes the one with the
// longest slug, which is the hardest case for the breadcrumbs at narrow widths.
const POST = process.env.POST_PATH ?? "";
if (!POST.startsWith("/posts/")) {
  throw new Error("Set POST_PATH to a published post, e.g. POST_PATH=/posts/my-post");
}

/** Resolves once the WASM bundle has hydrated the page (see src/lib.rs). */
async function hydrated(page: Page) {
  await expect(page.locator("html")).toHaveAttribute("data-hydrated", "");
}

/** The body's background and the dark theme's page background, as rgb(). */
async function backgrounds(page: Page) {
  return page.evaluate(() => {
    const probe = document.createElement("div");
    probe.style.backgroundColor = "var(--neutral-900)"; // dark --bg-page
    document.body.append(probe);
    const dark = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return { body: getComputedStyle(document.body).backgroundColor, dark };
  });
}

test.describe("pre-paint theme script, before hydration", () => {
  test.use({ colorScheme: "light" });

  test("applies a saved dark theme with the WASM bundle blocked", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("theme", "dark"));
    let blocked = 0;
    await page.route("**/*.wasm", (route) => {
      blocked++;
      return route.abort();
    });

    await page.goto("/");

    const html = page.locator("html");
    await expect(html).toHaveClass(/(^|\s)theme-dark(\s|$)/);
    await expect(html).not.toHaveClass(/theme-system/);
    // The page did ask for the bundle, and never hydrated: the class came
    // from the inline script in shell(), not from SiteSettings::apply_theme.
    await expect.poll(() => blocked).toBeGreaterThan(0);
    await expect(html).not.toHaveAttribute("data-hydrated", "");
    const { body, dark } = await backgrounds(page);
    expect(body).toBe(dark);
  });
});

test.describe("without JavaScript, OS dark", () => {
  test.use({ javaScriptEnabled: false, colorScheme: "dark" });

  test("follows the OS color scheme", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("html")).toHaveClass(/(^|\s)theme-system(\s|$)/);
    const { body, dark } = await backgrounds(page);
    expect(body).toBe(dark);
  });

  test("tuning page shows System selected", async ({ page }) => {
    await page.goto("/tuning");
    await expect(page.locator("#theme-system")).toBeChecked();
    await expect(page.locator("#theme-light")).not.toBeChecked();
    await expect(page.locator("#theme-dark")).not.toBeChecked();
  });

  test("Tune control is a plain link to /tuning", async ({ page }) => {
    await page.goto(POST);
    const tune = page.getByRole("link", { name: "Tune" });
    await expect(tune).toHaveAttribute("href", "/tuning");
    expect(await tune.evaluate((el) => el.tagName)).toBe("A");
    await tune.click();
    await expect(page).toHaveURL("/tuning");
  });
});

test.describe("with JavaScript", () => {
  test("Dark switch on /tuning applies and saves the theme", async ({ page }) => {
    await page.goto("/tuning");
    await hydrated(page);

    await page.locator('label.vswitch-label[for="theme-dark"]').click();

    const html = page.locator("html");
    await expect(html).toHaveClass(/(^|\s)theme-dark(\s|$)/);
    await expect(html).not.toHaveClass(/theme-system/);
    await expect(page.locator("#theme-dark")).toBeChecked();
    expect(await page.evaluate(() => localStorage.getItem("theme"))).toBe("dark");
  });

  test("Tune on /tuning returns to the post it was opened from", async ({ page }) => {
    await page.goto(POST);
    await hydrated(page);

    const tune = page.getByRole("link", { name: "Tune" });
    await tune.click();
    await expect(page).toHaveURL("/tuning");
    // Without the client-side "back" behavior this link goes to "/".
    await tune.click();
    await expect(page).toHaveURL(POST);
  });
});

test.describe("routing", () => {
  for (const path of ["/no-such-page", "/posts/no-such-post"]) {
    test(`unknown slug ${path} is a 404`, async ({ page }) => {
      const response = await page.goto(path);
      expect(response?.status()).toBe(404);
      // Same title block as every other page, not a bare unstyled <main>
      await expect(page.locator("main .page-header h1")).toHaveText("404");
    });
  }
});

test.describe("narrow viewport", () => {
  test.use({ viewport: { width: 390, height: 844 } });

  for (const javaScriptEnabled of [true, false]) {
    test.describe(javaScriptEnabled ? "with JS" : "without JS", () => {
      test.use({ javaScriptEnabled });

      test(`no horizontal overflow on ${POST}`, async ({ page }) => {
        await page.goto(POST);
        if (javaScriptEnabled) await hydrated(page);
        const { scrollWidth, clientWidth } = await page.evaluate(() => ({
          scrollWidth: document.documentElement.scrollWidth,
          clientWidth: document.documentElement.clientWidth,
        }));
        expect(scrollWidth).toBeLessThanOrEqual(clientWidth);
      });
    });
  }
});
