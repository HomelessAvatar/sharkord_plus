import { setThreadsOpen } from '@/features/server/threads/actions';
import {
  useFollowedThreadsReadState,
  useThreadsOpen
} from '@/features/server/threads/hooks';
import { TestId } from '@sharkord/shared';
import { cn, Tooltip, UnreadCount } from '@sharkord/ui';
import { MessageSquareText, X } from 'lucide-react';
import { memo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';

const ThreadsButton = memo(() => {
  const { t } = useTranslation('sidebar');
  const threadsOpen = useThreadsOpen();
  const { unreadCount, hasUnreadMention } = useFollowedThreadsReadState();

  const onToggleThreads = useCallback(() => {
    setThreadsOpen(!threadsOpen);
  }, [threadsOpen]);

  return (
    <Tooltip content={threadsOpen ? t('closeThreads') : t('openThreads')}>
      <button
        type="button"
        data-testid={TestId.THREADS_TOGGLE}
        onClick={onToggleThreads}
        className={cn(
          'flex w-full items-center min-h-[44px] md:min-h-0 gap-2.5 md:gap-2 rounded-md px-2.5 py-2.5 md:px-2 md:py-1.5 text-[15px] md:text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground',
          threadsOpen &&
            'bg-accent text-accent-foreground ring-1 ring-inset ring-primary/30'
        )}
      >
        <MessageSquareText className="h-5 w-5 md:h-4 md:w-4 shrink-0" />
        <span className="flex-1 text-left">{t('threads')}</span>
        {threadsOpen && <X className="h-5 w-5 md:h-4 md:w-4 shrink-0" />}
        <UnreadCount count={unreadCount} hasMention={hasUnreadMention} />
      </button>
    </Tooltip>
  );
});

export { ThreadsButton };
