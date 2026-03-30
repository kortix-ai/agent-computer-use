'use client';

import Link from 'next/link';
import Image from 'next/image';
import { ThemeToggle } from './theme-toggle';
import { SearchButton } from './search';
import { MobileSidebar } from './mobile-sidebar';
import { useEffect, useState } from 'react';

export function Nav() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 12);
    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  }, []);

  return (
    <div className="sticky top-0 z-50 w-full px-4 pt-3 pb-2">
      <nav
        className={`mx-auto max-w-[70rem] flex h-14 items-center justify-between rounded-full px-4 transition-all duration-300 ${
          scrolled ? 'border border-border/50 bg-background/70 backdrop-blur-xl' : 'bg-transparent'
        }`}
      >
        <div className="flex items-center gap-2">
          <Link href="/" className="flex items-center gap-2.5">
            <Image
              src="/brandkit/Logo/Brandmark/SVG/Brandmark White.svg"
              alt="Kortix"
              width={18}
              height={16}
              className="h-4 w-auto hidden dark:block"
            />
            <Image
              src="/brandkit/Logo/Brandmark/SVG/Brandmark Black.svg"
              alt="Kortix"
              width={18}
              height={16}
              className="h-4 w-auto dark:hidden"
            />
            <span className="text-xl font-semibold tracking-tight">agent-click</span>
            <div className="h-3.5 w-px bg-border/60 hidden sm:block" />
            <span className="text-[11px] text-muted-foreground/50 hidden sm:block">by Kortix</span>
          </Link>
        </div>

        <div className="flex items-center gap-2">
          <div className="hidden lg:block">
            <SearchButton />
          </div>
          <MobileSidebar />
          <ThemeToggle />
          <a
            href="https://github.com/kortix-ai/agent-click"
            className="flex h-7 w-7 items-center justify-center rounded-full text-muted-foreground transition-colors hover:text-foreground hover:bg-foreground/[0.05]"
          >
            <GithubIcon className="h-4 w-4" />
          </a>
        </div>
      </nav>
    </div>
  );
}

function GithubIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" className={className}>
      <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
    </svg>
  );
}
