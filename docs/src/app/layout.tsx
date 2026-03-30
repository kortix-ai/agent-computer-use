import type { Metadata } from 'next';
import { ThemeProvider } from '@/components/theme-provider';
import { roobert } from './fonts/roobert';
import { roobertMono } from './fonts/roobert-mono';
import './globals.css';

export const metadata: Metadata = {
  title: 'agent-click — Computer use CLI for AI agents',
  description:
    'Control any desktop app from the terminal. Click buttons, type text, read screens — all through the accessibility tree. Built in Rust for macOS. Windows and Linux coming soon.',
  keywords: [
    'Computer use',
    'AI agent',
    'computer use',
    'accessibility',
    'CLI',
    'macOS automation',
    'Rust',
    'Electron',
    'CDP',
    'screen reader',
    'agent-click',
  ],
  metadataBase: new URL('https://agent-click.dev'),
  openGraph: {
    title: 'agent-click — Computer use CLI for AI agents',
    description:
      'Control any desktop app from the terminal. Click buttons, type text, read screens. Built in Rust.',
    url: 'https://agent-click.dev',
    siteName: 'agent-click',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'agent-click — Computer use CLI for AI agents',
    description:
      'Control any desktop app from the terminal. Click buttons, type text, read screens. Built in Rust.',
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${roobert.variable} ${roobertMono.variable} antialiased`}
      suppressHydrationWarning
    >
      <body className="font-sans">
        <ThemeProvider>{children}</ThemeProvider>
      </body>
    </html>
  );
}
