import type { MDXComponents } from 'mdx/types';
import { CodeBlock } from '@/components/code-block';

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '');
}

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    h1: ({ children }) => (
      <div className="mb-8 border-b border-border pb-6">
        <h1 className="text-xl mb-2 font-bold tracking-tight">{children}</h1>
      </div>
    ),
    h2: ({ children }) => {
      const id = slugify(String(children));
      return (
        <h2 id={id} className="mt-10 mb-4 text-xl font-semibold tracking-tight scroll-mt-20">
          {children}
        </h2>
      );
    },
    h3: ({ children }) => {
      const id = slugify(String(children));
      return (
        <h3 id={id} className="mt-8 mb-3 text-lg font-semibold tracking-tight scroll-mt-20">
          {children}
        </h3>
      );
    },
    p: ({ children }) => (
      <p className="mb-4 text-[15px] leading-relaxed text-muted-foreground">{children}</p>
    ),
    a: ({ href, children }) => (
      <a
        href={href}
        className="text-foreground underline underline-offset-4 hover:text-foreground/80"
        target={href?.startsWith('http') ? '_blank' : undefined}
        rel={href?.startsWith('http') ? 'noopener noreferrer' : undefined}
      >
        {children}
      </a>
    ),
    code: ({ children, className }) => {
      const isBlock = className?.startsWith('language-');
      if (isBlock) {
        const lang = className?.replace('language-', '') || 'bash';
        return <CodeBlock lang={lang}>{String(children)}</CodeBlock>;
      }
      return (
        <code className="rounded bg-muted px-1.5 py-0.5 text-[13px] font-medium font-mono text-foreground">
          {children}
        </code>
      );
    },
    pre: ({ children }) => {
      return <>{children}</>;
    },
    ul: ({ children }) => (
      <ul className="list-disc pl-6 space-y-1 text-sm text-muted-foreground my-3">{children}</ul>
    ),
    ol: ({ children }) => (
      <ol className="list-decimal pl-6 space-y-1 text-sm text-muted-foreground my-3">{children}</ol>
    ),
    li: ({ children }) => <li>{children}</li>,
    table: ({ children }) => (
      <div className="my-6 overflow-x-auto rounded-lg border border-border">
        <table className="w-full text-sm">{children}</table>
      </div>
    ),
    thead: ({ children }) => (
      <thead className="[&_tr]:border-b [&_tr]:border-border [&_tr]:bg-muted/50">{children}</thead>
    ),
    th: ({ children }) => (
      <th className="px-4 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground">
        {children}
      </th>
    ),
    tbody: ({ children }) => <tbody>{children}</tbody>,
    tr: ({ children }) => <tr className="border-b border-border/50 last:border-0">{children}</tr>,
    td: ({ children }) => <td className="px-4 py-2.5 text-[13px]">{children}</td>,
    blockquote: ({ children }) => (
      <blockquote className="border-l-2 border-border pl-4 my-4 text-sm text-muted-foreground italic">
        {children}
      </blockquote>
    ),
    hr: () => <hr className="my-8 border-border/50" />,
    ...components,
  };
}
