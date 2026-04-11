import Image from 'next/image';

export function Footer() {
  return (
    <footer className="mt-auto">
      <div className="mx-auto max-w-[72rem] px-6 py-6">
        <div className="flex items-center justify-between text-[11px] text-muted-foreground/40">
          <div className="flex items-center gap-2">
            <Image
              src="/brandkit/Logo/Brandmark/SVG/Brandmark White.svg"
              alt="Kortix"
              width={10}
              height={9}
              className="h-[9px] w-auto hidden dark:block opacity-30"
            />
            <Image
              src="/brandkit/Logo/Brandmark/SVG/Brandmark Black.svg"
              alt="Kortix"
              width={10}
              height={9}
              className="h-[9px] w-auto dark:hidden opacity-30"
            />
            <span>agent-computer-use by Kortix</span>
            <span>&middot;</span>
            <span>MIT</span>
          </div>
          <div className="flex items-center gap-3">
            <a
              href="https://github.com/kortix-ai/agent-computer-use"
              className="hover:text-muted-foreground transition-colors"
            >
              GitHub
            </a>
            <a href="/changelog" className="hover:text-muted-foreground transition-colors">
              Changelog
            </a>
            <a
              href="https://www.npmjs.com/package/agent-computer-use"
              className="hover:text-muted-foreground transition-colors"
            >
              npm
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}
