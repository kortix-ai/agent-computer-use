import { ImageResponse } from 'next/og';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

export const alt = 'agent-click — Computer use CLI for AI agents';
export const size = { width: 1200, height: 630 };
export const contentType = 'image/png';

export default async function OGImage() {
  const fontData = await readFile(
    join(process.cwd(), 'public/fonts/roobert/RoobertUprightsVF.woff2'),
  );

  const logoSvg = await readFile(
    join(process.cwd(), 'public/brandkit/Logo/Brandmark/SVG/Brandmark White.svg'),
    'utf-8',
  );
  const logoDataUri = `data:image/svg+xml;base64,${Buffer.from(logoSvg).toString('base64')}`;

  return new ImageResponse(
    (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'center',
          alignItems: 'flex-start',
          padding: '80px 100px',
          backgroundColor: '#2b2b2b',
          fontFamily: 'Roobert',
          color: '#f0f0f0',
        }}
      >
        {/* Logo */}
        <img src={logoDataUri} width={72} height={62} alt="" />

        {/* Title */}
        <div
          style={{
            fontSize: 72,
            fontWeight: 700,
            letterSpacing: '-0.03em',
            marginTop: 40,
            lineHeight: 1.1,
          }}
        >
          agent-click
        </div>

        {/* Tagline */}
        <div
          style={{
            fontSize: 32,
            fontWeight: 400,
            color: '#999999',
            marginTop: 20,
            lineHeight: 1.4,
          }}
        >
          Computer use CLI for AI agents.
        </div>

        {/* Badges */}
        <div style={{ display: 'flex', gap: 12, marginTop: 40 }}>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              padding: '8px 20px',
              borderRadius: 999,
              backgroundColor: 'rgba(255,255,255,0.12)',
              fontSize: 18,
              fontWeight: 500,
              color: '#f0f0f0',
            }}
          >
            macOS — available now
          </div>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              padding: '8px 20px',
              borderRadius: 999,
              border: '1.5px solid rgba(255,255,255,0.2)',
              fontSize: 18,
              fontWeight: 500,
              color: '#999999',
            }}
          >
            Windows — coming soon
          </div>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              padding: '8px 20px',
              borderRadius: 999,
              border: '1.5px solid rgba(255,255,255,0.2)',
              fontSize: 18,
              fontWeight: 500,
              color: '#999999',
            }}
          >
            Linux — coming soon
          </div>
        </div>
      </div>
    ),
    {
      ...size,
      fonts: [
        {
          name: 'Roobert',
          data: fontData,
          style: 'normal',
        },
      ],
    },
  );
}
