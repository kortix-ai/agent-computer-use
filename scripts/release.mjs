import prompts from 'prompts';
import pc from 'picocolors';
import { readdirSync, readFileSync } from 'fs';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import { execSync } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const root = join(__dirname, '..');
const changesetDir = join(root, '.changeset');

// ── Parse changesets ─────────────────────────────────────────────

function getChangesets() {
  const files = readdirSync(changesetDir).filter((f) => f.endsWith('.md') && f !== 'README.md');

  return files.map((file) => {
    const raw = readFileSync(join(changesetDir, file), 'utf-8');
    const match = raw.match(/^---\n([\s\S]*?)\n---\n\n?([\s\S]*)$/);

    if (!match) return { file, packages: [], summary: raw.trim() };

    const frontmatter = match[1];
    const summary = match[2].trim();

    const packages = frontmatter
      .split('\n')
      .filter((l) => l.trim())
      .map((line) => {
        const m = line.match(/"([^"]+)":\s*(patch|minor|major)/);
        if (!m) return null;
        return { name: m[1], bump: m[2] };
      })
      .filter(Boolean);

    return { file: file.replace('.md', ''), packages, summary };
  });
}

function bumpColor(bump) {
  if (bump === 'major') return pc.red(bump);
  if (bump === 'minor') return pc.yellow(bump);
  return pc.green(bump);
}

function run(cmd, opts = {}) {
  try {
    execSync(cmd, {
      cwd: root,
      stdio: opts.capture ? 'pipe' : 'inherit',
      encoding: 'utf-8',
    });
    return true;
  } catch (err) {
    if (opts.capture) {
      console.log(err.stdout || '');
      console.error(err.stderr || '');
    }
    return false;
  }
}

// ── Main ─────────────────────────────────────────────────────────

async function main() {
  console.log();
  console.log(pc.bold(pc.cyan('\u{1F680} agent-computer-use')) + pc.bold(' \u2014 Release'));
  console.log(pc.dim('\u2500'.repeat(24)));
  console.log();

  const changesets = getChangesets();

  if (changesets.length === 0) {
    console.log(pc.dim('  No pending changesets found.'));
    console.log(pc.dim('  Run ') + pc.bold('pnpm changeset') + pc.dim(' to create one.'));
    console.log();
    return;
  }

  // ── List pending changesets ───────────────────────────────────

  console.log(pc.bold('  Pending changesets:'));
  console.log();

  for (const cs of changesets) {
    const pkgInfo = cs.packages
      .map((p) => `${pc.bold(p.name)} ${pc.dim('(')}${bumpColor(p.bump)}${pc.dim(')')}`)
      .join(pc.dim(', '));

    console.log(
      pc.dim('  \u2022 ') +
        pc.dim(cs.file + ': ') +
        pkgInfo +
        (cs.summary ? pc.dim(' \u2014 ') + cs.summary : ''),
    );
  }

  console.log();

  // ── Action prompt ─────────────────────────────────────────────

  const onCancel = () => {
    console.log();
    console.log(pc.dim('  Cancelled.'));
    console.log();
    process.exit(0);
  };

  const { action } = await prompts(
    {
      type: 'select',
      name: 'action',
      message: 'How would you like to proceed?',
      choices: [
        {
          title: `${pc.green('Apply versions')} ${pc.dim('\u2014 update package versions from changesets')}`,
          value: 'version',
        },
        {
          title: `${pc.blue('Publish to npm')} ${pc.dim('\u2014 build, bundle & publish')}`,
          value: 'publish',
        },
        {
          title: `${pc.yellow('Preview changes')} ${pc.dim('\u2014 dry run, no changes made')}`,
          value: 'preview',
        },
        {
          title: pc.dim('Cancel'),
          value: 'cancel',
        },
      ],
    },
    { onCancel },
  );

  console.log();

  // ── Execute action ────────────────────────────────────────────

  if (action === 'cancel') {
    console.log(pc.dim('  Cancelled.'));
    console.log();
    return;
  }

  if (action === 'preview') {
    console.log(pc.bold(pc.yellow('  \u25B6 Running dry run...')));
    console.log();
    const ok = run('npx changeset status --verbose');
    console.log();
    if (ok) {
      console.log(pc.green(pc.bold('  \u2714 Preview complete.')));
    } else {
      console.log(pc.red(pc.bold('  \u2716 Preview failed.')));
    }
    console.log();
    return;
  }

  if (action === 'version') {
    console.log(pc.bold(pc.cyan('  \u25B6 Applying versions...')));
    console.log();
    const ok = run('npx changeset version');
    console.log();
    if (ok) {
      console.log(pc.green(pc.bold('  \u2714 Versions updated!')));
      console.log();
      console.log(
        pc.dim('  Commit the changes and run ') +
          pc.bold('pnpm release:publish') +
          pc.dim(' when ready.'),
      );
    } else {
      console.log(pc.red(pc.bold('  \u2716 Version update failed.')));
    }
    console.log();
    return;
  }

  if (action === 'publish') {
    const { confirm } = await prompts(
      {
        type: 'confirm',
        name: 'confirm',
        message: `This will ${pc.bold('build')}, ${pc.bold('bundle')}, and ${pc.bold('publish')} to npm. Continue?`,
        initial: false,
      },
      { onCancel },
    );

    if (!confirm) {
      console.log();
      console.log(pc.dim('  Cancelled.'));
      console.log();
      return;
    }

    console.log();

    const steps = [
      { label: 'Building CLI', cmd: 'pnpm build' },
      { label: 'Bundling for npm', cmd: 'pnpm bundle' },
      { label: 'Publishing to npm', cmd: 'npx changeset publish' },
    ];

    for (const step of steps) {
      console.log(pc.bold(pc.cyan(`  \u25B6 ${step.label}...`)));
      const ok = run(step.cmd);
      if (!ok) {
        console.log();
        console.log(pc.red(pc.bold(`  \u2716 ${step.label} failed.`)));
        console.log();
        process.exit(1);
      }
      console.log(pc.green(`  \u2714 ${step.label} done.`));
      console.log();
    }

    console.log(pc.green(pc.bold('  \u2714 Published successfully!')));
    console.log();
  }
}

main().catch((err) => {
  console.error(pc.red(err.message));
  process.exit(1);
});
