'use client';

import { useEffect, useState, useCallback } from 'react';
import { usePathname } from 'next/navigation';
import { cn } from '@/lib/utils';

interface TocItem {
  id: string;
  text: string;
  level: number;
}

export function TableOfContents() {
  const [headings, setHeadings] = useState<TocItem[]>([]);
  const [activeId, setActiveId] = useState<string>('');
  const pathname = usePathname();

  useEffect(() => {
    // Wait for MDX content to render
    const timer = setTimeout(() => {
      const elements = document.querySelectorAll('main h2, main h3');
      const items: TocItem[] = [];

      elements.forEach((el) => {
        // Generate id if missing
        if (!el.id) {
          const text = el.textContent?.trim() || '';
          el.id = text
            .toLowerCase()
            .replace(/[^a-z0-9]+/g, '-')
            .replace(/(^-|-$)/g, '');
        }
        if (el.id) {
          items.push({
            id: el.id,
            text: el.textContent?.trim() || '',
            level: el.tagName === 'H2' ? 2 : 3,
          });
        }
      });

      setHeadings(items);

      const observer = new IntersectionObserver(
        (entries) => {
          for (const entry of entries) {
            if (entry.isIntersecting) {
              setActiveId(entry.target.id);
            }
          }
        },
        { rootMargin: '-80px 0px -70% 0px', threshold: 0 },
      );

      elements.forEach((el) => observer.observe(el));
      return () => observer.disconnect();
    }, 100);

    return () => clearTimeout(timer);
  }, [pathname]);

  const scrollTo = useCallback((e: React.MouseEvent, id: string) => {
    e.preventDefault();
    const el = document.getElementById(id);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
      // Update URL hash without jumping
      window.history.replaceState(null, '', `#${id}`);
      setActiveId(id);
    }
  }, []);

  if (headings.length === 0) return null;

  return (
    <aside className="hidden lg:flex w-[160px] shrink-0 flex-col sticky top-16 h-[calc(100vh-4rem)] pt-8 pl-6">
      <h4 className="text-[10px] font-medium uppercase tracking-[0.1em] text-muted-foreground/40 mb-3">
        On this page
      </h4>
      <nav className="relative space-y-0">
        <div className="absolute left-0 top-0 w-px bg-border/30 h-full" />
        {headings.map((h) => (
          <a
            key={h.id}
            href={`#${h.id}`}
            onClick={(e) => scrollTo(e, h.id)}
            className={cn(
              'relative block text-[12px] leading-normal py-1.5 transition-all duration-200 border-l',
              h.level === 3 ? 'pl-5' : 'pl-3',
              activeId === h.id
                ? 'text-foreground border-foreground'
                : 'text-muted-foreground/40 border-transparent hover:text-muted-foreground/70 hover:border-border/50',
            )}
          >
            {h.text}
          </a>
        ))}
      </nav>
    </aside>
  );
}
