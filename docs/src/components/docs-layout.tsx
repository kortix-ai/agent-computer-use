import type { ReactNode } from 'react';
import { Sidebar } from './sidebar';
import { Nav } from './nav';
import { Footer } from './footer';
import { TableOfContents } from './toc';
import { PageNav } from './page-nav';
import { CopyPage } from './copy-page';

export function DocsLayout({ children }: { children: ReactNode }) {
  return (
    <div className="min-h-screen flex flex-col">
      <Nav />
      <div className="flex-1 mx-auto w-full max-w-[72rem] flex">
        <Sidebar />
        <main className="flex-1 min-w-0">
          <div className="relative max-w-[680px] px-6 md:px-10 py-8 pb-20">
            <div className="absolute right-6 md:right-10 top-8">
              <CopyPage />
            </div>
            {children}
            <PageNav />
          </div>
        </main>
        <TableOfContents />
      </div>
      <Footer />
    </div>
  );
}
