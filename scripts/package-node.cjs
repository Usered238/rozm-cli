'use strict';

// Assemble the npm runtime from previously built native executables and data.
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const dataRoot = path.resolve(process.argv[2] || path.join(root, 'orig'));
const files = ['BSNbn', 'skfs/skf', 'skfs/wif', 'skfs/wlf',
  ...[1, 2, 3].flatMap(voice => ['ip2f', 'lp2f', 'Sd2f'].map(file => `snf${voice}/${file}`))];
const targets = [
  ['linux-x64', 'target/linux-build/release/rozm-cli', 'rozm-cli'],
  ['win32-ia32', 'target/i686-pc-windows-msvc/release/rozm-cli.exe', 'rozm-cli.exe'],
];
// Verify every source before changing the package directory.
for (const file of files) {
  if (!fs.statSync(path.join(dataRoot, file)).isFile()) throw new Error(`Missing resource: ${file}`);
}
for (const [, binary] of targets) {
  if (!fs.statSync(path.join(root, binary)).isFile()) throw new Error(`Build the release binary first: ${binary}`);
}
for (const [target, source, name] of targets) {
  const destination = path.join(root, 'nodejs/bin', target);
  fs.mkdirSync(destination, { recursive: true });
  const binary = path.join(destination, name);
  fs.copyFileSync(path.join(root, source), binary);
  fs.chmodSync(binary, 0o755);
  for (const file of files) {
    const output = path.join(destination, 'data', file);
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.copyFileSync(path.join(dataRoot, file), output);
  }
  console.log(`Packaged Node.js runtime: ${target}`);
}
