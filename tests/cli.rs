use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

#[test]
fn stdin_accepts_utf8_text_and_rejects_invalid_or_excessive_input_before_output() {
    let dir = TestDirectory::new();
    for (input, expected, error) in [
        (
            b"\"\xd0\xb0\"\n\xd0\xb0".to_vec(),
            Some(&include_bytes!("fixtures/original-a-v1-s5.wav")[..]),
            "",
        ),
        (vec![0xff], None, "UTF-8"),
        (b"a\0b".to_vec(), None, "U+0000"),
        (vec![b'a'; 32769], None, "32768"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
            .args(["--stdin", "--output", "-"])
            .env(
                "ROZM_DATA_DIR",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
            )
            .current_dir(&dir.0)
            .bounded_input(Some(input))
            .unwrap();
        if let Some(one) = expected {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let mut wav = one[..44].to_vec();
            let length = 2 * (one.len() - 44);
            wav[4..8].copy_from_slice(&((length + 36) as u32).to_le_bytes());
            wav[40..44].copy_from_slice(&(length as u32).to_le_bytes());
            wav.extend_from_slice(&one[44..]);
            wav.extend_from_slice(&one[44..]);
            assert!(output.stdout == wav);
        } else {
            assert!(!output.status.success());
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(error),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    for args in [
        vec!["--stdin", "--text", "а"],
        vec!["--stdin=а"],
        vec!["--stdin", "--stdin"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
            .args(args)
            .current_dir(&dir.0)
            .bounded_output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 0);
}

#[test]
fn wav_can_be_streamed_to_stdout_without_creating_a_file() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "Привіт", "--output", "-"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout == include_bytes!("fixtures/original-pryvit-v1-s5.wav"));
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 0);
}

trait BoundedOutput {
    fn bounded_output(&mut self) -> std::io::Result<std::process::Output> {
        self.bounded_input(None)
    }
    fn bounded_input(&mut self, input: Option<Vec<u8>>) -> std::io::Result<std::process::Output>;
}
impl BoundedOutput for Command {
    fn bounded_input(&mut self, input: Option<Vec<u8>>) -> std::io::Result<std::process::Output> {
        use std::io::{Read, Write};
        use std::process::Stdio;
        let mut child = self
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let writer = input.map(|bytes| {
            let mut stdin = child.stdin.take().unwrap();
            std::thread::spawn(move || {
                // A rejected request may close stdin before every input byte is written.
                let _ = stdin.write_all(&bytes);
            })
        });
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let out = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).map(|_| bytes)
        });
        let err = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            stderr.read_to_end(&mut bytes).map(|_| bytes)
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(5))
                }
                result => {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = out.join();
                    let _ = err.join();
                    if let Some(writer) = writer {
                        let _ = writer.join();
                    }
                    return Err(result.err().unwrap_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "CLI exceeded 30-second deadline",
                        )
                    }));
                }
            }
        };
        if let Some(writer) = writer {
            writer.join().unwrap();
        }
        Ok(std::process::Output {
            status,
            stdout: out.join().unwrap()?,
            stderr: err.join().unwrap()?,
        })
    }
}

struct TestDirectory(PathBuf);
impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rozm-cli-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn help_explains_text_voice_speed_and_default_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .arg("--help")
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let help = String::from_utf8(output.stdout).unwrap();
    for expected in ["--text", "--output", "out.wav", "--voice", "--speed"] {
        assert!(help.contains(expected), "missing {expected}: {help}");
    }
}

#[test]
fn text_creates_default_wav_matching_original_pcm() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "а"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read(dir.0.join("out.wav")).unwrap(),
        include_bytes!("fixtures/original-a-v1-s5.wav")
    );
}

#[test]
fn ukrainian_word_uses_original_dictionary_stress_and_phonetics() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "Привіт"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read(dir.0.join("out.wav")).unwrap(),
        include_bytes!("fixtures/original-pryvit-v1-s5.wav")
    );
}

