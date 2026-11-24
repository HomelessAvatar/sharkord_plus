import { LeftSidebar } from '@/components/left-sidebar';
import { ModViewSheet } from '@/components/mod-view-sheet';
import { Protect } from '@/components/protect';
import { RightSidebar } from '@/components/right-sidebar';
import { ThreadSidebar } from '@/components/thread-sidebar';
import { TopBar } from '@/components/top-bar';
import { VoiceChatSidebar } from '@/components/voice-chat-sidebar';
import { VoiceProvider } from '@/components/voice-provider';
import { useSelectedDmChannelId, useThreadSidebar } from '@/features/app/hooks';
import { setDmsOpen } from '@/features/server/actions';
import { setSelectedChannelId } from '@/features/server/channels/actions';
import { useSelectedChannelId } from '@/features/server/channels/hooks';
import { useDmsOpen, usePublicServerSettings } from '@/features/server/hooks';
import { getDesktopApi } from '@/helpers/desktop-api';
import {
  getLocalStorageItemBool,
  LocalStorageKey,
  setLocalStorageItemBool
} from '@/helpers/storage';
import { useSwipeGestures } from '@/hooks/use-swipe-gestures';
import { cn } from '@/lib/utils';
import { Permission, TestId } from '@sharkord/shared';
import { memo, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { ContentWrapper } from './content-wrapper';
import { MobileDrawerContext } from './mobile-drawer-context';
import { PreventBrowser } from './prevent-browser';

const ServerView = memo(() => {
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
  const [isMobileUsersOpen, setIsMobileUsersOpen] = useState(false);
  const [isDesktopRightSidebarOpen, setIsDesktopRightSidebarOpen] = useState(
    getLocalStorageItemBool(LocalStorageKey.RIGHT_SIDEBAR_STATE, true)
  );
  const [isDesktopLeftSidebarOpen, setIsDesktopLeftSidebarOpen] = useState(
    getLocalStorageItemBool(LocalStorageKey.LEFT_SIDEBAR_STATE, true)
  );
  const dmsOpen = useDmsOpen();
  const selectedDmChannelId = useSelectedDmChannelId();
  const selectedChannelId = useSelectedChannelId();
  const publicSettings = usePublicServerSettings();
  const previousServerChannelIdRef = useRef<number | undefined>(undefined);
  const { isOpen: isThreadSidebarOpen } = useThreadSidebar();

  const openMobileMenu = useCallback(() => {
    setIsMobileMenuOpen(true);
    setIsMobileUsersOpen(false);
  }, []);

  const closeMobileMenu = useCallback(() => {
    setIsMobileMenuOpen(false);
  }, []);

  const toggleMobileMenu = useCallback(() => {
    setIsMobileMenuOpen((prev) => !prev);
    setIsMobileUsersOpen(false);
  }, []);

  const openMobileUsers = useCallback(() => {
    setIsMobileUsersOpen(true);
    setIsMobileMenuOpen(false);
  }, []);

  const closeMobileUsers = useCallback(() => {
    setIsMobileUsersOpen(false);
  }, []);

  const toggleMobileUsers = useCallback(() => {
    setIsMobileUsersOpen((prev) => !prev);
    setIsMobileMenuOpen(false);
  }, []);

  const drawerContextValue = useMemo(
    () => ({
      isMobileMenuOpen,
      isMobileUsersOpen,
      openMobileMenu,
      closeMobileMenu,
      toggleMobileMenu,
      openMobileUsers,
      closeMobileUsers,
      toggleMobileUsers
    }),
    [
      isMobileMenuOpen,
      isMobileUsersOpen,
      openMobileMenu,
      closeMobileMenu,
      toggleMobileMenu,
      openMobileUsers,
      closeMobileUsers,
      toggleMobileUsers
    ]
  );

  const handleDesktopRightSidebarToggle = useCallback(() => {
    setIsDesktopRightSidebarOpen((prev) => !prev);
    setLocalStorageItemBool(
      LocalStorageKey.RIGHT_SIDEBAR_STATE,
      !isDesktopRightSidebarOpen
    );
  }, [isDesktopRightSidebarOpen]);

  const handleDesktopLeftSidebarToggle = useCallback(() => {
    setIsDesktopLeftSidebarOpen((prev) => {
      const next = !prev;
      setLocalStorageItemBool(LocalStorageKey.LEFT_SIDEBAR_STATE, next);
      return next;
    });
  }, []);

  useEffect(() => {
    const desktopApi = getDesktopApi();
    if (!desktopApi) {
      return;
    }

    return desktopApi.onServerListVisibleChange((visible) => {
      setIsDesktopLeftSidebarOpen(visible);
      setLocalStorageItemBool(LocalStorageKey.LEFT_SIDEBAR_STATE, visible);
    });
  }, []);

  const handleSwipeRight = useCallback(() => {
    if (isMobileMenuOpen || isMobileUsersOpen) {
      setIsMobileMenuOpen(false);
      setIsMobileUsersOpen(false);
      return;
    }

    setIsMobileMenuOpen(true);
  }, [isMobileMenuOpen, isMobileUsersOpen]);

  const handleSwipeLeft = useCallback(() => {
    if (isMobileMenuOpen || isMobileUsersOpen) {
      setIsMobileMenuOpen(false);
      setIsMobileUsersOpen(false);
      return;
    }

    setIsMobileUsersOpen(true);
  }, [isMobileMenuOpen, isMobileUsersOpen]);

  const swipeHandlers = useSwipeGestures({
    onSwipeRight: handleSwipeRight,
    onSwipeLeft: handleSwipeLeft
  });

  // Automatically close mobile drawers when active channel changes
  useEffect(() => {
    setIsMobileMenuOpen(false);
    setIsMobileUsersOpen(false);
  }, [selectedChannelId, selectedDmChannelId]);

  useEffect(() => {
    if (publicSettings?.directMessagesEnabled === false && dmsOpen) {
      setDmsOpen(false);

      if (previousServerChannelIdRef.current) {
        setSelectedChannelId(previousServerChannelIdRef.current);
      }
    }
  }, [publicSettings?.directMessagesEnabled, dmsOpen]);

  return (
    <VoiceProvider>
      <MobileDrawerContext.Provider value={drawerContextValue}>
        <div
          data-testid={TestId.SERVER_VIEW}
          className="flex h-dvh flex-col bg-background text-foreground dark"
          {...swipeHandlers}
        >
          <TopBar
            onToggleLeftSidebar={handleDesktopLeftSidebarToggle}
            isLeftOpen={isDesktopLeftSidebarOpen}
            onToggleRightSidebar={handleDesktopRightSidebarToggle}
            isOpen={isDesktopRightSidebarOpen}
          />
          <div className="relative flex min-h-0 flex-1 overflow-hidden">
            <PreventBrowser />

            {isMobileMenuOpen && (
              <div
                className="md:hidden fixed inset-0 bg-black/60 backdrop-blur-xs z-30 transition-opacity"
                onClick={closeMobileMenu}
              />
            )}

            {isMobileUsersOpen && (
              <div
                className="lg:hidden fixed inset-0 bg-black/60 backdrop-blur-xs z-30 transition-opacity"
                onClick={closeMobileUsers}
              />
            )}

            <LeftSidebar
              className={cn(
                'md:relative md:flex fixed inset-0 left-0 h-full z-40 md:z-0 transition-transform duration-300 ease-in-out shadow-2xl md:shadow-none max-md:!w-[85vw] max-md:!max-w-[340px]',
                isMobileMenuOpen
                  ? 'translate-x-0'
                  : '-translate-x-full md:translate-x-0'
              )}
              isOpen={isMobileMenuOpen || isDesktopLeftSidebarOpen}
            />

            <ContentWrapper
              isDmMode={dmsOpen}
              selectedDmChannelId={selectedDmChannelId}
            />

            <VoiceChatSidebar />

            <ThreadSidebar isOpen={isThreadSidebarOpen} />

            <RightSidebar
              className={cn(
                'fixed top-0 bottom-0 right-0 h-full z-40 shadow-2xl lg:shadow-none max-lg:!w-[85vw] max-lg:!max-w-[340px]',
                'lg:relative lg:z-0 transition-transform duration-300 ease-in-out',
                isMobileUsersOpen
                  ? 'translate-x-0 lg:translate-x-0'
                  : 'translate-x-full lg:translate-x-0'
              )}
              isOpen={isMobileUsersOpen || isDesktopRightSidebarOpen}
            />

            <Protect permission={Permission.MANAGE_USERS}>
              <ModViewSheet />
            </Protect>
          </div>
        </div>
      </MobileDrawerContext.Provider>
    </VoiceProvider>
  );
});

export { ServerView };
