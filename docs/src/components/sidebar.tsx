'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { cn } from '@/lib/utils';
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

interface NavItem {
  title: string;
  href: string;
  icon?: React.ComponentType<{ className?: string }>;
}

interface NavGroup {
  title: string;
  items: NavItem[];
}

const navigation: NavGroup[] = [
  {
    title: 'Getting Started',
    items: [
      { title: 'Introduction', href: '/', icon: BookOpen },
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
      { title: 'All Commands', href: '/commands', icon: FileText },
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

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="hidden lg:flex w-[220px] shrink-0 flex-col sticky top-16 h-[calc(100vh-4rem)] overflow-y-auto pb-10 pt-6 pr-2 pl-4">
      <nav className="space-y-6">
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
                      className={cn(
                        'group flex items-center gap-2 rounded-lg px-2 py-[6px] text-[13px] transition-all duration-150',
                        isActive
                          ? 'bg-foreground/[0.06] text-foreground font-medium'
                          : 'text-muted-foreground hover:text-foreground hover:bg-foreground/[0.03]',
                      )}
                    >
                      {Icon && (
                        <Icon
                          className={cn(
                            'h-3.5 w-3.5 shrink-0 transition-colors',
                            isActive
                              ? 'text-foreground'
                              : 'text-muted-foreground/50 group-hover:text-muted-foreground',
                          )}
                        />
                      )}
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
    </aside>
  );
}