#[test]
fn explicit_unicode_output_replaces_existing_file_without_appending() {
    let dir = TestDirectory::new();
    let path = dir.0.join("запис з пробілами.wav");
    std::fs::write(&path, vec![42; 20000]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "Привіт", "--output"])
        .arg(&path)
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read(path).unwrap(),
        include_bytes!("fixtures/original-pryvit-v1-s5.wav")
    );
}

#[test]
fn voice_and_speed_options_match_original_waveforms() {
    for (voice, speed, expected) in [
        (1, 1, &include_bytes!("fixtures/original-a-v1-s1.wav")[..]),
        (1, 9, &include_bytes!("fixtures/original-a-v1-s9.wav")[..]),
        (2, 5, &include_bytes!("fixtures/original-a-v2-s5.wav")[..]),
        (3, 5, &include_bytes!("fixtures/original-a-v3-s5.wav")[..]),
    ] {
        let dir = TestDirectory::new();
        let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
            .args([
                "--text",
                "а",
                "--voice",
                &voice.to_string(),
                "--speed",
                &speed.to_string(),
            ])
            .env(
                "ROZM_DATA_DIR",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
            )
            .current_dir(&dir.0)
            .bounded_output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read(dir.0.join("out.wav")).unwrap(),
            expected,
            "voice {voice}, speed {speed}"
        );
    }
}

#[test]
fn punctuation_preserves_original_clause_pauses() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "Привіт, світе!"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::fs::read(dir.0.join("out.wav")).unwrap()
            == include_bytes!("fixtures/original-greeting-v1-s5.wav"),
        "greeting WAV differs from original"
    );
}

#[test]
fn empty_text_fails_without_destroying_existing_output() {
    for text in ["", " \t\r\n ", "?!"] {
        let dir = TestDirectory::new();
        std::fs::write(dir.0.join("out.wav"), b"previous result").unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
            .args(["--text", text])
            .env(
                "ROZM_DATA_DIR",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
            )
            .current_dir(&dir.0)
            .bounded_output()
            .unwrap();
        assert!(!output.status.success(), "empty input was accepted");
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
        assert_eq!(
            std::fs::read(dir.0.join("out.wav")).unwrap(),
            b"previous result"
        );
    }
}

#[test]
fn numbers_are_spoken_with_original_ukrainian_inflection() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "123"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::fs::read(dir.0.join("out.wav")).unwrap()
            == include_bytes!("fixtures/original-123-v1-s5.wav"),
        "numeric speech differs from original"
    );
}

#[test]
fn latin_text_uses_original_transliteration() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "hello"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::fs::read(dir.0.join("out.wav")).unwrap()
            == include_bytes!("fixtures/original-hello-v1-s5.wav"),
        "Latin speech differs from original"
    );
}

#[test]
fn unsupported_output_format_is_rejected_before_overwriting() {
    let dir = TestDirectory::new();
    let path = dir.0.join("out.mp3");
    std::fs::write(&path, b"existing mp3").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "Привіт", "--output"])
        .arg(&path)
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(!output.status.success(), "MP3 output was accepted");
    assert!(String::from_utf8_lossy(&output.stderr).contains("WAV"));
    assert_eq!(std::fs::read(path).unwrap(), b"existing mp3");
}

#[test]
fn oversized_text_is_rejected_before_loading_resources() {
    let dir = TestDirectory::new();
    std::fs::write(dir.0.join("out.wav"), b"previous").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", &"а".repeat(17000)])
        .env("ROZM_DATA_DIR", dir.0.join("missing"))
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("text exceeds"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(std::fs::read(dir.0.join("out.wav")).unwrap(), b"previous");
}

#[test]
fn damaged_correction_count_fails_without_panic_or_output_loss() {
    let dir = TestDirectory::new();
    let data = dir.0.join("data");
    std::fs::create_dir_all(data.join("skfs")).unwrap();
    let original = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig");
    for name in ["BSNbn", "skfs/skf", "skfs/wif", "skfs/wlf"] {
        std::fs::copy(original.join(name), data.join(name)).unwrap();
    }
    let path = data.join("skfs/wif");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[..4].copy_from_slice(&u32::MAX.to_le_bytes());
    std::fs::write(path, bytes).unwrap();
    std::fs::write(dir.0.join("out.wav"), b"previous").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "а"])
        .env("ROZM_DATA_DIR", data)
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(!error.contains("panicked"), "{error}");
    assert!(error.contains("dictionary count"), "{error}");
    assert_eq!(std::fs::read(dir.0.join("out.wav")).unwrap(), b"previous");
}

