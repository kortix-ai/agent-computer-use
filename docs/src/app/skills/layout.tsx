import { DocsLayout } from '@/components/docs-layout';
export default function Layout({ children }: { children: React.ReactNode }) {
  return <DocsLayout>{children}</DocsLayout>;
}
