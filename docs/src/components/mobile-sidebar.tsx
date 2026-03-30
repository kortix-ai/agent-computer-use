'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Menu } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Sheet, SheetContent, SheetTrigger, SheetTitle } from '@/components/ui/sheet';
import { SearchButton } from './search';
import {
  Rocket,
  Terminal,
  Zap,
  Eye,
  Search,
  Layers,
  FileText,
  Cpu,
  BookOpen,
  MousePointerClick,
  ChevronRight,
} from 'lucide-react';

const navigation = [
  {
    title: 'Getting Started',
    items: [
      { title: 'What is agent-click?', href: '/', icon: BookOpen },
      { title: 'Quick Start', href: '/quickstart', icon: Rocket },
      { title: 'Installation', href: '/installation', icon: Terminal },
    ],
  },
  {
    title: 'Core Concepts',
    items: [
      { title: 'Snapshots & Refs', href: '/snapshots', icon: Zap },
      { title: 'Selectors', href: '/selectors', icon: Search },
      { title: 'Background Ops', href: '/background', icon: Eye },
    ],
  },
  {
    title: 'Commands',
    items: [
      { title: 'click', href: '/commands/click', icon: MousePointerClick },
      { title: 'type', href: '/commands/type', icon: Terminal },
      { title: 'snapshot', href: '/commands/snapshot', icon: Layers },
      { title: 'All Commands', href: '/commands', icon: Terminal },
    ],
  },
  {
    title: 'Automation',
    items: [
      { title: 'YAML Workflows', href: '/workflows', icon: FileText },
      { title: 'Batch Execution', href: '/batch', icon: Layers },
      { title: 'AI Agent Mode', href: '/ai-mode', icon: Cpu },
    ],
  },
  {
    title: 'Reference',
    items: [{ title: 'Global Options', href: '/options', icon: Terminal }],
  },
];

export function MobileSidebar() {
  const [open, setOpen] = useState(false);
  const pathname = usePathname();

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger className="lg:hidden flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors">
        <Menu className="h-4 w-4" />
      </SheetTrigger>
      <SheetContent side="right" className="w-72 p-0">
        <SheetTitle className="sr-only">Navigation</SheetTitle>
        <div className="flex flex-col h-full">
          <div className="p-4 border-b border-border/50">
            <SearchButton />
          </div>
          <nav className="flex-1 overflow-y-auto p-4 space-y-6">
            {navigation.map((group) => (
              <div key={group.title}>
                <h4 className="mb-1.5 px-2 text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground/60">
                  {group.title}
                </h4>
                <ul className="space-y-px">
                  {group.items.map((item) => {
                    const isActive = pathname === item.href;
                    const Icon = item.icon;
                    return (
                      <li key={item.href}>
                        <Link
                          href={item.href}
                          onClick={() => setOpen(false)}
                          className={cn(
                            'group flex items-center gap-2 rounded-lg px-2 py-[6px] text-[13px] transition-all duration-150',
                            isActive
                              ? 'bg-foreground/[0.06] text-foreground font-medium'
                              : 'text-muted-foreground hover:text-foreground hover:bg-foreground/[0.03]',
                          )}
                        >
                          <Icon
                            className={cn(
                              'h-3.5 w-3.5 shrink-0 transition-colors',
                              isActive
                                ? 'text-foreground'
                                : 'text-muted-foreground/50 group-hover:text-muted-foreground',
                            )}
                          />
                          <span>{item.title}</span>
                          {isActive && (
                            <ChevronRight className="ml-auto h-3 w-3 text-muted-foreground/40" />
                          )}
                        </Link>
                      </li>
                    );
                  })}
                </ul>
              </div>
            ))}
          </nav>
        </div>
      </SheetContent>
    </Sheet>
  );
}