#[test]
fn quotes_and_line_breaks_reach_synthesis_as_literal_text() {
    let cases = [
        "\"Привіт\"",
        "'Привіт'",
        "«Привіт»",
        "а\nа",
        "а\r\nа",
        "а\rа",
        r"а\nа",
        "--приклад",
        "Привіт $ ` & | (світе);",
        "п'ять",
        "п’ять",
    ];
    for (index, text) in cases.iter().enumerate() {
        let dir = TestDirectory::new();
        let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
            .args(["--text", text])
            .env(
                "ROZM_DATA_DIR",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
            )
            .current_dir(&dir.0)
            .bounded_output()
            .unwrap();
        assert!(
            output.status.success(),
            "case {index}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let wav = std::fs::read(dir.0.join("out.wav")).unwrap();
        assert!(wav.len() > 44);
        assert_eq!(&wav[..4], b"RIFF");
        if index == 0 || index == 2 {
            assert!(
                wav == include_bytes!("fixtures/original-pryvit-v1-s5.wav"),
                "quotes changed speech in case {index}"
            );
        }
        if (3..6).contains(&index) {
            let a = include_bytes!("fixtures/original-a-v1-s5.wav");
            assert_eq!(&wav[44..], [&a[44..], &a[44..]].concat());
        }
        if index == 6 {
            let a = include_bytes!("fixtures/original-a-v1-s5.wav");
            assert_ne!(
                wav.len(),
                44 + 2 * (a.len() - 44),
                "literal backslash n became a newline"
            );
        }
    }
}

#[test]
fn mathematical_symbols_are_spoken_like_original() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "2+3=5"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::fs::read(dir.0.join("out.wav")).unwrap()
            == include_bytes!("fixtures/original-symbols-v1-s5.wav"),
        "symbol expansion differs from original"
    );
}

#[test]
fn ukrainian_number_sign_is_supported() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "№ 5"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(std::fs::read(dir.0.join("out.wav")).unwrap().len() > 44);
}

#[test]
fn ukrainian_stress_and_soft_consonants_match_original_corpus() {
    let cases = [
        (
            "Україна",
            &include_bytes!("fixtures/original-corpus-0.wav")[..],
        ),
        (
            "п’ять",
            &include_bytes!("fixtures/original-corpus-1.wav")[..],
        ),
        (
            "для мене",
            &include_bytes!("fixtures/original-corpus-2.wav")[..],
        ),
        ("ніч", &include_bytes!("fixtures/original-corpus-3.wav")[..]),
        (
            "щастя",
            &include_bytes!("fixtures/original-corpus-4.wav")[..],
        ),
        (
            "дзвін",
            &include_bytes!("fixtures/original-corpus-5.wav")[..],
        ),
        (
            "невідомеслово",
            &include_bytes!("fixtures/original-corpus-6.wav")[..],
        ),
        (
            r"а\том",
            &include_bytes!("fixtures/original-corpus-7.wav")[..],
        ),
        (
            "Ґанок",
            &include_bytes!("fixtures/original-corpus-8.wav")[..],
        ),
    ];
    for (text, expected) in cases {
        let dir = TestDirectory::new();
        let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
            .args(["--text", text])
            .env(
                "ROZM_DATA_DIR",
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
            )
            .current_dir(&dir.0)
            .bounded_output()
            .unwrap();
        assert!(
            output.status.success(),
            "{text}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            std::fs::read(dir.0.join("out.wav")).unwrap() == expected,
            "original corpus differs for {text}"
        );
    }
}

