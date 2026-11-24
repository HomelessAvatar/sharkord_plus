import { PluginSlotRenderer } from '@/components/plugin-slot-renderer';
import { useCan, useUserSettingsPlugins } from '@/features/server/hooks';
import { Permission, PluginSlot } from '@sharkord/shared';
import { type TSettingsEntry } from '@sharkord/ui';
import { Package } from 'lucide-react';
import { memo, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import type { TServerScreenBaseProps } from '../screens';
import { SettingsShell } from '../settings-shell';
import { DeleteAccount } from './delete-account';
import { DesktopSettings } from './desktop';
import { Devices } from './devices';
import { getUserSettingsEntries } from './helpers';
import { Hotkeys } from './hotkeys';
import { Notifications } from './notifications';
import { Others } from './others';
import { Password } from './password';
import { Profile } from './profile';
import { Sessions } from './sessions';

type TUserSettingsProps = TServerScreenBaseProps & { defaultTab?: string };

const UserSettings = memo(({ close, defaultTab }: TUserSettingsProps) => {
  const { t } = useTranslation('settings');
  const can = useCan();
  const userSettingsPlugins = useUserSettingsPlugins();

  const contentMap = useMemo(
    () => ({
      profile: <Profile />,
      devices: <Devices />,
      hotkeys: <Hotkeys />,
      desktop: <DesktopSettings />,
      password: <Password />,
      sessions: <Sessions />,
      notifications: <Notifications />,
      others: <Others />,
      deleteAccount: <DeleteAccount />
    }),
    []
  );

  const entries = useMemo<TSettingsEntry[]>(
    () => getUserSettingsEntries(t, contentMap),
    [t, contentMap]
  );

  // the plugin owns everything inside its entry, so this renders its slot and
  // nothing else
  const pluginEntries = useMemo<TSettingsEntry[]>(() => {
    if (!can(Permission.USE_PLUGINS)) return [];

    return userSettingsPlugins.map((plugin) => ({
      id: `plugin-${plugin.pluginId}`,
      label: plugin.name,
      icon: Package,
      logo: plugin.logo,
      content: (
        <PluginSlotRenderer
          slotId={PluginSlot.USER_SETTINGS}
          onlyPluginId={plugin.pluginId}
        />
      )
    }));
  }, [can, userSettingsPlugins]);

  return (
    <SettingsShell
      title={t('userSettingsTitle')}
      close={close}
      entries={entries}
      pluginEntries={pluginEntries}
      defaultTab={defaultTab}
    />
  );
});

export { UserSettings };
