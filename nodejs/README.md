# rozm-tts

Украинский TTS Rozm для Node.js 22+: `createAudioStream(text, options)` возвращает
стандартный `Readable` с WAV-байтами. Работает без рабочего стола и аудиоустройства.
В готовый npm-архив включены Rust-бинарники для Ubuntu 24.04 x86_64 и Windows
x86, а также голосовые данные. Установочных скриптов и npm-зависимостей нет.
На Windows x64 Node.js запускает 32-битный движок через WOW64.

```sh
npm install /path/to/rozm-tts-0.1.0.tgz
```

```js
// ESM; для CommonJS: const { createAudioStream } = require('rozm-tts');
import { createAudioStream } from 'rozm-tts';
import { createWriteStream } from 'node:fs';
import { pipeline } from 'node:stream/promises';

await pipeline(
  createAudioStream('Привіт, світе!\n"Це аудіо"', { voice: 1, speed: 7 }),
  createWriteStream('speech.wav'),
);
```

Файл создаёт только получатель потока. Библиотека передаёт текст через UTF-8
stdin, получает WAV через stdout и не использует временные аудиофайлы.
Каждый вызов запускает отдельный процесс. `pipeline()` и async iteration
передают ошибки вызывающему коду; обычный `.pipe()` требует собственного
обработчика `error`. Используйте `try/catch` вокруг создания потока и `pipeline()`.

Пример HTTP-ответа:

```js
import { createServer } from 'node:http';
import { pipeline } from 'node:stream/promises';
import { createAudioStream } from 'rozm-tts';

createServer(async (request, response) => {
  try {
    response.setHeader('Content-Type', 'audio/wav');
    await pipeline(createAudioStream('Привіт, світе!'), response);
  } catch (error) {
    // pipeline destroys the response if the stream fails or the client leaves.
    console.error(error);
    response.destroy();
  }
}).listen(3000);
```

| Опция | По умолчанию | Значение |
|---|---|---|
| `voice` | `1` | Целое 1–3 |
| `speed` | `5` | Целое 1–9; действует только для голоса 1 |
| `format` | `'wav'` | Только `'wav'`; MP3 пока не реализован |
| `signal` | — | `AbortSignal` для отмены |
| `binaryPath` | Встроенный бинарник для платформы | Полный или относительный путь к совместимому Rozm CLI |
| `dataDir` | `ROZM_DATA_DIR` или `data/` рядом с бинарником | Каталог оригинальных голосовых ресурсов |

Относительные пути разрешаются от `process.cwd()`. Путь встроенного бинарника
определяется относительно установленной библиотеки и не зависит от cwd.
Типы для TypeScript включены; TypeScript-проекту нужны стандартные `@types/node`.

```js
const controller = new AbortController();
const audio = createAudioStream('Довгий текст', { signal: controller.signal });
const pending = pipeline(audio, destination);
controller.abort(); // pending отклоняется с name='AbortError', code='ABORT_ERR'
await pending;
// audio.destroy() тоже прекращает работу дочернего процесса.
```

Аргументы неверного типа, неподдерживаемый формат, неправильные диапазоны и
текст >32768 UTF-8 байт вызывают синхронный `TypeError`/`RangeError`.
Непарные UTF-16 surrogate отклоняются, без неявной замены символов.
Ошибки синтеза и ресурсов приходят через поток: `code='ROZM_EXIT'`,
`exitCode`, `signal` и `stderr` (до 16 KiB). Ошибка запуска сохраняет системный
код, например `ENOENT`. Неподдерживаемые символы, включая NUL, проверяет движок.

Поток учитывает backpressure получателя. Он должен быть прочитан или уничтожен:
непрочитанный поток удерживает процесс и его буферы. Отмена/`destroy()` прекращает
работу процесса. Event loop Node.js не выполняет синтез.

Это поток **экспорта**, а синтез завершается до первого WAV-байта: движок
сначала собирает PCM в памяти, затем записывает WAV с точными RIFF/data длинами.
Лимит PCM — 64 MiB на вызов. Формат: mono, 11025 Hz, unsigned 8-bit.
Первый байт не означает начало речи в реальном времени. MP3 и потоковый
синтез с выдачей по мере генерации в эту версию не входят.

Особенности оригинального Rozm сохранены: украинский словарь, Latin fallback,
числа, кавычки/апострофы и переключение голосов `#1`/`#2`/`#3`.
Реальные LF/CRLF/CR разделяют фразы; литерал `\\n` не является переносом.
Апостроф после гласной может задавать ударение. `ґ`/`Ґ` пропускаются исходной
нормализацией. Словарь английского произношения отсутствует в исходных данных.

Для разработки из исходников сначала соберите Rust CLI и задайте `binaryPath`
и `dataDir`. Для комплекта с обоими бинарниками из корня репозитория:

```sh
# После сборки target/linux-build/release/rozm-cli и Win32 release:
node scripts/package-node.cjs
npm pack ./nodejs --pack-destination ./dist
# Интеграционные тесты из корня, с реальным движком и orig/:
ROZM_TEST_BINARY=/absolute/path/to/rozm-cli node --test nodejs/test/stream.test.cjs
```

Упаковка не скачивает ничего и не выполняет оригинальный Rozm.exe. Для
переупаковки после изменений пересоберите Rust release-бинарники. На обычной
Linux-сборке можно направить `CARGO_TARGET_DIR=target/linux-build`.
