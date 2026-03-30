import type { NextConfig } from 'next';
import createMDX from '@next/mdx';

const nextConfig: NextConfig = {
  pageExtensions: ['ts', 'tsx', 'md', 'mdx'],
  // Use webpack for MDX support (Turbopack doesn't serialize remark plugins)
  experimental: {},
};

const withMDX = createMDX({
  options: {
    remarkPlugins: [['remark-gfm']],
  },
});

export default withMDX(nextConfig);
