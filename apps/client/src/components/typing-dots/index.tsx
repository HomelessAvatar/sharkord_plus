import { cn } from '@/lib/utils';
import { memo } from 'react';

type TTypingDotsProps = {
  className?: string;
};

const TypingDots = memo(({ className = '' }: TTypingDotsProps) => {
  return (
    <div className={cn('flex items-center space-x-1 py-0.5', className)}>
      <div className="size-1.5 bg-muted-foreground/70 rounded-full animate-pulse [animation-delay:-0.3s]" />
      <div className="size-1.5 bg-muted-foreground/70 rounded-full animate-pulse [animation-delay:-0.15s]" />
      <div className="size-1.5 bg-muted-foreground/70 rounded-full animate-pulse" />
    </div>
  );
});

TypingDots.displayName = 'TypingDots';

export { TypingDots };
