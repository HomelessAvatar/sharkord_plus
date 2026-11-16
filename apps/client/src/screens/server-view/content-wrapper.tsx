import { TextChannel } from '@/components/channel-view/text';
import { VoiceChannel } from '@/components/channel-view/voice';
import { PluginSlotRenderer } from '@/components/plugin-slot-renderer';
import { ThreadContent } from '@/components/thread-sidebar/thread-content';
import { setSelectedThread } from '@/features/app/actions';
import { useSelectedThread } from '@/features/app/hooks';
import {
  useSelectedChannelId,
  useSelectedChannelType
} from '@/features/server/channels/hooks';
import {
  useActiveFullscreenPluginId,
  useServerName
} from '@/features/server/hooks';
import { useThreadsOpen } from '@/features/server/threads/hooks';
import { ChannelType, PluginSlot } from '@sharkord/shared';
import { Button } from '@sharkord/ui';
import { Hash, Menu, Users } from 'lucide-react';
import { memo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
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
    const threadsOpen = useThreadsOpen();
    const selectedThread = useSelectedThread();
    const { openMobileMenu, openMobileUsers } = useMobileDrawer();

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
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            {t('selectThreadPrompt')}
          </div>
        );
      }

      return (
        <main className="flex flex-1 flex-col bg-background relative min-w-0 min-h-0">
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
          <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
            {t('selectDmPrompt')}
          </div>
        );
      }

      return (
        <main className="flex flex-1 flex-col bg-background relative min-w-0 min-h-0">
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
        <>
          <div className="flex-col gap-2 h-full w-full hidden lg:flex overflow-auto">
            <PluginSlotRenderer slotId={PluginSlot.HOME_SCREEN} />
          </div>
          <div className="flex flex-col items-center justify-center h-full gap-5 p-6 text-center md:hidden">
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
                className="w-full justify-center gap-2 font-medium"
                onClick={openMobileMenu}
              >
                <Menu className="w-4 h-4" />
                <span>{t('swipeRightForChannels')}</span>
              </Button>
              <Button
                variant="outline"
                size="sm"
                className="w-full justify-center gap-2 text-muted-foreground"
                onClick={openMobileUsers}
              >
                <Users className="w-4 h-4" />
                <span>{t('swipeLeftForUsers')}</span>
              </Button>
            </div>
          </div>
        </>
      );
    }

    return (
      <main className="flex flex-1 flex-col bg-background relative min-w-0 min-h-0">
        {content}
      </main>
    );
  }
);

export { ContentWrapper };