#[test]
fn unsupported_unicode_and_invalid_options_fail_cleanly() {
    for args in [
        vec!["--text", "Привіт 😀"],
        vec!["--text", "а\u{0301}"],
        vec!["--text", "а", "--voice", "0"],
        vec!["--text", "а", "--speed", "10"],
        vec!["--text", "а", "--speed", "x"],
        vec!["--text"],
        vec![],
        vec!["--unknown"],
        vec!["--text", "а", "--text", "б"],
    ] {
        let dir = TestDirectory::new();
        std::fs::write(dir.0.join("out.wav"), b"previous").unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
            .args(args)
            .env("ROZM_DATA_DIR", dir.0.join("missing"))
            .current_dir(&dir.0)
            .bounded_output()
            .unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!error.contains("panicked"), "{error}");
        assert!(
            !error.contains("resource"),
            "input not validated first: {error}"
        );
        assert_eq!(std::fs::read(dir.0.join("out.wav")).unwrap(), b"previous");
    }
}

#[cfg(any(unix, windows))]
#[test]
fn non_unicode_text_argument_is_rejected_without_panic() {
    #[cfg(unix)]
    use std::os::unix::ffi::OsStringExt;
    #[cfg(windows)]
    use std::os::windows::ffi::OsStringExt;
    #[cfg(unix)]
    let argument = std::ffi::OsString::from_vec(vec![0xff]);
    #[cfg(windows)]
    let argument = std::ffi::OsString::from_wide(&[0xd800]);
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .arg("--text")
        .arg(argument)
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not valid Unicode"));
    assert!(!dir.0.join("out.wav").exists());
}

#[test]
fn missing_resources_and_unwritable_destination_report_errors() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "а"])
        .env("ROZM_DATA_DIR", dir.0.join("missing"))
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("BSNbn"));
    assert!(!dir.0.join("out.wav").exists());
    let blocked = dir.0.join("directory.wav");
    std::fs::create_dir(&blocked).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "а", "--output"])
        .arg(&blocked)
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(!output.status.success());
    assert!(blocked.is_dir());
    assert!(
        !std::fs::read_dir(&dir.0).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".rozm-")),
        "temporary file leaked"
    );
}

#[test]
fn excessive_voice_resource_is_rejected_before_synthesis() {
    let dir = TestDirectory::new();
    let data = dir.0.join("data");
    let original = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig");
    for name in [
        "BSNbn",
        "skfs/skf",
        "skfs/wif",
        "skfs/wlf",
        "snf1/ip2f",
        "snf1/lp2f",
    ] {
        let path = data.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(original.join(name), path).unwrap();
    }
    let path = data.join("snf1/Sd2f");
    std::fs::copy(original.join("snf1/Sd2f"), &path).unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .unwrap()
        .set_len(33 * 1024 * 1024)
        .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "а"])
        .env("ROZM_DATA_DIR", data)
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(!output.status.success(), "oversized resource was accepted");
    assert!(String::from_utf8_lossy(&output.stderr).contains("size limit"));
    assert!(!dir.0.join("out.wav").exists());
}

