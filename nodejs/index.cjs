'use strict';

const { spawn } = require('node:child_process');
const { PassThrough } = require('node:stream');
const path = require('node:path');

/** Return a WAV Readable; each call owns one native synthesis process. */
function createAudioStream(text, options = {}) {
  if (typeof text !== 'string' || !text.isWellFormed()) {
    throw new TypeError('text must be a well-formed Unicode string');
  }
  if (Buffer.byteLength(text, 'utf8') > 32768) {
    throw new RangeError('text exceeds 32768 UTF-8 bytes');
  }
  if (options === null || typeof options !== 'object' || Array.isArray(options)) {
    throw new TypeError('options must be an object');
  }
  for (const key of Object.keys(options)) {
    if (!['voice', 'speed', 'format', 'binaryPath', 'dataDir', 'signal'].includes(key)) {
      throw new TypeError(`unknown option ${key}`);
    }
  }
  if (options.format !== undefined && options.format !== 'wav') {
    throw new RangeError('only WAV export is available');
  }
  for (const [key, maximum] of [['voice', 3], ['speed', 9]]) {
    if (options[key] !== undefined && (!Number.isInteger(options[key]) || options[key] < 1 || options[key] > maximum)) {
      throw new RangeError(`${key} must be an integer in 1..${maximum}`);
    }
  }
  for (const key of ['binaryPath', 'dataDir']) {
    if (options[key] !== undefined && (typeof options[key] !== 'string' || !options[key].length || !options[key].isWellFormed() || options[key].includes('\0'))) {
      throw new TypeError(`${key} must be a nonempty path string without NUL`);
    }
  }
  if (options.signal !== undefined && !(options.signal instanceof AbortSignal)) {
    throw new TypeError('signal must be an AbortSignal');
  }
  if (!options.binaryPath && !((process.platform === 'linux' && process.arch === 'x64') ||
      (process.platform === 'win32' && ['ia32', 'x64'].includes(process.arch)))) {
    throw new Error('No bundled binary for this platform; supply binaryPath');
  }
  const binaryPath = options.binaryPath || path.join(__dirname, 'bin',
    process.platform === 'win32' ? 'win32-ia32' : 'linux-x64',
    process.platform === 'win32' ? 'rozm-cli.exe' : 'rozm-cli');
  const env = { ...process.env };
  if (options.dataDir !== undefined) env.ROZM_DATA_DIR = path.resolve(options.dataDir);
  const output = new PassThrough();
  const signal = options.signal;
  const abort = () => {
    const error = new Error('Audio synthesis aborted', { cause: signal.reason });
    error.name = 'AbortError';
    error.code = 'ABORT_ERR';
    output.destroy(error);
  };
  if (signal?.aborted) {
    process.nextTick(abort);
    return output;
  }
  const child = spawn(path.resolve(binaryPath), ['--stdin', '--output', '-',
    '--voice', String(options.voice ?? 1), '--speed', String(options.speed ?? 5)],
  { shell: false, windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'], env });
  let stdoutEnded = false;
  let processClosed = false;
  let succeeded = false;
  const diagnostics = [];
  let diagnosticBytes = 0;
  const finish = () => { if (stdoutEnded && succeeded) output.end(); };
  child.stdout.pipe(output, { end: false });
  child.stdout.once('end', () => { stdoutEnded = true; finish(); });
  child.once('error', error => output.destroy(error));
  child.once('close', (code, signal) => {
    processClosed = true;
    if (code === 0) { succeeded = true; finish(); }
    else {
      const stderr = Buffer.concat(diagnostics).toString('utf8').trim();
      const error = new Error(stderr || `rozm-cli exited with code ${code}, signal ${signal}`);
      error.code = 'ROZM_EXIT';
      error.exitCode = code;
      error.signal = signal;
      error.stderr = stderr;
      output.destroy(error);
    }
  });
  // Always drain stderr, retaining at most 16 KiB of diagnostics.
  child.stderr.on('data', chunk => {
    const retained = chunk.subarray(0, Math.max(0, 16384 - diagnosticBytes));
    if (retained.length) diagnostics.push(retained);
    diagnosticBytes += retained.length;
  });
  child.stdout.once('error', error => output.destroy(error));
  child.stderr.once('error', error => output.destroy(error));
  output.once('close', () => {
    signal?.removeEventListener('abort', abort);
    if (!processClosed) child.kill();
    child.stdin.destroy();
    child.stdout.destroy();
    child.stderr.destroy();
  });
  signal?.addEventListener('abort', abort, { once: true });
  child.stdin.on('error', error => { if (error.code !== 'EPIPE') output.destroy(error); });
  child.stdin.end(text, 'utf8');
  return output;
}

exports.createAudioStream = createAudioStream;
