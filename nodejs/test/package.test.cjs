const test = require('node:test');
const assert = require('node:assert/strict');
const { readFile, mkdtemp, readdir, rm } = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');

const root = path.resolve(__dirname, '../..');
const packagePath = process.env.ROZM_TEST_PACKAGE || path.resolve(__dirname, '..');

test('installed package selects its bundled binary and data independently of cwd', { timeout: 30000 }, async () => {
  const { createAudioStream } = require(packagePath);
  const cwd = process.cwd();
  const previous = process.env.ROZM_DATA_DIR;
  const directory = await mkdtemp(path.join(os.tmpdir(), 'rozm node проба '));
  try {
    delete process.env.ROZM_DATA_DIR;
    process.chdir(directory);
    const chunks = [];
    for await (const chunk of createAudioStream('Привіт, світе!')) chunks.push(chunk);
    assert.ok(Buffer.concat(chunks).equals(await readFile(path.join(root, 'tests/fixtures/original-greeting-v1-s5.wav'))));
    assert.deepEqual(await readdir(directory), []);
  } finally {
    process.chdir(cwd);
    if (previous === undefined) delete process.env.ROZM_DATA_DIR;
    else process.env.ROZM_DATA_DIR = previous;
    await rm(directory, { recursive: true, force: true });
  }
});
