import { expect, type Page } from '@playwright/test'

// =================
// === goToGraph ===
// =================

export async function goToGraph(page: Page) {
  await page.goto('/')
  expect(page.locator('.App')).toBeVisible()
}
