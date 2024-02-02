import { expect, type Page } from '@playwright/test'
import * as customExpect from './customExpect'
import * as locate from './locate'

// =================
// === goToGraph ===
// =================

export async function goToGraph(page: Page) {
  await page.goto('/')
  expect(page.locator('.App')).toBeVisible()
  // Wait until nodes are loaded.
  customExpect.toExist(locate.graphNode(page))
}
