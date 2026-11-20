import { toggleVoiceChatSidebar } from '@/features/app/actions';
import { useVoiceChatSidebar } from '@/features/app/hooks';
import { useChannelById } from '@/features/server/channels/hooks';
import {
  useHasUnreadMentions,
  useUnreadMessagesCount,
  useVoiceUsersByChannelId
} from '@/features/server/hooks';
import { useOwnUserId } from '@/features/server/users/hooks';
import {
  useHideNonVideoParticipants,
  useHideOwnScreenShare,
  useVoiceChannelExternalStreamsList
} from '@/features/server/voice/hooks';
import { cn } from '@/lib/utils';
import { useMobileDrawer } from '@/screens/server-view/mobile-drawer-context';
import { Button, IconButton, UnreadCount } from '@sharkord/ui';
import { Menu, MessageSquare, Users, Volume2 } from 'lucide-react';
import { memo, useCallback, useMemo } from 'react';
import { ControlsBar } from './controls-bar';
import { ExternalStreamCard } from './external-stream-card';
import { usePinCardController } from './hooks/use-pin-card-controller';
import { ScreenShareCard } from './screen-share-card';
import { VoiceGrid } from './voice-grid';
import { VoiceUserCard } from './voice-user-card';

type TChannelProps = {
  channelId: number;
};

