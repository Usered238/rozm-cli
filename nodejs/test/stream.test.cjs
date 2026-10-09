const test = require('node:test');
const assert = require('node:assert/strict');
const { readFile } = require('node:fs/promises');
const path = require('node:path');
const { Readable, Writable } = require('node:stream');
const { pipeline } = require('node:stream/promises');
const { spawn } = require('node:child_process');

const root = path.resolve(__dirname, '../..');
const binaryPath = process.env.ROZM_TEST_BINARY || path.join(root, 'target/debug',
  process.platform === 'win32' ? 'rozm-cli.exe' : 'rozm-cli');
const dataDir = path.join(root, 'orig');

async function collect(stream) {
  const chunks = [];
  for await (const chunk of stream) chunks.push(chunk);
  return Buffer.concat(chunks);
}

test('Node.js returns a Readable containing the original WAV', { timeout: 30000 }, async () => {
  const { createAudioStream } = require('..');
  const stream = createAudioStream('Привіт', { binaryPath, dataDir });
  assert.ok(stream instanceof Readable);
  const actual = await collect(stream);
  const expected = await readFile(path.join(root, 'tests/fixtures/original-pryvit-v1-s5.wav'));
  assert.ok(actual.equals(expected));
});

test('synthesis errors reject the stream with native diagnostics and exit status', { timeout: 30000 }, async () => {
  const { createAudioStream } = require('..');
  for (const [text, options, message] of [
    ['а🙂', {}, /U\+1F642/],
    ['а\0б', {}, /U\+0000/],
    ['а', { dataDir: path.join(root, 'target/nonexistent-node-resources') }, /BSNbn/],
  ]) {
    await assert.rejects(collect(createAudioStream(text, { binaryPath, dataDir, ...options })), error => {
      assert.equal(error.code, 'ROZM_EXIT');
      assert.equal(error.exitCode, 1);
      assert.match(error.message, message);
      assert.match(error.stderr, message);
      return true;
    });
  }
  await assert.rejects(collect(createAudioStream('а', {
    binaryPath: path.join(root, 'target/nonexistent-rozm-executable'), dataDir,
  })), { code: 'ENOENT' });
});

test('AbortSignal cancels an active or not-yet-started stream', { timeout: 30000 }, async () => {
  const { createAudioStream } = require('..');
  for (const alreadyAborted of [false, true]) {
    const controller = new AbortController();
    if (alreadyAborted) controller.abort();
    const stream = createAudioStream('а '.repeat(4000), {
      binaryPath, dataDir, signal: controller.signal,
    });
    const result = assert.rejects(collect(stream), { name: 'AbortError', code: 'ABORT_ERR' });
    if (!alreadyAborted) controller.abort();
    await result;
  }
});

test('invalid API arguments fail synchronously before starting synthesis', () => {
  const { createAudioStream } = require('..');
  for (const text of [null, 42, Buffer.from('a'), '\ud800', '\udfff', 'а'.repeat(16385)]) {
    assert.throws(() => createAudioStream(text, { binaryPath, dataDir }), /text/);
  }
  for (const options of [
    null, [], { format: 'mp3' }, { voice: 4 }, { voice: '1' },
    { speed: 0 }, { speed: 1.5 }, { binaryPath: '' },
    { dataDir: 42 }, { signal: {} }, { unexpected: true },
  ]) {
    assert.throws(() => createAudioStream('а', options));
  }
});

test('voice, speed and ESM import preserve original audio', { timeout: 30000 }, async () => {
  const { createAudioStream } = await import('../index.cjs');
  for (const [voice, speed, fixture] of [[1, 1, 'a-v1-s1'], [1, 9, 'a-v1-s9'], [2, 5, 'a-v2-s5'], [3, 5, 'a-v3-s5']]) {
    const actual = await collect(createAudioStream('а', { binaryPath, dataDir, voice, speed, format: 'wav' }));
    assert.ok(actual.equals(await readFile(path.join(root, `tests/fixtures/original-${fixture}.wav`))));
  }
});

test('stdin carries quotes, newlines and text at the UTF-8 size limit', { timeout: 30000 }, async () => {
  const { createAudioStream } = require('..');
  const one = await readFile(path.join(root, 'tests/fixtures/original-a-v1-s5.wav'));
  const long = await collect(createAudioStream(' '.repeat(32766) + 'а', { binaryPath, dataDir }));
  assert.equal(long.subarray(0, 4).toString(), 'RIFF');
  assert.equal(long.readUInt32LE(4), long.length - 8);
  assert.equal(long.readUInt32LE(40), long.length - 44);
  assert.ok(long.length > 44);
  assert.ok((await collect(createAudioStream('"а"', { binaryPath, dataDir }))).equals(one));
  for (const separator of ['\n', '\r\n', '\r']) {
    const actual = await collect(createAudioStream('"а"' + separator + 'а', { binaryPath, dataDir }));
    assert.equal(actual.readUInt32LE(40), 2 * one.readUInt32LE(40));
    assert.ok(actual.subarray(44).equals(Buffer.concat([one.subarray(44), one.subarray(44)])));
  }
});

test('pipeline tolerates a slow consumer and concurrent synthesis', { timeout: 30000 }, async () => {
  const { createAudioStream } = require('..');
  const consume = async text => {
    const chunks = [];
    await pipeline(createAudioStream(text, { binaryPath, dataDir }), new Writable({
      highWaterMark: 1,
      write(chunk, encoding, callback) { chunks.push(Buffer.from(chunk)); setTimeout(callback, 2); },
    }));
    return Buffer.concat(chunks);
  };
  const [clauses, greeting] = await Promise.all([consume('а\n'.repeat(256)), consume('Привіт, світе!')]);
  const original = await readFile(path.join(root, 'tests/fixtures/original-a-v1-s5.wav'));
  const pcm = Buffer.concat(Array(256).fill(original.subarray(44)));
  assert.equal(clauses.readUInt32LE(40), pcm.length);
  assert.equal(clauses.readUInt32LE(4), clauses.length - 8);
  assert.ok(clauses.subarray(44).equals(pcm));
  assert.ok(greeting.equals(await readFile(path.join(root, 'tests/fixtures/original-greeting-v1-s5.wav'))));
});

test('destroy and abort release the native process even with a blocked consumer', { timeout: 30000 }, async () => {
  for (const mode of ['destroy', 'abort']) {
    const child = spawn(process.execPath, [path.join(__dirname, 'lifecycle.cjs'), mode], {
      env: { ...process.env, ROZM_TEST_BINARY: binaryPath },
      stdio: ['ignore', 'pipe', 'pipe'], timeout: 10000,
    });
    let stderr = '';
    child.stderr.on('data', chunk => { stderr += chunk; });
    child.stdout.resume();
    const [code, signal] = await new Promise((resolve, reject) => {
      child.once('error', reject);
      child.once('close', (code, signal) => resolve([code, signal]));
    });
    assert.equal(signal, null, stderr);
    assert.equal(code, 0, stderr);
  }
});
