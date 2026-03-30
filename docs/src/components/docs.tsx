import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';

export function PageHeader({ title, description }: { title: string; description?: string }) {
  return (
    <div className="mb-8 border-b border-border pb-6">
      <h1 className="text-3xl mb-2 font-bold tracking-tight">{title}</h1>
      {description && <p className="mt-2 text-lg text-muted-foreground">{description}</p>}
    </div>
  );
}

function slugify(text: ReactNode): string {
  return String(text)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '');
}

export function H2({ children, id }: { children: ReactNode; id?: string }) {
  const slug = id || slugify(children);
  return (
    <h2 id={slug} className="mt-10 mb-4 text-xl font-semibold tracking-tight scroll-mt-20">
      {children}
    </h2>
  );
}

export function H3({ children, id }: { children: ReactNode; id?: string }) {
  const slug = id || slugify(children);
  return (
    <h3 id={slug} className="mt-8 mb-3 text-lg font-semibold tracking-tight scroll-mt-20">
      {children}
    </h3>
  );
}

export function P({ children }: { children: ReactNode }) {
  return <p className="mb-4 text-[15px] leading-relaxed text-muted-foreground">{children}</p>;
}

export function InlineCode({ children }: { children: ReactNode }) {
  return (
    <code className="rounded bg-muted px-1.5 py-0.5 text-[13px] font-medium font-mono text-foreground">
      {children}
    </code>
  );
}

export function CardGrid({ children }: { children: ReactNode }) {
  return <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 my-6">{children}</div>;
}

export function LinkCard({
  href,
  title,
  description,
  icon: Icon,
}: {
  href: string;
  title: string;
  description: string;
  icon?: React.ComponentType<{ className?: string }>;
}) {
  return (
    <a
      href={href}
      className="group flex flex-col gap-1.5 rounded-lg border border-border p-4 transition-colors hover:border-foreground/20 hover:bg-muted/50"
    >
      <div className="flex items-center gap-2">
        {Icon && <Icon className="h-4 w-4 text-muted-foreground group-hover:text-foreground" />}
        <span className="text-sm font-medium group-hover:text-foreground">{title}</span>
      </div>
      <span className="text-xs text-muted-foreground">{description}</span>
    </a>
  );
}

export function Table({ headers, rows }: { headers: string[]; rows: (string | ReactNode)[][] }) {
  return (
    <div className="my-6 overflow-x-auto rounded-lg border border-border">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-border bg-muted/50">
            {headers.map((h) => (
              <th
                key={h}
                className="px-4 py-2.5 text-left text-xs font-medium uppercase tracking-wider text-muted-foreground"
              >
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, i) => (
            <tr key={i} className="border-b border-border/50 last:border-0">
              {row.map((cell, j) => (
                <td key={j} className="px-4 py-2.5 text-[13px]">
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function Step({
  number,
  title,
  children,
}: {
  number: number;
  title: string;
  children: ReactNode;
}) {
  return (
    <div className="relative mb-8 pl-6 border-l border-border/60">
      <div className="absolute -left-[9px] top-0 flex h-[18px] w-[18px] items-center justify-center rounded-full bg-muted border border-border text-[10px] font-medium text-muted-foreground">
        {number}
      </div>
      <h4 className="mb-1 text-sm font-medium">{title}</h4>
      <div className="text-sm text-muted-foreground">{children}</div>
    </div>
  );
}
