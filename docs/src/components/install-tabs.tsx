'use client';

import { useState } from 'react';
import { CodeBlock } from './code-block';

type ManagerCommand = { id: string; label: string; cmd: string };

const installCommands: ManagerCommand[] = [
  { id: 'npm', label: 'npm', cmd: 'npm install -g @kortix-ai/agent-computer-use' },
  { id: 'pnpm', label: 'pnpm', cmd: 'pnpm add -g @kortix-ai/agent-computer-use' },
  { id: 'yarn', label: 'yarn', cmd: 'yarn global add @kortix-ai/agent-computer-use' },
  { id: 'bun', label: 'bun', cmd: 'bun add -g @kortix-ai/agent-computer-use' },
];

export function InstallTabs({ commands = installCommands }: { commands?: ManagerCommand[] }) {
  const [active, setActive] = useState(commands[0].id);
  const current = commands.find((m) => m.id === active) ?? commands[0];

  return (
    <div className="my-4">
      <div className="flex items-center gap-1 mb-0 bg-muted/60 rounded-t-lg border border-b-0 border-border px-1.5 py-2">
        {commands.map((m) => (
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
