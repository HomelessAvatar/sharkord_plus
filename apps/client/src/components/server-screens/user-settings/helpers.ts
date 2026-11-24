import { isDesktopApp } from '@/helpers/is-desktop-app';
import type { TSettingsEntry } from '@sharkord/ui';
import {
  Bell,
  Headphones,
  Keyboard,
  KeyRound,
  Monitor,
  MonitorSmartphone,
  SlidersHorizontal,
  Trash2,
  User
} from 'lucide-react';
import type { ReactNode } from 'react';

export const getUserSettingsEntries = (
  t: (key: string) => string,
  contentMap: Record<string, ReactNode> = {}
): TSettingsEntry[] => [
  {
    id: 'profile',
    label: t('profileTab'),
    icon: User,
    content: contentMap.profile
  },
  {
    id: 'devices',
    label: t('devicesTab'),
    icon: Headphones,
    content: contentMap.devices
  },
  {
    id: 'hotkeys',
    label: t('hotkeysTab'),
    icon: Keyboard,
    content: contentMap.hotkeys
  },
  ...(isDesktopApp()
    ? [
        {
          id: 'desktop',
          label: t('desktopTab'),
          icon: Monitor,
          content: contentMap.desktop
        }
      ]
    : []),
  {
    id: 'password',
    label: t('passwordTab'),
    icon: KeyRound,
    content: contentMap.password
  },
  {
    id: 'sessions',
    label: t('userSessionsTab'),
    icon: MonitorSmartphone,
    content: contentMap.sessions
  },
  {
    id: 'notifications',
    label: t('notificationsTab'),
    icon: Bell,
    content: contentMap.notifications
  },
  {
    id: 'others',
    label: t('othersTab'),
    icon: SlidersHorizontal,
    content: contentMap.others
  },
  {
    id: 'delete-account',
    label: t('deleteAccountTab'),
    icon: Trash2,
    content: contentMap.deleteAccount
  }
];
