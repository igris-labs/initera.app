import { defineConfig } from '@playwright/test';
import process from 'node:process';

const port = process.env.PM_TEST_PORT ?? '1420';
const baseURL = `http://127.0.0.1:${port}`;

export default defineConfig({
  testDir: './tests',
  workers: 1,
  use: {
    baseURL,
    browserName: 'chromium',
    viewport: { width: 1440, height: 900 },
  },
  webServer: {
    command: `pnpm dev --port ${port}`,
    url: baseURL,
    reuseExistingServer: true,
    timeout: 30_000,
  },
});
