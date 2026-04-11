import type { Metadata } from 'next';
import { ThemeProvider } from '@/components/theme-provider';
import { roobert } from './fonts/roobert';
import { roobertMono } from './fonts/roobert-mono';
import './globals.css';

export const metadata: Metadata = {
  title: 'agent-computer-use — Computer use CLI for AI agents',
  description:
    'Control any desktop app from the terminal. Click buttons, type text, read screens — all through the accessibility tree. Built in Rust.',
  keywords: [
    'Computer use',
    'AI agent',
    'computer use',
    'accessibility',
    'CLI',
    'desktop automation',
    'Rust',
    'Electron',
    'CDP',
    'screen reader',
    'agent-computer-use',
  ],
  metadataBase: new URL('https://agent-computer-use.dev'),
  openGraph: {
    title: 'agent-computer-use — Computer use CLI for AI agents',
    description:
      'Control any desktop app from the terminal. Click buttons, type text, read screens. Built in Rust.',
    url: 'https://agent-computer-use.dev',
    siteName: 'agent-computer-use',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'agent-computer-use — Computer use CLI for AI agents',
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
