import { TextChannel } from '@/components/channel-view/text';
import { VoiceChannel } from '@/components/channel-view/voice';
import { Dialog } from '@/components/dialogs/dialogs';
import { PluginSlotRenderer } from '@/components/plugin-slot-renderer';
import { ServerScreen } from '@/components/server-screens/screens';
import { ThreadContent } from '@/components/thread-sidebar/thread-content';
import { setSelectedThread } from '@/features/app/actions';
import { useSelectedThread } from '@/features/app/hooks';
import { openDialog } from '@/features/dialogs/actions';
import { openServerScreen } from '@/features/server-screens/actions';
import {
  useSelectedChannelId,
  useSelectedChannelType
} from '@/features/server/channels/hooks';
import {
  useActiveFullscreenPluginId,
  usePublicServerSettings,
  useServerName
} from '@/features/server/hooks';
import { useThreadsOpen } from '@/features/server/threads/hooks';
import { ChannelType, PluginSlot } from '@sharkord/shared';
import { Button, IconButton } from '@sharkord/ui';
import { Hash, Menu, Search, Settings, Users } from 'lucide-react';
import { memo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { MobileActiveVoicePill } from './mobile-active-voice-pill';
import { useMobileDrawer } from './mobile-drawer-context';

type TContentWrapperProps = {
  isDmMode: boolean;
  selectedDmChannelId?: number;
};

const ContentWrapper = memo(
  ({ isDmMode, selectedDmChannelId }: TContentWrapperProps) => {
    const { t } = useTranslation();
    const selectedChannelId = useSelectedChannelId();
    const selectedChannelType = useSelectedChannelType();
    const serverName = useServerName();
    const activeFullscreenPluginId = useActiveFullscreenPluginId();
    const publicSettings = usePublicServerSettings();
    const threadsOpen = useThreadsOpen();
    const selectedThread = useSelectedThread();
    const { openMobileMenu, openMobileUsers } = useMobileDrawer();

    const openSearchDialog = useCallback(() => {
      openDialog(Dialog.SEARCH);
    }, []);

    const openUserSettings = useCallback(() => {
      openServerScreen(ServerScreen.USER_SETTINGS);
    }, []);

    const onCloseThread = useCallback(() => setSelectedThread(undefined), []);

    if (activeFullscreenPluginId) {
      return (
        <main className="flex flex-1 flex-col bg-background relative min-w-0 min-h-0">
          <div className="flex-col gap-2 h-full w-full flex overflow-auto relative bg-background">
            <PluginSlotRenderer
              slotId={PluginSlot.FULL_SCREEN}
              onlyPluginId={activeFullscreenPluginId}
            />
          </div>
        </main>
      );
    }

    let content;

    if (threadsOpen) {
      if (selectedThread) {
        content = (
          <ThreadContent
            key={selectedThread.parentMessageId}
            parentMessageId={selectedThread.parentMessageId}
            channelId={selectedThread.channelId}
            onClose={onCloseThread}
          />
        );
      } else {
        content = (
          <div className="flex flex-col h-full w-full">
            <div className="lg:hidden flex h-14 items-center justify-between px-3 border-b border-border bg-card w-full shrink-0">
              <IconButton
                icon={Menu}
                variant="ghost"
                className="md:hidden size-10 flex items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/40 [&_svg]:size-5"
                onClick={openMobileMenu}
                aria-label={t('viewChannels')}
              />
              <IconButton
                icon={Settings}
                variant="ghost"
                className="size-10 flex items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/40 [&_svg]:size-5"
                onClick={openUserSettings}
                aria-label="Ayarlar"
              />
            </div>
            <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
              {t('selectThreadPrompt')}
            </div>
          </div>
        );
      }

      return (
        <main className="flex flex-1 flex-col bg-background relative min-w-0 min-h-0">
          <MobileActiveVoicePill />
          {content}
        </main>
      );
    }

    if (isDmMode) {
      if (selectedDmChannelId) {
        content = (
          <TextChannel
            key={selectedDmChannelId}
            channelId={selectedDmChannelId}
          />
        );
      } else {
        content = (
          <div className="flex flex-col h-full w-full">
            <div className="lg:hidden flex h-14 items-center justify-between px-3 border-b border-border bg-card w-full shrink-0">
              <IconButton
                icon={Menu}
                variant="ghost"
                className="md:hidden size-10 flex items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/40 [&_svg]:size-5"
                onClick={openMobileMenu}
                aria-label={t('viewChannels')}
              />
              <IconButton
                icon={Settings}
                variant="ghost"
                className="size-10 flex items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/40 [&_svg]:size-5"
                onClick={openUserSettings}
                aria-label="Ayarlar"
              />
            </div>
            <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
              {t('selectDmPrompt')}
            </div>
          </div>
        );
      }

      return (
        <main className="flex flex-1 flex-col bg-background relative min-w-0 min-h-0">
          <MobileActiveVoicePill />
          {content}
        </main>
      );
    }

    if (selectedChannelId) {
      if (selectedChannelType === ChannelType.TEXT) {
        content = (
          <TextChannel key={selectedChannelId} channelId={selectedChannelId} />
        );
      } else if (selectedChannelType === ChannelType.VOICE) {
        content = (
          <VoiceChannel key={selectedChannelId} channelId={selectedChannelId} />
        );
      }
    } else {
      content = (
        <div className="flex flex-col h-full w-full">
          <div className="flex-col gap-2 h-full w-full hidden lg:flex overflow-auto">
            <PluginSlotRenderer slotId={PluginSlot.HOME_SCREEN} />
          </div>
          <div className="lg:hidden flex h-14 items-center justify-between px-3.5 border-b border-border bg-card w-full shrink-0">
            <IconButton
              icon={Menu}
              variant="ghost"
              className="md:hidden size-10 flex items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/40 [&_svg]:size-5"
              onClick={openMobileMenu}
              aria-label={t('viewChannels')}
            />
            <span className="font-semibold text-sm text-foreground truncate max-w-44 text-center">
              {serverName}
            </span>
            <div className="flex items-center gap-2.5">
              {publicSettings?.enableSearch && (
                <IconButton
                  icon={Search}
                  variant="ghost"
                  className="size-10 flex items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/40 [&_svg]:size-5"
                  onClick={openSearchDialog}
                  aria-label="Arama"
                />
              )}
              <IconButton
                icon={Settings}
                variant="ghost"
                className="size-10 flex items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/40 [&_svg]:size-5"
                onClick={openUserSettings}
                aria-label="Ayarlar"
              />
              <IconButton
                icon={Users}
                variant="ghost"
                className="size-10 flex items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/40 [&_svg]:size-5"
                onClick={openMobileUsers}
                aria-label={t('viewMembers')}
              />
            </div>
          </div>
          <div className="flex flex-col items-center justify-center flex-1 gap-5 p-6 text-center lg:hidden">
            <div className="w-16 h-16 rounded-2xl bg-card border border-border flex items-center justify-center shadow-lg">
              <Hash className="w-8 h-8 text-primary" />
            </div>
            <div className="flex flex-col gap-1 max-w-xs">
              <h2 className="text-xl font-bold tracking-tight text-foreground">
                {t('welcomeToServer', { name: serverName })}
              </h2>
              <p className="text-xs text-muted-foreground">
                {t('swipeRightForChannels')}
              </p>
            </div>
            <div className="flex flex-col gap-2.5 w-full max-w-xs">
              <Button
                variant="default"
                size="default"
                className="md:hidden w-full justify-center gap-2 font-medium h-11 text-sm rounded-lg"
                onClick={openMobileMenu}
              >
                <Menu className="w-4 h-4" />
                <span>{t('viewChannels')}</span>
              </Button>
              <Button
                variant="outline"
                size="sm"
                className="w-full justify-center gap-2 text-muted-foreground h-10 text-sm rounded-lg"
                onClick={openMobileUsers}
              >
                <Users className="w-4 h-4" />
                <span>{t('viewMembers')}</span>
              </Button>
            </div>
          </div>
        </div>
      );
    }

    return (
      <main className="flex flex-1 flex-col bg-background relative min-w-0 min-h-0">
        <MobileActiveVoicePill />
        {content}
      </main>
    );
  }
);

export { ContentWrapper };
