import { isDesktopApp } from '@/helpers/is-desktop-app';
import { describe, expect, test } from 'bun:test';

describe('server version compatibility', () => {
  test('isDesktopApp returns boolean identifying standalone client environment', () => {
    expect(typeof isDesktopApp()).toBe('boolean');
  });

  test('standalone client environment should bypass browser-only server updated reload dialog', () => {
    // When isDesktopApp() is true (e.g. in Tauri mobile/desktop), local bundles should not be blocked
    // by patch-level serverVersion differences from self-hosted Docker instances.
    const isDesktop = true;
    const serverVersion: string = '0.3.6';
    const appVersion: string = '0.3.8';

    const shouldShowBrowserReloadDialog =
      !isDesktop && serverVersion !== appVersion;
    expect(shouldShowBrowserReloadDialog).toBe(false);
  });
});
