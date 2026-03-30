'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { ChevronLeft, ChevronRight } from 'lucide-react';

const pages = [
  { href: '/', title: 'What is agent-click?' },
  { href: '/quickstart', title: 'Quick Start' },
  { href: '/installation', title: 'Installation' },
  { href: '/snapshots', title: 'Snapshots & Refs' },
  { href: '/selectors', title: 'Selectors' },
  { href: '/background', title: 'Background Ops' },
  { href: '/commands/click', title: 'click' },
  { href: '/commands/type', title: 'type' },
  { href: '/commands/snapshot', title: 'snapshot' },
  { href: '/commands', title: 'All Commands' },
  { href: '/workflows', title: 'Workflows' },
  { href: '/batch', title: 'Batch Execution' },
  { href: '/ai-mode', title: 'AI Agent Mode' },
  { href: '/architecture', title: 'Architecture' },
  { href: '/options', title: 'Global Options' },
];

export function PageNav() {
  const pathname = usePathname();
  const index = pages.findIndex((p) => p.href === pathname);
  if (index === -1) return null;

  const prev = index > 0 ? pages[index - 1] : null;
  const next = index < pages.length - 1 ? pages[index + 1] : null;

  return (
    <div className="mt-12 flex items-stretch gap-3 border-t border-border/40 pt-6">
      {prev ? (
        <Link
          href={prev.href}
          className="group flex-1 flex items-center gap-2 rounded-lg border border-border/50 px-4 py-3 text-sm transition-colors hover:border-border hover:bg-muted/30"
        >
          <ChevronLeft className="h-3.5 w-3.5 text-muted-foreground/50 group-hover:text-foreground transition-colors" />
          <div className="flex flex-col">
            <span className="text-[10px] uppercase tracking-wider text-muted-foreground/50">
              Previous
            </span>
            <span className="text-foreground/80 group-hover:text-foreground transition-colors">
              {prev.title}
            </span>
          </div>
        </Link>
      ) : (
        <div className="flex-1" />
      )}
      {next ? (
        <Link
          href={next.href}
          className="group flex-1 flex items-center justify-end gap-2 rounded-lg border border-border/50 px-4 py-3 text-sm text-right transition-colors hover:border-border hover:bg-muted/30"
        >
          <div className="flex flex-col items-end">
            <span className="text-[10px] uppercase tracking-wider text-muted-foreground/50">
              Next
            </span>
            <span className="text-foreground/80 group-hover:text-foreground transition-colors">
              {next.title}
            </span>
          </div>
          <ChevronRight className="h-3.5 w-3.5 text-muted-foreground/50 group-hover:text-foreground transition-colors" />
        </Link>
      ) : (
        <div className="flex-1" />
      )}
    </div>
  );
}
