import prompts from 'prompts';
import pc from 'picocolors';
import { writeFileSync, existsSync, mkdirSync } from 'fs';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const root = join(__dirname, '..');

// ── Word lists for random filenames ──────────────────────────────

const adjectives = [
  'cool',
  'red',
  'blue',
  'green',
  'bold',
  'calm',
  'dark',
  'fast',
  'gold',
  'hot',
  'icy',
  'keen',
  'lazy',
  'loud',
  'mild',
  'neat',
  'odd',
  'pale',
  'pink',
  'raw',
  'rich',
  'shy',
  'slim',
  'soft',
  'tall',
  'tiny',
  'warm',
  'wild',
  'wise',
  'witty',
  'brave',
  'crisp',
  'dry',
  'fair',
  'fresh',
  'fuzzy',
  'glad',
  'happy',
  'lucky',
  'moody',
  'quiet',
  'rough',
  'sharp',
  'sweet',
  'thick',
  'thin',
  'tough',
  'vast',
  'young',
];

const nouns = [
  'dogs',
  'cats',
  'birds',
  'foxes',
  'bears',
  'fish',
  'wolves',
  'owls',
  'bees',
  'ants',
  'deer',
  'ducks',
  'frogs',
  'goats',
  'hens',
  'jays',
  'koi',
  'larks',
  'mice',
  'newts',
  'pigs',
  'rams',
  'seals',
  'toads',
  'yaks',
  'crabs',
  'elms',
  'ferns',
  'gems',
  'hills',
  'inks',
  'jets',
  'keys',
  'lamps',
  'maps',
  'nets',
  'oaks',
  'pens',
  'rays',
  'suns',
  'tides',
  'vines',
  'waves',
  'clouds',
  'moths',
  'pines',
  'roads',
  'stars',
];

const verbs = [
  'fly',
  'run',
  'jump',
  'swim',
  'sing',
  'dance',
  'glow',
  'spin',
  'roar',
  'bark',
  'buzz',
  'clap',
  'dash',
  'drum',
  'fade',
  'grin',
  'hike',
  'knit',
  'lean',
  'melt',
  'nod',
  'peek',
  'race',
  'sigh',
  'trot',
  'wade',
  'yawn',
  'beam',
  'brew',
  'chat',
  'dig',
  'drip',
  'flip',
  'hum',
  'lift',
  'mix',
  'play',
  'rest',
  'roll',
  'snap',
  'soar',
  'tick',
  'turn',
  'wave',
  'wink',
];

function pick(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

function generateName() {
  return `${pick(adjectives)}-${pick(nouns)}-${pick(verbs)}`;
}

// ── Packages ─────────────────────────────────────────────────────

const packages = [
  { title: `agent-computer-use ${pc.dim('(the CLI)')}`, value: 'agent-computer-use' },
  { title: `docs ${pc.dim('(documentation site)')}`, value: 'docs' },
];

const bumpTypes = [
  {
    title: `${pc.green('patch')}  ${pc.dim('\u2014 bug fix, small tweak')}`,
    value: 'patch',
  },
  {
    title: `${pc.yellow('minor')}  ${pc.dim('\u2014 new feature, non-breaking')}`,
    value: 'minor',
  },
  {
    title: `${pc.red('major')}  ${pc.dim('\u2014 breaking change')}`,
    value: 'major',
  },
];

// ── Main ─────────────────────────────────────────────────────────

async function main() {
  console.log();
  console.log(pc.bold(pc.cyan('\u{1F4E6} agent-computer-use')) + pc.bold(' \u2014 New Changeset'));
  console.log(pc.dim('\u2500'.repeat(30)));
  console.log();

  const onCancel = () => {
    console.log();
    console.log(pc.dim('  Cancelled.'));
    console.log();
    process.exit(0);
  };

  const { selectedPackages } = await prompts(
    {
      type: 'multiselect',
      name: 'selectedPackages',
      message: 'What packages were affected?',
      choices: packages,
      min: 1,
      hint: 'Space to select, Enter to confirm',
      instructions: false,
    },
    { onCancel },
  );

  const { bumpType } = await prompts(
    {
      type: 'select',
      name: 'bumpType',
      message: 'What type of change?',
      choices: bumpTypes,
      hint: 'Controls version bump',
    },
    { onCancel },
  );

  const { summary } = await prompts(
    {
      type: 'text',
      name: 'summary',
      message: 'Describe the change (shown in CHANGELOG):',
      validate: (v) => (v.trim().length === 0 ? 'Please enter a description' : true),
    },
    { onCancel },
  );

  // ── Build changeset file ────────────────────────────────────

  const frontmatter = selectedPackages.map((pkg) => `"${pkg}": ${bumpType}`).join('\n');

  const content = `---\n${frontmatter}\n---\n\n${summary.trim()}\n`;

  // ── Write file ──────────────────────────────────────────────

  const changesetDir = join(root, '.changeset');
  if (!existsSync(changesetDir)) {
    mkdirSync(changesetDir, { recursive: true });
  }

  let name = generateName();
  while (existsSync(join(changesetDir, `${name}.md`))) {
    name = generateName();
  }

  const filePath = join(changesetDir, `${name}.md`);
  writeFileSync(filePath, content, 'utf-8');

  // ── Summary ─────────────────────────────────────────────────

  console.log();
  console.log(pc.green(pc.bold('\u2714 Changeset created!')));
  console.log(pc.dim(`  .changeset/${name}.md`));
  console.log();

  console.log(
    pc.dim(
      '  \u250C\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500',
    ),
  );
  for (const pkg of selectedPackages) {
    const color = bumpType === 'major' ? pc.red : bumpType === 'minor' ? pc.yellow : pc.green;
    console.log(pc.dim('  \u2502 ') + pc.bold(pkg) + pc.dim(' \u2192 ') + color(bumpType));
  }
  console.log(pc.dim('  \u2502'));
  console.log(pc.dim('  \u2502 ') + pc.italic(summary.trim()));
  console.log(
    pc.dim(
      '  \u2514\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500\u2500',
    ),
  );
  console.log();
}

main().catch((err) => {
  console.error(pc.red(err.message));
  process.exit(1);
});
