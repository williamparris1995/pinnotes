// Render src-tauri/icons/icon.svg -> src-tauri/icons/icon-source.png (1024×1024)
// so `tauri icon` can generate the full platform icon set.
// Re-run after editing icon.svg:  npm run gen:app-icon
import { Resvg } from '@resvg/resvg-js';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '..');
const svgPath = join(root, 'src-tauri/icons/icon.svg');
const outPath = join(root, 'src-tauri/icons/icon-source.png');

const svg = readFileSync(svgPath, 'utf8');
const png = new Resvg(svg, { fitTo: { mode: 'width', value: 1024 } }).render().asPng();
writeFileSync(outPath, png);
console.log('wrote', outPath);
