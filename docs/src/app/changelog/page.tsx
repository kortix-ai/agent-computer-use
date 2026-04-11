'use client';

interface Release {
  date: string;
  version: string;
  summary: string;
  sections: {
    label: string;
    items: string[];
  }[];
}

const releases: Release[] = [
  {
    date: 'Mar 28, 2026',
    version: '0.3.0',
    summary: 'Windows backend, click reliability, new commands, MDX docs.',
    sections: [
      {
        label: 'New',
        items: [
          'Windows backend (UI Automation)',
          'press, get-value, scroll-to commands',
          'id~= partial ID matching, index=N selector',
          'Path-based ref resolution — instant element lookup',
          'Docs site with MDX, search, and mobile nav',
        ],
      },
      {
        label: 'Improved',
        items: [
          'Clicks send mouseMoved first (3-event sequence)',
          'Auto scroll-into-view before clicking',
          'type clears field by default (--append to keep)',
          'Shared element utilities in agent-computer-use-core',
        ],
      },
      {
        label: 'Fixed',
        items: [
          'Ambiguous selector on unnamed buttons',
          'Double-click timing (50ms between events)',
        ],
      },
    ],
  },
  {
    date: 'Mar 27, 2026',
    version: '0.2.0',
    summary: 'Snapshots, selectors, background clicks, workflows.',
    sections: [
      {
        label: 'New',
        items: [
          'Snapshot refs (@e1, @e2) for deterministic targeting',
          'Selector DSL with role, name, id, descendant chains',
          'Background clicks via accessibility API',
          'YAML workflows, batch execution, TUI explorer',
        ],
      },
    ],
  },
  {
    date: 'Mar 26, 2026',
    version: '0.1.0',
    summary: 'Initial release. macOS backend, core commands.',
    sections: [
      {
        label: 'New',
        items: [
          'macOS backend with accessibility tree + input simulation',
          'click, type, key, scroll, find, tree, text, screenshot',
          'JSON output for AI agents',
        ],
      },
    ],
  },
];

export default function Changelog() {
  return (
    <div>
      <div className="mb-10 border-b border-border pb-6">
        <h1 className="text-xl mb-2 font-bold tracking-tight">Changelog</h1>
        <p className="mt-2 text-sm text-muted-foreground">What shipped and when.</p>
      </div>

      <div className="space-y-0">
        {releases.map((release) => (
          <div key={release.version} className="relative flex gap-6 pb-10 last:pb-0">
            {/* Left column — sticky date */}
            <div className="hidden sm:block w-28 shrink-0">
              <div className="sticky top-20">
                <span className="text-xs text-muted-foreground/50">{release.date}</span>
              </div>
            </div>

            {/* Timeline */}
            <div className="relative flex flex-col items-center shrink-0">
              <div className="h-2 w-2 rounded-full bg-foreground/20 mt-1.5" />
              <div className="flex-1 w-px bg-border/30 mt-2" />
            </div>

            {/* Content */}
            <div className="flex-1 pb-2 min-w-0">
              <div className="flex items-baseline gap-2 flex-wrap">
                <span className="text-sm font-semibold">{release.version}</span>
                <span className="sm:hidden text-xs text-muted-foreground/50">{release.date}</span>
              </div>
              <p className="text-sm text-muted-foreground mt-1">{release.summary}</p>

              <div className="mt-4 space-y-3">
                {release.sections.map((section) => (
                  <div key={section.label}>
                    <span className="text-[11px] font-medium uppercase tracking-wider text-muted-foreground/40">
                      {section.label}
                    </span>
                    <ul className="mt-1 space-y-0.5">
                      {section.items.map((item, i) => (
                        <li
                          key={i}
                          className="flex items-start gap-2 text-[13px] text-muted-foreground leading-relaxed"
                        >
                          <span className="mt-2 h-1 w-1 rounded-full bg-muted-foreground/30 shrink-0" />
                          {item}
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
