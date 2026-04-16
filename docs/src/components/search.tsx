'use client';

import { useCallback, useEffect, useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/ui/command';
import {
  Search as SearchIcon,
  FileText,
  Hash,
  Rocket,
  Terminal,
  Zap,
  MousePointer,
  Type,
  Camera,
  Play,
  Layers,
  Bot,
  Box,
  Settings,
  BookOpen,
  Plug,
} from 'lucide-react';
import searchIndex from '@/generated/search-index.json';

interface SearchEntry {
  title: string;
  content: string;
  href: string;
  pageTitle: string;
}

const quickLinks = [
  {
    title: 'Introduction',
    href: '/',
    icon: BookOpen,
    group: 'Getting Started',
  },
  {
    title: 'Quick Start',
    href: '/quickstart',
    icon: Rocket,
    group: 'Getting Started',
  },
  {
    title: 'Installation',
    href: '/installation',
    icon: Terminal,
    group: 'Getting Started',
  },
  {
    title: 'Claude Code & Agents',
    href: '/skills',
    icon: Plug,
    group: 'Getting Started',
  },
  {
    title: 'Snapshots & Refs',
    href: '/snapshots',
    icon: Camera,
    group: 'Core Concepts',
  },
  {
    title: 'Selectors',
    href: '/selectors',
    icon: SearchIcon,
    group: 'Core Concepts',
  },
  {
    title: 'Background Ops',
    href: '/background',
    icon: Zap,
    group: 'Core Concepts',
  },
  {
    title: 'click',
    href: '/commands/click',
    icon: MousePointer,
    group: 'Commands',
  },
  { title: 'type', href: '/commands/type', icon: Type, group: 'Commands' },
  {
    title: 'snapshot',
    href: '/commands/snapshot',
    icon: Camera,
    group: 'Commands',
  },
  {
    title: 'All Commands',
    href: '/commands',
    icon: Terminal,
    group: 'Commands',
  },
  {
    title: 'YAML Workflows',
    href: '/workflows',
    icon: Play,
    group: 'Automation',
  },
  {
    title: 'Batch Execution',
    href: '/batch',
    icon: Layers,
    group: 'Automation',
  },
  {
    title: 'Architecture',
    href: '/architecture',
    icon: Box,
    group: 'Reference',
  },
];

export function SearchButton() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const router = useRouter();

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen((o) => !o);
      }
    };
    document.addEventListener('keydown', down);
    return () => document.removeEventListener('keydown', down);
  }, []);

  const results = useMemo(() => {
    if (!query || query.length < 2) return [];
    const q = query.toLowerCase();
    const scored: (SearchEntry & { score: number })[] = [];

    for (const entry of searchIndex as SearchEntry[]) {
      let score = 0;
      const titleLower = entry.title.toLowerCase();
      const contentLower = entry.content.toLowerCase();
      const pageLower = entry.pageTitle.toLowerCase();

      if (titleLower === q) score += 100;
      else if (titleLower.startsWith(q)) score += 80;
      else if (titleLower.includes(q)) score += 60;
      if (pageLower.includes(q)) score += 20;

      const words = q.split(/\s+/);
      for (const word of words) {
        if (contentLower.includes(word)) score += 10;
        if (titleLower.includes(word)) score += 30;
      }

      if (score > 0) scored.push({ ...entry, score });
    }

    scored.sort((a, b) => b.score - a.score);
    return scored.slice(0, 12);
  }, [query]);

  const onSelect = useCallback(
    (href: string) => {
      setOpen(false);
      setQuery('');
      router.push(href);
    },
    [router],
  );

  const grouped = useMemo(() => {
    const groups: Record<string, typeof results> = {};
    for (const r of results) {
      const key = r.pageTitle || 'Other';
      if (!groups[key]) groups[key] = [];
      groups[key].push(r);
    }
    return groups;
  }, [results]);

  const isSearching = query.length >= 2;

  const quickLinkGroups = [...new Set(quickLinks.map((l) => l.group))];

  return (
    <>
      <button
        onClick={() => setOpen(true)}
        className="flex items-center gap-2 rounded-lg border border-border/50 bg-muted/30 px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground hover:border-border hover:bg-muted/50 w-48"
      >
        <SearchIcon className="h-3.5 w-3.5" />
        <span className="flex-1 text-left text-xs">Search...</span>
        <kbd className="hidden sm:inline-flex items-center gap-0.5 rounded border border-border/50 bg-muted px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground">
          <span className="text-xs">&#8984;</span>K
        </kbd>
      </button>
      <CommandDialog open={open} onOpenChange={setOpen}>
        <CommandInput placeholder="Search docs..." value={query} onValueChange={setQuery} />
        <CommandList>
          {isSearching ? (
            results.length === 0 ? (
              <CommandEmpty>No results for &quot;{query}&quot;</CommandEmpty>
            ) : (
              Object.entries(grouped).map(([page, items]) => (
                <CommandGroup key={page} heading={page}>
                  {items.map((item, i) => (
                    <CommandItem
                      key={`${item.href}-${i}`}
                      value={`${item.title} ${item.content.slice(0, 100)}`}
                      onSelect={() => onSelect(item.href)}
                      className="gap-2"
                    >
                      {item.title === item.pageTitle ? (
                        <FileText className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                      ) : (
                        <Hash className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                      )}
                      <div className="flex flex-col min-w-0">
                        <span className="text-sm truncate">{item.title}</span>
                        <span className="text-xs text-muted-foreground truncate">
                          {item.content.slice(0, 80)}...
                        </span>
                      </div>
                    </CommandItem>
                  ))}
                </CommandGroup>
              ))
            )
          ) : (
            <>
              {quickLinkGroups.map((group, gi) => (
                <div key={group}>
                  {gi > 0 && <CommandSeparator />}
                  <CommandGroup heading={group}>
                    {quickLinks
                      .filter((l) => l.group === group)
                      .map((link) => (
                        <CommandItem
                          key={link.href}
                          value={link.title}
                          onSelect={() => onSelect(link.href)}
                          className="gap-2"
                        >
                          <link.icon className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                          <span className="text-sm">{link.title}</span>
                        </CommandItem>
                      ))}
                  </CommandGroup>
                </div>
              ))}
              <CommandSeparator />
              <CommandGroup heading="Keyboard Shortcuts">
                <div className="px-2 py-2 space-y-1.5 text-xs text-muted-foreground">
                  <div className="flex justify-between">
                    <span>Open search</span>
                    <div className="flex gap-1">
                      <kbd className="rounded border border-border/50 bg-muted px-1.5 py-0.5 text-[10px] font-medium">
                        &#8984;K
                      </kbd>
                    </div>
                  </div>
                  <div className="flex justify-between">
                    <span>Navigate results</span>
                    <div className="flex gap-1">
                      <kbd className="rounded border border-border/50 bg-muted px-1.5 py-0.5 text-[10px] font-medium">
                        ↑
                      </kbd>
                      <kbd className="rounded border border-border/50 bg-muted px-1.5 py-0.5 text-[10px] font-medium">
                        ↓
                      </kbd>
                    </div>
                  </div>
                  <div className="flex justify-between">
                    <span>Open page</span>
                    <kbd className="rounded border border-border/50 bg-muted px-1.5 py-0.5 text-[10px] font-medium">
                      Enter
                    </kbd>
                  </div>
                  <div className="flex justify-between">
                    <span>Close</span>
                    <kbd className="rounded border border-border/50 bg-muted px-1.5 py-0.5 text-[10px] font-medium">
                      Esc
                    </kbd>
                  </div>
                </div>
              </CommandGroup>
            </>
          )}
        </CommandList>
      </CommandDialog>
    </>
  );
}
