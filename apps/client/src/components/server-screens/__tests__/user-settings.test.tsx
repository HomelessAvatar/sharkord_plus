import * as isDesktopAppModule from '@/helpers/is-desktop-app';
import { describe, expect, spyOn, test } from 'bun:test';
import { getUserSettingsEntries } from '../user-settings/helpers';

describe('UserSettings tab registration', () => {
  test('should include desktop entry when isDesktopApp returns true', () => {
    spyOn(isDesktopAppModule, 'isDesktopApp').mockReturnValue(true);

    const entries = getUserSettingsEntries((key) => key);
    const desktopEntry = entries.find((entry) => entry.id === 'desktop');

    expect(desktopEntry).toBeDefined();
    expect(desktopEntry?.id).toBe('desktop');
    expect(desktopEntry?.label).toBe('desktopTab');

    const ids = entries.map((entry) => entry.id);
    expect(ids).toContain('desktop');
    expect(ids.indexOf('desktop')).toBe(ids.indexOf('hotkeys') + 1);
  });

  test('should exclude desktop entry when isDesktopApp returns false', () => {
    spyOn(isDesktopAppModule, 'isDesktopApp').mockReturnValue(false);

    const entries = getUserSettingsEntries((key) => key);
    const desktopEntry = entries.find((entry) => entry.id === 'desktop');

    expect(desktopEntry).toBeUndefined();
    expect(entries.map((entry) => entry.id)).not.toContain('desktop');
  });

  test('should pass content through contentMap', () => {
    spyOn(isDesktopAppModule, 'isDesktopApp').mockReturnValue(true);

    const contentMap = {
      desktop: 'custom-desktop-node',
      profile: 'custom-profile-node'
    };

    const entries = getUserSettingsEntries((key) => key, contentMap);
    expect(entries.find((e) => e.id === 'desktop')?.content).toBe(
      'custom-desktop-node'
    );
    expect(entries.find((e) => e.id === 'profile')?.content).toBe(
      'custom-profile-node'
    );
  });
});
