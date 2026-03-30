'use client';

import { useState } from 'react';
import { CodeBlock } from './code-block';

const managers = [
  { id: 'npm', label: 'npm', cmd: 'npm install -g agent-click' },
  { id: 'pnpm', label: 'pnpm', cmd: 'pnpm add -g agent-click' },
  { id: 'yarn', label: 'yarn', cmd: 'yarn global add agent-click' },
  { id: 'bun', label: 'bun', cmd: 'bun add -g agent-click' },
];

export function InstallTabs() {
  const [active, setActive] = useState('npm');
  const current = managers.find((m) => m.id === active)!;

  return (
    <div className="my-4">
      <div className="flex items-center gap-1 mb-0 bg-muted/60 rounded-t-lg border border-b-0 border-border px-1.5 py-2">
        {managers.map((m) => (
          <button
            key={m.id}
            onClick={() => setActive(m.id)}
            className={`rounded-md w-12 text-[12px] font-medium transition-all ${
              active === m.id
                ? 'text-foreground'
                : 'text-muted-foreground/60 hover:text-muted-foreground'
            }`}
          >
            {m.label}
          </button>
        ))}
      </div>
      <div className="[&>div]:my-0 [&>div]:rounded-t-none [&>div]:border-t-0">
        <CodeBlock minimal={false}>{current.cmd}</CodeBlock>
      </div>
    </div>
  );
}
