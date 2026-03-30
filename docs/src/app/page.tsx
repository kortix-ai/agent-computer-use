import { DocsLayout } from '@/components/docs-layout';
import { CardGrid, LinkCard, Step } from '@/components/docs';
import { CodeBlock } from '@/components/code-block';
import { InstallTabs } from '@/components/install-tabs';
import { Rocket, Terminal, Zap, Search, FileText } from 'lucide-react';

export default function Home() {
  return (
    <DocsLayout>
      <div className="mb-8 border-b border-border pb-6">
        <h1 className="text-5xl mb-2 font-bold tracking-tight">agent-click</h1>
        <p className="mt-2 text-lg text-muted-foreground">Computer use CLI for AI Agents.</p>
        <div className="mt-3 flex gap-2">
          <span className="inline-flex items-center rounded-full bg-foreground/10 px-3 py-1 text-xs font-medium text-foreground">
            macOS — available now
          </span>
          <span className="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">
            Windows — coming soon
          </span>
          <span className="inline-flex items-center rounded-full border border-border px-3 py-1 text-xs font-medium text-muted-foreground">
            Linux — coming soon
          </span>
        </div>
      </div>

      <p className="mb-4 text-[15px] leading-relaxed text-muted-foreground">
        Agent-click lets you control desktop apps from the terminal. Click buttons, type into fields, read
        what's on screen. All using a single CLI.
      </p>

      <p className="mb-4 text-[15px] leading-relaxed text-muted-foreground">
        Built for AI agents. An agent can snapshot the screen, decide what to click, and act while
        you sit back and watch.{' '}
        <a
          href="https://github.com/kortix-ai/agent-click"
          className="text-foreground underline underline-offset-4"
        >
          Star it on GitHub
        </a>
      </p>

      <InstallTabs />

      <CardGrid>
        <LinkCard
          href="/quickstart"
          title="Quick Start"
          description="Install and try it in 2 minutes"
          icon={Rocket}
        />
        <LinkCard
          href="/commands"
          title="Commands"
          description="Everything agent-click can do"
          icon={Terminal}
        />
        <LinkCard
          href="/snapshots"
          title="Snapshots"
          description="See what's on screen, then act on it"
          icon={Zap}
        />
        <LinkCard
          href="/selectors"
          title="Selectors"
          description="Find buttons, fields, anything"
          icon={Search}
        />
        <LinkCard
          href="/workflows"
          title="Workflows"
          description="Chain steps into reusable scripts"
          icon={FileText}
        />
        <LinkCard
          href="/ai-mode"
          title="AI Agents"
          description="How agents use agent-click"
          icon={Terminal}
        />
      </CardGrid>

      <h2 className="mt-10 mb-4 text-xl font-semibold tracking-tight scroll-mt-20">How it works</h2>

      <p className="mb-4 text-[15px] leading-relaxed text-muted-foreground">
        agent-click reads the accessibility tree — the same structure screen readers use. It sees every
        button, text field, and menu item in any app. You point, it acts.
      </p>

      <div className="mt-6 mb-2">
        <Step number={1} title="Snapshot">
          Capture every interactive element. Each gets a ref.
          <CodeBlock>{`$ agent-click snapshot -a Calculator -i -c
[@e1] button "All Clear"   [@e5] button "7"
[@e8] button "Multiply"    [@e11] button "6"
[@e20] button "Equals"`}</CodeBlock>
        </Step>

        <Step number={2} title="Act">
          Use refs to click, type, or read.
          <CodeBlock>{`$ agent-click click @e5 && agent-click click @e8 && agent-click click @e11 && agent-click click @e20
$ agent-click text -a Calculator
42`}</CodeBlock>
        </Step>

        <Step number={3} title="Re-snapshot">
          UI changed? Snapshot again for fresh refs.
          <CodeBlock>{`$ agent-click snapshot -a Calculator -i -c`}</CodeBlock>
        </Step>
      </div>

      <h2 className="mt-10 mb-4 text-xl font-semibold tracking-tight scroll-mt-20">
        What can you do with it?
      </h2>

      <p className="mb-4 text-[15px] leading-relaxed text-muted-foreground">
        Anything you'd do by clicking around:
      </p>

      <div className="grid grid-cols-1 gap-2 my-4 text-sm">
        {[
          'Open Maps and search for the Colosseum',
          'Send a Slack message to a teammate',
          'Fill out a form in a browser',
          'Multiply numbers in Calculator',
          'Read the price of a flight from a booking site',
          'Scrape data from a desktop app into a spreadsheet',
          'Click through a setup wizard automatically',
          'Automate a multi-step workflow with a YAML file',
        ].map((item) => (
          <div
            key={item}
            className="flex items-center gap-2.5 rounded-lg border border-border/50 px-3 py-2.5"
          >
            <div className="h-1.5 w-1.5 rounded-full bg-foreground/30 shrink-0" />
            <span className="text-muted-foreground">{item}</span>
          </div>
        ))}
      </div>
    </DocsLayout>
  );
}
