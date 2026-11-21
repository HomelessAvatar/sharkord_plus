import { memo, type ReactNode } from 'react';

type TSectionHeaderProps = {
  title: string;
  children?: ReactNode;
};

const SectionHeader = memo(({ title, children }: TSectionHeaderProps) => (
  <div className="mb-1 flex items-center justify-between min-h-[38px] md:min-h-0 px-2.5 py-1.5 md:px-2 md:py-1">
    <span className="text-[13px] md:text-xs font-semibold text-muted-foreground">
      {title}
    </span>
    {children}
  </div>
));

export { SectionHeader };
