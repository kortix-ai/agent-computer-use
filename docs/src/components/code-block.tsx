'use client';

import { useEffect, useState } from 'react';
import { useTheme } from 'next-themes';
import { Check, Copy } from 'lucide-react';
import { codeToHtml } from 'shiki';
import { cn } from '@/lib/utils';

export function CodeBlock({
  children,
  title,
  lang = 'bash',
  minimal = true,
}: {
  children: string;
  title?: string;
  lang?: string;
  minimal?: boolean;
}) {
  const [copied, setCopied] = useState(false);
  const [html, setHtml] = useState<string>('');
  const { resolvedTheme } = useTheme();

  useEffect(() => {
    const theme = resolvedTheme === 'dark' ? 'andromeeda' : 'github-light';
    codeToHtml(children.trim(), { lang, theme }).then(setHtml);
  }, [children, lang, resolvedTheme]);

  const copy = () => {
    navigator.clipboard.writeText(children.trim());
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div
      className={cn(
        'group relative flex flex-col p-1 rounded-lg border border-border bg-muted/60 overflow-hidden my-4',
        minimal && 'border-none p-0',
      )}
    >
      {title && (
        <div className="flex items-center justify-between px-2 pb-2 pt-1">
          <span className="text-xs font-medium text-muted-foreground">{title}</span>
        </div>
      )}
      <div className="w-full p-2 rounded-lg dark:bg-background/60 bg-muted-foreground/5 border">
        {html ? (
          <div
            className="overflow-x-auto p-2 text-[13px] leading-relaxed [&_pre]:!bg-transparent [&_code]:!bg-transparent"
            dangerouslySetInnerHTML={{ __html: html }}
          />
        ) : (
          <div className="overflow-x-auto p-2 text-[13px] leading-relaxed">
            <pre className="font-mono text-foreground/90">{children.trim()}</pre>
          </div>
        )}
        <button
          onClick={copy}
          className="absolute right-3 top-2 rounded-md p-1.5 text-muted-foreground opacity-0 transition-opacity hover:bg-muted hover:text-foreground group-hover:opacity-100"
        >
          {copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
        </button>
      </div>
    </div>
  );
}
