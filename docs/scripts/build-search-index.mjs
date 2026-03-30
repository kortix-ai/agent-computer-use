import fs from 'fs';
import path from 'path';

const APP_DIR = path.join(process.cwd(), 'src/app');
const OUTPUT = path.join(process.cwd(), 'src/generated/search-index.json');

const routes = {
  'page.tsx': '/',
  'quickstart/page.mdx': '/quickstart',
  'installation/page.mdx': '/installation',
  'snapshots/page.mdx': '/snapshots',
  'selectors/page.mdx': '/selectors',
  'background/page.mdx': '/background',
  'commands/page.mdx': '/commands',
  'commands/click/page.mdx': '/commands/click',
  'commands/type/page.mdx': '/commands/type',
  'commands/snapshot/page.mdx': '/commands/snapshot',
  'workflows/page.mdx': '/workflows',
  'batch/page.mdx': '/batch',
  'ai-mode/page.mdx': '/ai-mode',
  'architecture/page.mdx': '/architecture',
  'options/page.mdx': '/options',
};

function slugify(text) {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '');
}

function extractSections(content, href) {
  const lines = content.split('\n');
  const sections = [];
  let currentTitle = '';
  let currentContent = [];
  let pageTitle = '';
  let inCodeBlock = false;

  // Extract page title from PageHeader JSX if present
  const headerMatch = content.match(/<PageHeader\s+title="([^"]+)"/);
  if (headerMatch) {
    pageTitle = headerMatch[1];
    currentTitle = pageTitle;
  }

  for (const line of lines) {
    // Track code blocks (both ``` and JSX <CodeBlock>)
    if (line.trim().startsWith('```')) {
      inCodeBlock = !inCodeBlock;
      continue;
    }
    if (line.trim().startsWith('<CodeBlock')) {
      inCodeBlock = true;
      continue;
    }
    if (line.trim().includes('</CodeBlock>') || line.trim().endsWith('}`}')) {
      inCodeBlock = false;
      continue;
    }
    if (inCodeBlock) continue;

    // Skip imports, exports, and JSX component tags
    if (line.startsWith('import ') || line.startsWith('export ')) continue;
    const trimmed = line.trim();
    if (trimmed.match(/^<\/?[A-Z]/) || trimmed.match(/^<[a-z]+\s.*\/>$/)) continue;

    const h1 = line.match(/^#\s+(.+)/);
    const h2 = line.match(/^##\s+(.+)/);
    const h3 = line.match(/^###\s+(.+)/);

    if (h1) {
      pageTitle = h1[1].trim();
      if (currentContent.length > 0) {
        sections.push({
          title: currentTitle || pageTitle,
          content: currentContent.join(' '),
          href,
          pageTitle,
        });
      }
      currentTitle = pageTitle;
      currentContent = [];
      continue;
    }

    if (h2 || h3) {
      if (currentContent.length > 0 || currentTitle) {
        sections.push({
          title: currentTitle || pageTitle,
          content: currentContent.join(' '),
          href: `${href}#${slugify((h2 || h3)[1])}`,
          pageTitle,
        });
      }
      currentTitle = (h2 || h3)[1].trim();
      currentContent = [];
      continue;
    }

    // Clean markdown formatting for search
    const cleaned = line
      .replace(/`([^`]+)`/g, '$1')
      .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
      .replace(/\*\*([^*]+)\*\*/g, '$1')
      .replace(/\*([^*]+)\*/g, '$1')
      .replace(/\|/g, ' ')
      .replace(/-{3,}/g, '')
      .trim();

    if (cleaned) {
      currentContent.push(cleaned);
    }
  }

  if (currentContent.length > 0 || currentTitle) {
    sections.push({
      title: currentTitle || pageTitle,
      content: currentContent.join(' '),
      href,
      pageTitle,
    });
  }

  return sections;
}

function extractFromTsx(content, href) {
  const strings = [];
  const titleMatch = content.match(/title="([^"]+)"/);
  const descMatch = content.match(/description="([^"]+)"/);
  if (titleMatch) strings.push(titleMatch[1]);
  if (descMatch) strings.push(descMatch[1]);

  const textMatches = content.matchAll(/>\s*([^<>{]+)\s*</g);
  for (const m of textMatches) {
    const text = m[1].trim();
    if (text && text.length > 5 && !text.startsWith('//') && !text.startsWith('{')) {
      strings.push(text);
    }
  }

  return [
    {
      title: titleMatch?.[1] || 'Home',
      content: strings.join(' '),
      href,
      pageTitle: titleMatch?.[1] || 'Home',
    },
  ];
}

const index = [];

for (const [file, href] of Object.entries(routes)) {
  const filePath = path.join(APP_DIR, file);
  if (!fs.existsSync(filePath)) continue;

  const content = fs.readFileSync(filePath, 'utf-8');

  if (file.endsWith('.mdx')) {
    index.push(...extractSections(content, href));
  } else {
    index.push(...extractFromTsx(content, href));
  }
}

const filtered = index.filter((s) => s.content.trim().length > 10);

fs.mkdirSync(path.dirname(OUTPUT), { recursive: true });
fs.writeFileSync(OUTPUT, JSON.stringify(filtered, null, 2));

console.log(
  `Built search index: ${filtered.length} sections from ${Object.keys(routes).length} pages`,
);