#[test]
fn excessive_audio_is_rejected_without_replacing_previous_file() {
    let dir = TestDirectory::new();
    let data = dir.0.join("data");
    let original = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig");
    for name in ["BSNbn", "skfs/skf", "skfs/wif", "skfs/wlf"] {
        let path = data.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(original.join(name), path).unwrap();
    }
    std::fs::create_dir_all(data.join("snf2")).unwrap();
    std::fs::write(data.join("snf2/ip2f"), vec![0; 7056]).unwrap();
    std::fs::write(
        data.join("snf2/lp2f"),
        (0..1764)
            .flat_map(|_| (1024 * 1024_i32).to_le_bytes())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    std::fs::write(data.join("snf2/Sd2f"), vec![128; 1024 * 1024]).unwrap();
    std::fs::write(dir.0.join("out.wav"), b"previous").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", &"а".repeat(100), "--voice", "2"])
        .env("ROZM_DATA_DIR", data)
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(!output.status.success(), "unbounded audio was accepted");
    assert!(String::from_utf8_lossy(&output.stderr).contains("audio exceeds"));
    assert_eq!(std::fs::read(dir.0.join("out.wav")).unwrap(), b"previous");
}

#[test]
fn ukrainian_initial_is_read_using_original_letter_name() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "А."])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::fs::read(dir.0.join("out.wav")).unwrap()
            == include_bytes!("fixtures/original-initial-v1-s5.wav"),
        "initial differs from original"
    );
}

#[test]
fn voice_directives_switch_voice_without_gui_and_keep_one_file() {
    let dir = TestDirectory::new();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "а\n#2а\n#3а"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = [
        &include_bytes!("fixtures/original-a-v1-s5.wav")[44..],
        &include_bytes!("fixtures/original-a-v2-s5.wav")[44..],
        &include_bytes!("fixtures/original-a-v3-s5.wav")[44..],
    ]
    .concat();
    assert!(std::fs::read(dir.0.join("out.wav")).unwrap()[44..] == expected);
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 1);
}

#[test]
fn resources_are_found_beside_executable_from_another_working_directory() {
    let dir = TestDirectory::new();
    let bin = dir.0.join("installed");
    let data = bin.join("data");
    let original = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig");
    for name in [
        "BSNbn",
        "skfs/skf",
        "skfs/wif",
        "skfs/wlf",
        "snf1/ip2f",
        "snf1/lp2f",
        "snf1/Sd2f",
    ] {
        let path = data.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(original.join(name), path).unwrap();
    }
    let executable = bin.join(if cfg!(windows) {
        "rozm-cli.exe"
    } else {
        "rozm-cli"
    });
    std::fs::copy(env!("CARGO_BIN_EXE_rozm-cli"), &executable).unwrap();
    let output = Command::new(executable)
        .args(["--text=а"])
        .env_remove("ROZM_DATA_DIR")
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        std::fs::read(dir.0.join("out.wav")).unwrap()
            == include_bytes!("fixtures/original-a-v1-s5.wav")
    );
}

#[test]
fn malformed_voice_tables_fail_cleanly() {
    let dir = TestDirectory::new();
    let data = dir.0.join("data");
    let original = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig");
    for name in [
        "BSNbn",
        "skfs/skf",
        "skfs/wif",
        "skfs/wlf",
        "snf1/ip2f",
        "snf1/lp2f",
        "snf1/Sd2f",
    ] {
        let path = data.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(original.join(name), path).unwrap();
    }
    for name in ["snf1/Sd2f", "snf1/ip2f"] {
        let path = data.join(name);
        let saved = std::fs::read(&path).unwrap();
        std::fs::write(&path, b"broken").unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
            .args(["--text", "а"])
            .env("ROZM_DATA_DIR", &data)
            .current_dir(&dir.0)
            .bounded_output()
            .unwrap();
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
        assert!(!dir.0.join("out.wav").exists());
        std::fs::write(path, saved).unwrap();
    }
}

#[test]
fn voice_directives_alone_are_rejected_as_empty_speech() {
    let dir = TestDirectory::new();
    std::fs::write(dir.0.join("out.wav"), b"previous").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rozm-cli"))
        .args(["--text", "#1#2#3"])
        .env(
            "ROZM_DATA_DIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("orig"),
        )
        .current_dir(&dir.0)
        .bounded_output()
        .unwrap();
    assert!(!output.status.success(), "empty speech was accepted");
    assert_eq!(std::fs::read(dir.0.join("out.wav")).unwrap(), b"previous");
}