const VoiceChannel = memo(({ channelId }: TChannelProps) => {
  const channel = useChannelById(channelId);
  const voiceUsers = useVoiceUsersByChannelId(channelId);
  const externalStreams = useVoiceChannelExternalStreamsList(channelId);
  const { pinnedCard, pinCard, unpinCard, isPinned } = usePinCardController();
  const hideNonVideoParticipants = useHideNonVideoParticipants();
  const hideOwnScreenShare = useHideOwnScreenShare();
  const ownUserId = useOwnUserId();
  const { toggleMobileMenu, toggleMobileUsers } = useMobileDrawer();
  const isAnyCardPinned = pinnedCard !== undefined;
  const { isOpen: isVoiceChatOpen, channelId: openVoiceChatChannelId } =
    useVoiceChatSidebar();
  const isThisVoiceChatOpen =
    isVoiceChatOpen && openVoiceChatChannelId === channelId;
  const unreadCount = useUnreadMessagesCount(channelId);
  const hasMentions = useHasUnreadMentions(channelId);

  const handleToggleVoiceChat = useCallback(() => {
    toggleVoiceChatSidebar(channelId);
  }, [channelId]);

  const cards = useMemo(() => {
    const cards: React.ReactNode[] = [];

    // Check if there are any video streams at all
    const hasAnyVideoStreams =
      voiceUsers.some(
        (user) => user.state.webcamEnabled || user.state.sharingScreen
      ) || externalStreams.some((stream) => stream.tracks.video);

    // Only apply the filter if there are some video streams
    const shouldFilterNonVideo = hideNonVideoParticipants && hasAnyVideoStreams;

    voiceUsers.forEach((voiceUser) => {
      const userCardId = `user-${voiceUser.id}`;
      const hasVideo = voiceUser.state.webcamEnabled;

      // Only show user card if not filtering, or if they have video
      if (!shouldFilterNonVideo || hasVideo) {
        cards.push(
          <VoiceUserCard
            key={userCardId}
            userId={voiceUser.id}
            channelId={channelId}
            isPinned={isPinned(userCardId)}
            isAnyCardPinned={isAnyCardPinned}
            cardId={userCardId}
            onPin={pinCard}
            onUnpin={unpinCard}
            voiceUser={voiceUser}
          />
        );
      }

      // Screen shares always have video, so always show them
      const shouldHideOwnScreenShare =
        hideOwnScreenShare && voiceUser.id === ownUserId;
      if (voiceUser.state.sharingScreen && !shouldHideOwnScreenShare) {
        const screenShareCardId = `screen-share-${voiceUser.id}`;

        cards.push(
          <ScreenShareCard
            key={screenShareCardId}
            userId={voiceUser.id}
            isPinned={isPinned(screenShareCardId)}
            isAnyCardPinned={isAnyCardPinned}
            cardId={screenShareCardId}
            onPin={pinCard}
            onUnpin={unpinCard}
            showPinControls
          />
        );
      }
    });

    externalStreams.forEach((stream) => {
      const externalStreamCardId = `external-stream-${stream.streamId}`;
      const hasVideo = stream.tracks.video;

      // Only show external stream card if not filtering, or if it has video
      if (!shouldFilterNonVideo || hasVideo) {
        cards.push(
          <ExternalStreamCard
            key={externalStreamCardId}
            streamId={stream.streamId}
            stream={stream}
            isPinned={isPinned(externalStreamCardId)}
            isAnyCardPinned={isAnyCardPinned}
            cardId={externalStreamCardId}
            onPin={pinCard}
            onUnpin={unpinCard}
            showPinControls
          />
        );
      }
    });

    return cards;
  }, [
    voiceUsers,
    externalStreams,
    channelId,
    isPinned,
    pinCard,
    unpinCard,
    hideNonVideoParticipants,
    hideOwnScreenShare,
    ownUserId,
    isAnyCardPinned
  ]);

  if (voiceUsers.length === 0) {
    return (
      <div className="flex-1 flex flex-col size-full relative">
        <div className="md:hidden flex h-12 items-center justify-between px-3 border-b border-border bg-card">
          <IconButton
            icon={Menu}
            size="sm"
            variant="ghost"
            className="-ml-1 text-muted-foreground hover:text-foreground"
            onClick={toggleMobileMenu}
          />
          <div className="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground">
            <Volume2 className="w-3.5 h-3.5" />
            <span className="truncate max-w-40">{channel?.name}</span>
          </div>
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="sm"
              className={cn(
                'h-8 px-2 text-muted-foreground hover:text-foreground relative',
                isThisVoiceChatOpen && 'text-foreground'
              )}
              onClick={handleToggleVoiceChat}
              aria-label="Voice Chat"
            >
              <div className="relative flex items-center justify-center">
                <MessageSquare
                  className={cn(
                    'w-4 h-4 transition-all duration-200 ease-in-out',
                    isThisVoiceChatOpen && 'fill-current'
                  )}
                />
                <UnreadCount
                  count={unreadCount}
                  hasMention={hasMentions}
                  className="absolute -top-2 -right-3 ml-0 min-w-4 h-4 px-1 text-[10px] leading-none"
                />
              </div>
            </Button>
            <IconButton
              icon={Users}
              size="sm"
              variant="ghost"
              className="text-muted-foreground hover:text-foreground"
              onClick={toggleMobileUsers}
            />
          </div>
        </div>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <p className="text-muted-foreground text-lg mb-2">
              No one in the voice channel
            </p>
            <p className="text-muted-foreground text-sm">
              Join the voice channel to start a meeting
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col size-full relative bg-background overflow-hidden group/voice-stage">
      <div className="md:hidden flex h-12 items-center justify-between px-3 border-b border-border bg-card shrink-0 z-10">
        <IconButton
          icon={Menu}
          size="sm"
          variant="ghost"
          className="-ml-1 text-muted-foreground hover:text-foreground"
          onClick={toggleMobileMenu}
        />
        <div className="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground">
          <Volume2 className="w-3.5 h-3.5" />
          <span className="truncate max-w-40">{channel?.name}</span>
        </div>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            className={cn(
              'h-8 px-2 text-muted-foreground hover:text-foreground relative',
              isThisVoiceChatOpen && 'text-foreground'
            )}
            onClick={handleToggleVoiceChat}
            aria-label="Voice Chat"
          >
            <div className="relative flex items-center justify-center">
              <MessageSquare
                className={cn(
                  'w-4 h-4 transition-all duration-200 ease-in-out',
                  isThisVoiceChatOpen && 'fill-current'
                )}
              />
              <UnreadCount
                count={unreadCount}
                hasMention={hasMentions}
                className="absolute -top-2 -right-3 ml-0 min-w-4 h-4 px-1 text-[10px] leading-none"
              />
            </div>
          </Button>
          <IconButton
            icon={Users}
            size="sm"
            variant="ghost"
            className="text-muted-foreground hover:text-foreground"
            onClick={toggleMobileUsers}
          />
        </div>
      </div>
      <VoiceGrid pinnedCardId={pinnedCard?.id}>{cards}</VoiceGrid>
      <ControlsBar channelId={channelId} />
    </div>
  );
});

export { VoiceChannel };
