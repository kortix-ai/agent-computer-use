'use client';

import { useState, useCallback } from 'react';
import { Copy, Check, FileText, FileCode, ChevronDown } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

function getPageContent(): { markdown: string; plain: string } {
  const main = document.querySelector('main');
  if (!main) return { markdown: '', plain: '' };

  const content = main.querySelector('div > *:first-child')?.parentElement;
  if (!content) return { markdown: '', plain: '' };

  const md: string[] = [];
  const plain: string[] = [];

  function walk(node: Element) {
    const tag = node.tagName.toLowerCase();

    // Skip nav elements, copy button itself, page-nav
    if (
      tag === 'nav' ||
      tag === 'button' ||
      node.closest('[data-copy-page]') ||
      node.classList.contains('page-nav')
    )
      return;

    if (tag === 'h1') {
      const text = node.textContent?.trim() || '';
      md.push(`# ${text}\n`);
      plain.push(text);
      plain.push('='.repeat(text.length));
      return;
    }
    if (tag === 'h2') {
      const text = node.textContent?.trim() || '';
      md.push(`\n## ${text}\n`);
      plain.push(`\n${text}`);
      plain.push('-'.repeat(text.length));
      return;
    }
    if (tag === 'h3') {
      const text = node.textContent?.trim() || '';
      md.push(`\n### ${text}\n`);
      plain.push(`\n${text}`);
      return;
    }
    if (tag === 'h4') {
      const text = node.textContent?.trim() || '';
      md.push(`\n#### ${text}\n`);
      plain.push(`\n${text}`);
      return;
    }

    if (tag === 'p') {
      const text = inlineToMd(node);
      md.push(`${text}\n`);
      plain.push(node.textContent?.trim() || '');
      return;
    }

    if (tag === 'pre' || node.querySelector('pre')) {
      const code = node.querySelector('code');
      const pre = node.querySelector('pre');
      const text = code?.textContent || pre?.textContent || node.textContent || '';
      const lang = code?.className?.match(/language-(\w+)/)?.[1] || '';
      md.push(`\n\`\`\`${lang}\n${text.trim()}\n\`\`\`\n`);
      plain.push(text.trim());
      return;
    }

    // Code blocks rendered by shiki (have dangerouslySetInnerHTML)
    if (node.querySelector("[class*='shiki']") || node.querySelector('pre code')) {
      const text = node.textContent?.trim() || '';
      md.push(`\n\`\`\`\n${text}\n\`\`\`\n`);
      plain.push(text);
      return;
    }

    if (tag === 'table') {
      const rows = node.querySelectorAll('tr');
      rows.forEach((row, i) => {
        const cells = row.querySelectorAll('th, td');
        const line = Array.from(cells)
          .map((c) => c.textContent?.trim() || '')
          .join(' | ');
        md.push(`| ${line} |`);
        plain.push(line);
        if (i === 0) {
          const sep = Array.from(cells)
            .map(() => '---')
            .join(' | ');
          md.push(`| ${sep} |`);
        }
      });
      md.push('');
      return;
    }

    if (tag === 'ul') {
      node.querySelectorAll(':scope > li').forEach((li) => {
        const text = li.textContent?.trim() || '';
        md.push(`- ${text}`);
        plain.push(`- ${text}`);
      });
      md.push('');
      return;
    }

    if (tag === 'ol') {
      node.querySelectorAll(':scope > li').forEach((li, i) => {
        const text = li.textContent?.trim() || '';
        md.push(`${i + 1}. ${text}`);
        plain.push(`${i + 1}. ${text}`);
      });
      md.push('');
      return;
    }

    if (tag === 'blockquote') {
      const text = node.textContent?.trim() || '';
      md.push(`> ${text}\n`);
      plain.push(text);
      return;
    }

    if (tag === 'hr') {
      md.push('\n---\n');
      plain.push('---');
      return;
    }

    // Recurse into divs and other containers
    if (tag === 'div' || tag === 'section' || tag === 'article') {
      Array.from(node.children).forEach(walk);
      return;
    }
  }

  Array.from(content.children).forEach(walk);

  return {
    markdown: md
      .join('\n')
      .replace(/\n{3,}/g, '\n\n')
      .trim(),
    plain: plain
      .join('\n')
      .replace(/\n{3,}/g, '\n\n')
      .trim(),
  };
}

function inlineToMd(node: Element): string {
  let result = '';
  node.childNodes.forEach((child) => {
    if (child.nodeType === Node.TEXT_NODE) {
      result += child.textContent || '';
    } else if (child.nodeType === Node.ELEMENT_NODE) {
      const el = child as Element;
      const tag = el.tagName.toLowerCase();
      if (tag === 'code') {
        result += `\`${el.textContent}\``;
      } else if (tag === 'a') {
        const href = el.getAttribute('href') || '';
        result += `[${el.textContent}](${href})`;
      } else if (tag === 'strong' || tag === 'b') {
        result += `**${el.textContent}**`;
      } else if (tag === 'em' || tag === 'i') {
        result += `*${el.textContent}*`;
      } else {
        result += el.textContent || '';
      }
    }
  });
  return result;
}

export function CopyPage() {
  const [copied, setCopied] = useState<string | null>(null);

  const copy = useCallback((format: 'md' | 'text') => {
    const { markdown, plain } = getPageContent();
    const content = format === 'md' ? markdown : plain;
    navigator.clipboard.writeText(content);
    setCopied(format);
    setTimeout(() => setCopied(null), 2000);
  }, []);

  return (
    <div data-copy-page>
      <DropdownMenu>
        <DropdownMenuTrigger className="flex items-center gap-1.5 rounded-lg border border-border/50 px-2.5 py-1.5 text-muted-foreground transition-colors hover:text-foreground hover:border-border hover:bg-muted/50 outline-none">
          {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
          {copied ? 'Copied!' : 'Copy page'}
          <ChevronDown className="h-3 w-3" />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-44">
          <DropdownMenuItem onClick={() => copy('md')} className="gap-2 h-8">
            <FileCode className="h-3.5 w-3.5" />
            Copy as Markdown
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => copy('text')} className="gap-2 h-8">
            <FileText className="h-3.5 w-3.5" />
            Copy as plain text
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
