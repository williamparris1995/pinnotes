// Usage: node scripts/extract-changelog.mjs <version>
// Prints the CHANGELOG.md section for <version> (the body under "## <version>"
// up to the next "## " heading) to stdout. Used by release.yml to inline the
// current version's notes into the GitHub Release body (instead of a link).
import { readFileSync } from 'node:fs';

const ver = process.argv[2];
if (!ver) {
  console.error('extract-changelog: version argument required');
  process.exit(1);
}
const md = readFileSync('CHANGELOG.md', 'utf8');
const esc = ver.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const m = md.match(new RegExp(`##\\s*${esc}[^\\n]*\\n([\\s\\S]*?)(?=\\n##\\s|$)`));
process.stdout.write((m ? m[1].trim() : 'See CHANGELOG.md.') + '\n');
