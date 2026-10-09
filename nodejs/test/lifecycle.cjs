'use strict';
const path = require('node:path');
const { createAudioStream } = require('..');
const controller = new AbortController();
const stream = createAudioStream('а '.repeat(4000), {
  binaryPath: process.env.ROZM_TEST_BINARY,
  dataDir: path.resolve(__dirname, '../../orig'),
  signal: controller.signal,
});
let acted = false;
stream.on('error', error => {
  if (process.argv[2] !== 'abort' || error.code !== 'ABORT_ERR') {
    console.error(error);
    process.exitCode = 1;
  }
});
stream.once('readable', () => {
  acted = true;
  if (process.argv[2] === 'abort') controller.abort();
  else stream.destroy();
});
process.on('beforeExit', () => {
  if (!acted || !stream.destroyed) {
    console.error('Stream was not cancelled');
    process.exitCode = 1;
  }
});
