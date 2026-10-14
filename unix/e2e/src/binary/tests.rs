//! Unit tests for the binary naming and for the accounts a dead process leaves behind.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use super::{
    COMMAND_LINE_UNITS, FATAL_LINES, KEEP, LAYER_LOG_EXT, PROCESS_LOG_EXT, WINE_START_EXE,
    WINE_START_EXE_FLAG, WineLauncher, argument_units, binary_name, fitting_prefix, keep_layer_log,
    keep_process, keep_stderr, layer_tail, stderr_tail, test_arguments,
};
use crate::{
    attribute::{BinaryOutcome, Launcher as _, ProcessEnd, Report, TestResult, run_binary},
    run::ExitKind,
};

const DRIVER_HANG: &str = "Caused GPU Hang Error \
    (00000003:kIOAccelCommandBufferCallbackErrorHang)";

#[derive(Default)]
struct Log {
    results: Vec<TestResult>,
    notes: Vec<String>,
}

impl Report for Log {
    fn result(&mut self, result: TestResult) {
        self.results.push(result);
    }

    fn note(&mut self, note: &str) {
        self.notes.push(note.to_owned());
    }
}

/// A fresh directory under the temp dir, named after the test using it.
fn dir(tag: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("mtld3d-e2e-stderr-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("temp dir");
    path
}

/// The units `fitting_prefix` has spent once every fixed argument is in.
///
/// Counted the same way it counts them: whichever of Wine's two image
/// spellings is larger, the fixed flags, and the one separator the first
/// name's own units still need added to it.
fn fixed_head_units(exe: &Path, threads: u32, ignored: bool) -> usize {
    let image = format!(r"\\?\unix{}", exe.to_string_lossy().replace('/', "\\"));
    let direct = argument_units(&image, true);
    let via_start_exe = argument_units(WINE_START_EXE, true)
        + 1
        + argument_units(WINE_START_EXE_FLAG, false)
        + 1
        + argument_units(&exe.to_string_lossy(), false);
    let mut units = direct.max(via_start_exe);
    for arg in test_arguments(Some(&[]), threads, ignored) {
        units += 1 + argument_units(&arg, false);
    }
    units + 1
}

/// Dozens of the `fixme:dbghelp` lines Wine prints around a backtrace.
fn chatter(count: usize) -> String {
    (0..count)
        .map(|i| format!("fixme:dbghelp:elf_search_auxv can't find symbol {i}\n"))
        .collect::<Vec<_>>()
        .concat()
}

#[test]
fn the_name_drops_cargos_hash_and_nothing_else() {
    assert_eq!(
        binary_name(Path::new("/t/deps/e2e-1030f4ab05278ecb.exe")),
        "e2e"
    );
    assert_eq!(
        binary_name(Path::new("snmalloc_drift-0a1b2c3d4e5f6071.exe")),
        "snmalloc_drift"
    );
    assert_eq!(binary_name(Path::new("unload.exe")), "unload");
    assert_eq!(
        binary_name(Path::new("multi-device.exe")),
        "multi-device",
        "a dash followed by a non-hex word is part of the name"
    );
}

#[test]
fn the_tail_keeps_the_last_lines() {
    let text = (0..20)
        .map(|i| format!("line {i}\n"))
        .collect::<Vec<_>>()
        .concat();
    let tail = stderr_tail(&text);
    assert!(tail.starts_with("line 5\n"));
    assert!(tail.ends_with("line 19"));
    assert_eq!(stderr_tail("a\nb"), "a\nb");
}

#[test]
fn the_tail_reaches_past_wines_chatter_to_the_failure() {
    let stderr = format!(
        "IOSurfaceClientCreate failed\n{}wine = 929;\nerr:dbghelp:elf_map_file cannot read\n",
        chatter(40)
    );
    let tail = stderr_tail(&stderr);
    assert_eq!(tail, "IOSurfaceClientCreate failed\nwine = 929;");
}

#[test]
fn a_wine_channel_line_is_chatter_only_for_fixme_and_dbghelp() {
    let stderr = "err:module:import_dll library not found\nfixme:d3d:unimplemented\ntrace:dbghelp:noise\nplain: text: line\n";
    assert_eq!(
        stderr_tail(stderr),
        "err:module:import_dll library not found\nplain: text: line"
    );
}

#[test]
fn the_whole_stderr_lands_in_a_file_the_report_can_name() {
    let dir = dir("kept");
    let stderr = format!("IOSurfaceClientCreate failed\n{}", chatter(40));
    let path = keep_stderr(&dir, "e2e", 4242, &stderr).expect("kept");
    assert_eq!(path, dir.join("e2e-4242.stderr"));
    assert_eq!(std::fs::read_to_string(&path).expect("read"), stderr);
}

#[test]
fn the_kept_files_are_capped_per_kind_and_never_touch_the_layers_logs() {
    let dir = dir("capped");
    std::fs::write(dir.join("e2e-1.log"), "the layer's own").expect("log");
    for pid in 0..u32::try_from(KEEP).expect("small") + 5 {
        keep_stderr(&dir, "e2e", pid, "stderr").expect("kept");
        keep_process(
            &dir,
            "e2e",
            &ProcessEnd {
                pid,
                kind: ExitKind::Code(5),
                stdout: "stdout".to_owned(),
                stderr: "stderr".to_owned(),
                gpu_hang: false,
            },
        )
        .expect("process kept");
        std::fs::write(dir.join(format!("e2e-abcd-{pid}.log")), "dead").expect("layer log");
        keep_layer_log(&dir, "e2e-abcd", "e2e", pid).expect("kept");
    }
    let kept = |ext: &str| {
        std::fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == ext))
            .count()
    };
    assert_eq!(kept("stderr"), KEEP);
    assert_eq!(kept(LAYER_LOG_EXT), KEEP);
    assert_eq!(kept(PROCESS_LOG_EXT), KEEP);
    assert_eq!(kept("log"), 1, "a live process's log is not the runner's");
}

#[test]
fn a_layer_log_is_read_from_its_fatal_line_on() {
    let work: String = (0..30)
        .map(|i| format!("[2026-01-01T00:00:00Z INFO  mtld3d::unix] work {i}\n"))
        .collect::<Vec<_>>()
        .concat();
    let log = format!(
        "{work}[mtld3d::unix] FATAL: SIGSEGV fault=0x0\n[mtld3d::unix] thread=mtld3d-encoder\n"
    );
    let tail = layer_tail(&log);
    assert!(tail.starts_with("[mtld3d::unix] FATAL: SIGSEGV"), "{tail}");
    assert!(tail.ends_with("thread=mtld3d-encoder"), "{tail}");
    assert!(!tail.contains("work 29"), "{tail}");
}

#[test]
fn a_layer_log_without_a_fatal_line_is_tailed_like_stderr() {
    let log = (0..20)
        .map(|i| format!("line {i}\n"))
        .collect::<Vec<_>>()
        .concat();
    assert_eq!(layer_tail(&log), stderr_tail(&log));
    assert!(layer_tail(&log).starts_with("line 5\n"));
}

#[test]
fn a_fatal_line_early_in_a_long_log_is_quoted_to_a_bound() {
    let log = format!(
        "[mtld3d::unix] FATAL: SIGABRT\n{}",
        (0..200)
            .map(|i| format!("frame {i}\n"))
            .collect::<Vec<_>>()
            .concat()
    );
    assert_eq!(layer_tail(&log).lines().count(), FATAL_LINES);
}

#[test]
fn the_layers_log_of_a_dead_process_is_moved_out_of_the_layers_retention() {
    let dir = dir("layer");
    let exe = Path::new("/t/deps/e2e-1030f4ab05278ecb.exe");
    let launcher = WineLauncher::new(
        Path::new("/usr/bin/true"),
        exe,
        Some(&dir),
        Duration::from_secs(1),
        Box::new(|_| {}),
    )
    .expect("absolute test path");
    assert!(
        launcher.keep_layer_log(4242).is_err(),
        "a process that logged nothing has no file"
    );
    let log = "ordinary line\n[mtld3d::unix] FATAL: SIGSEGV fault=0x8\n";
    let own = dir.join("e2e-1030f4ab05278ecb-4242.log");
    std::fs::write(&own, log).expect("layer log");
    let kept = launcher.keep_layer_log(4242).expect("the layer log");
    assert_eq!(kept.path, dir.join("e2e-4242.layer-log"));
    assert_eq!(kept.tail, "[mtld3d::unix] FATAL: SIGSEGV fault=0x8");
    assert!(kept.not_kept.is_none());
    assert!(
        kept.path.extension().is_none_or(|ext| ext != "log"),
        "the layer prunes by the `log` extension"
    );
    assert!(!own.exists(), "moved, not copied");
    assert_eq!(std::fs::read_to_string(&kept.path).expect("read"), log);
}

#[test]
fn a_layer_log_that_cannot_be_moved_is_still_quoted_where_it_is() {
    let dir = dir("unmovable");
    let own = dir.join("e2e-abcd-7.log");
    std::fs::write(&own, "[mtld3d::unix] FATAL: SIGBUS\n").expect("layer log");
    // A directory in the way of the kept name makes the move fail.
    std::fs::create_dir(dir.join("e2e-7.layer-log")).expect("blocker");
    let kept = keep_layer_log(&dir, "e2e-abcd", "e2e", 7).expect("read");
    assert_eq!(kept.path, own);
    assert_eq!(kept.tail, "[mtld3d::unix] FATAL: SIGBUS");
    assert!(
        kept.not_kept
            .is_some_and(|why| why.contains("e2e-7.layer-log"))
    );
}

#[test]
fn a_clean_processes_layer_hang_stops_relaunch_and_keeps_both_accounts() {
    let log_dir = dir("gpu-hang");
    let script = log_dir.join("e2e-abcdef.sh");
    let launch_count = log_dir.join("launches");
    let body = format!(
        "printf x >> '{}'\n\
         echo '[e2e] running a::one' >&2\n\
         echo 'initiating stderr' >&2\n\
         printf '%s\\n' '{DRIVER_HANG}' > \"{}/e2e-abcdef-$$.log\"\n\
         exit 0\n",
        launch_count.display(),
        log_dir.display()
    );
    std::fs::write(&script, body).expect("script");
    let mut launcher = WineLauncher::new(
        Path::new("/bin/sh"),
        &script,
        Some(&log_dir),
        Duration::from_secs(5),
        Box::new(|_| {}),
    )
    .expect("absolute test path");
    let selection = Some(vec!["a::one".to_owned(), "a::two".to_owned()]);
    let mut log = Log::default();
    let run = run_binary(&mut launcher, selection, 1, false, &mut log).expect("run fake child");

    assert_eq!(run.processes, 1);
    assert_eq!(run.outcome, BinaryOutcome::GpuHang);
    assert!(!run.failed);
    assert!(log.results.is_empty(), "a hung GPU gives no test verdict");
    assert_eq!(
        std::fs::read_to_string(&launch_count).expect("launch count"),
        "x"
    );
    let note = log.notes.first().expect("GPU hang note");
    assert!(note.contains("GPU hang"), "{note}");
    assert!(note.contains("has no verdict"), "{note}");

    let kept: Vec<PathBuf> = std::fs::read_dir(&log_dir)
        .expect("read log dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    let stderr = kept
        .iter()
        .find(|path| path.extension().is_some_and(|ext| ext == "stderr"))
        .expect("kept stderr");
    let layer = kept
        .iter()
        .find(|path| path.extension().is_some_and(|ext| ext == LAYER_LOG_EXT))
        .expect("kept layer log");
    assert_eq!(
        std::fs::read_to_string(stderr).expect("stderr evidence"),
        "[e2e] running a::one\ninitiating stderr\n"
    );
    assert_eq!(
        std::fs::read_to_string(layer).expect("layer evidence"),
        format!("{DRIVER_HANG}\n")
    );
}

#[test]
fn command_size_counts_windows_quoting_and_utf16_units() {
    for (argument, encoded) in [
        ("", r#""""#),
        ("plain", "plain"),
        ("a b", r#""a b""#),
        ("a\tb", "\"a\tb\""),
        (r#"a"b"#, r#"a\"b"#),
        (r#"a\"b"#, r#"a\\\"b"#),
        (r#"a\\"b"#, r#"a\\\\\"b"#),
        (r"a b\", r#""a b\\""#),
        (r"a b\\", r#""a b\\\\""#),
        ("\u{1f680}", "\u{1f680}"),
    ] {
        assert_eq!(
            argument_units(argument, false),
            encoded.encode_utf16().count(),
            "{argument:?}"
        );
    }
    assert_eq!(argument_units("plain", true), 7, "argv[0] is always quoted");
    assert_eq!(
        argument_units(r"C:\plain\", true),
        12,
        "the final slash doubles inside quotes"
    );
}

#[test]
fn command_size_boundary_includes_path_flags_separators_and_nul() {
    let exe = Path::new("/test path/\u{1f680}/suite.exe");
    let fixed = fixed_head_units(exe, u32::MAX, false);
    // Wine's own check fails once the built line reaches COMMAND_LINE_UNITS
    // units (excluding the NUL), so the last unit that still fits is one
    // short of it.
    let maximum = "x".repeat(COMMAND_LINE_UNITS - 1 - fixed);
    assert_eq!(
        fitting_prefix(exe, std::slice::from_ref(&maximum), u32::MAX, false).unwrap(),
        1
    );
    assert!(fitting_prefix(exe, &[maximum + "x"], u32::MAX, false).is_err());
    assert_eq!(fitting_prefix(exe, &[], u32::MAX, false).unwrap(), 0);
    // A longer path consumes capacity rather than borrowing an arbitrary
    // reserve from every selection, and a surrogate pair consumes two units.
    let short = Path::new("/suite.exe");
    let names = vec!["x".repeat(COMMAND_LINE_UNITS - 1 - fixed), "🚀".to_owned()];
    assert_eq!(fitting_prefix(exe, &names, u32::MAX, false).unwrap(), 1);
    assert_eq!(fitting_prefix(short, &names, u32::MAX, false).unwrap(), 2);
    assert_eq!(
        test_arguments(Some(&[]), 7, false),
        ["--test-threads=7", "--nocapture", "--exact"]
    );
}

#[test]
fn an_ignored_run_asks_libtest_for_the_ignored_tests_and_sizes_the_flag() {
    assert_eq!(
        test_arguments(None, 1, true),
        ["--test-threads=1", "--nocapture", "--ignored"]
    );
    assert_eq!(
        test_arguments(Some(&["a".to_owned()]), 1, true),
        [
            "--test-threads=1",
            "--nocapture",
            "--ignored",
            "--exact",
            "a"
        ]
    );
    let exe = Path::new("/suite.exe");
    let fixed = fixed_head_units(exe, 1, true);
    let name = "x".repeat(COMMAND_LINE_UNITS - 1 - fixed);
    assert_eq!(
        fitting_prefix(exe, std::slice::from_ref(&name), 1, true).unwrap(),
        1
    );
    assert!(fitting_prefix(exe, &[name + "x"], 1, true).is_err());
}

#[test]
fn launcher_uses_the_same_absolute_executable_for_sizing_and_launch() {
    let relative = Path::new("tests/../suite.exe");
    let launcher = WineLauncher::new(
        Path::new("/bin/false"),
        relative,
        None,
        Duration::from_secs(1),
        Box::new(|_| {}),
    )
    .unwrap();
    assert_eq!(launcher.exe, std::path::absolute(relative).unwrap());
    let mut launcher = launcher;
    let mut log = Log::default();
    let error = run_binary(
        &mut launcher,
        Some(vec!["x".repeat(COMMAND_LINE_UNITS)]),
        1,
        false,
        &mut log,
    )
    .err()
    .expect("oversized name rejected before launch");
    assert!(error.contains("executable-path allowance"));
    assert_eq!(log.results, [] as [TestResult; 0]);
}

#[test]
fn large_recovery_launches_bounded_processes_and_keeps_the_primary_failure() {
    let log_dir = dir("large-recovery");
    let script = log_dir.join("suite.sh");
    let runs = log_dir.join("runs");
    let body = format!(
        "joined=\"$*\"\nprintf '%s\\n' \"${{#joined}}\" >> '{}'\n\
         count=0\n\
         for name do\n\
           case \"$name\" in --*) continue;; esac\n\
           if [ \"$name\" = primary ]; then\n\
             echo \"thread 'primary' panicked at suite.rs:1:1:\" >&2\n\
             echo 'original failure' >&2\n\
             exit 101\n\
           fi\n\
           printf 'test %s ... ok\\n' \"$name\"\n\
           count=$((count + 1))\n\
         done\n\
         printf 'test result: ok. %s passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.1s\\n' \"$count\"\n",
        runs.display()
    );
    std::fs::write(&script, body).unwrap();
    let mut names = vec!["primary".to_owned()];
    names.extend((0..570).map(|index| format!("test_{index:03}_{}", "x".repeat(60))));
    let mut launcher = WineLauncher::new(
        Path::new("/bin/sh"),
        &script,
        Some(&log_dir),
        Duration::from_secs(5),
        Box::new(|_| {}),
    )
    .unwrap();
    let mut log = Log::default();
    let run = run_binary(&mut launcher, Some(names.clone()), 4, false, &mut log).unwrap();
    assert!(run.failed);
    assert_eq!(
        run.processes, 3,
        "primary process, its recovery, and the deferred suffix"
    );
    assert_eq!(
        log.results
            .iter()
            .map(|result| &result.name)
            .collect::<Vec<_>>(),
        names.iter().collect::<Vec<_>>()
    );
    assert!(
        matches!(&log.results[0].verdict, crate::attribute::Verdict::Failed(detail) if detail.contains("original failure"))
    );
    assert!(
        log.results[1..]
            .iter()
            .all(|result| result.verdict == crate::attribute::Verdict::Passed)
    );
    let lengths = std::fs::read_to_string(&runs).unwrap();
    assert_eq!(lengths.lines().count(), 3);
    assert!(
        lengths
            .lines()
            .all(|line| line.parse::<usize>().unwrap() < COMMAND_LINE_UNITS)
    );
    std::fs::remove_dir_all(log_dir).unwrap();
}

#[test]
fn complete_nonzero_exit_keeps_captured_output_without_retrying() {
    for custom_dir in [true, false] {
        let root = dir(if custom_dir {
            "complete-custom"
        } else {
            "complete-default"
        });
        let log_dir = root.join(if custom_dir {
            "selected-logs"
        } else {
            "mtld3d-logs"
        });
        let script = root.join("completed-abcdef.sh");
        let body = "printf x >> launches\nprintf '%s' \"$$\" > pid\nprintf '%s\\n' 'early stdout' 'stdout:' 'test a::one ... ok' 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.1s' 'late stdout'\nprintf '%s' 'early stderr
stderr:
unterminated stderr' >&2\nexit 5\n";
        std::fs::write(&script, body).expect("script");
        let mut launcher = WineLauncher::new(
            Path::new("/bin/sh"),
            &script,
            custom_dir.then_some(log_dir.as_path()),
            Duration::from_secs(5),
            Box::new(|_| {}),
        )
        .expect("launcher");
        let mut log = Log::default();
        let run = run_binary(
            &mut launcher,
            Some(vec!["a::one".to_owned()]),
            1,
            true,
            &mut log,
        )
        .expect("fake process");
        assert_eq!(run.processes, 1);
        assert_eq!(run.outcome, BinaryOutcome::Complete);
        assert!(!run.failed);
        assert_eq!(std::fs::read_to_string(root.join("launches")).unwrap(), "x");
        assert_eq!(log.results.len(), 1);
        assert_eq!(log.results[0].name, "a::one");
        assert_eq!(log.results[0].verdict, crate::attribute::Verdict::Passed);
        let pid = std::fs::read_to_string(root.join("pid")).expect("pid");
        let kept = log_dir.join(format!("completed-{pid}.process-log"));
        let output =
            std::fs::read_to_string(&kept).expect("abnormal completed process output retained");
        let stdout = "early stdout\nstdout:\ntest a::one ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.1s\nlate stdout\n";
        let stderr = "early stderr\nstderr:\nunterminated stderr";
        assert_eq!(
            output,
            format!(
                "binary: completed\npid: {pid}\nexit: exit code 5\nstdout-bytes: {}\nstderr-bytes: {}\n\nstdout:\n{stdout}\nstderr:\n{stderr}",
                stdout.len(),
                stderr.len(),
            )
        );
        assert_eq!(log.notes.len(), 1);
        assert!(log.notes[0].contains("exit code 5"));
        assert!(log.notes[0].contains(kept.to_str().unwrap()));
        std::fs::remove_dir_all(root).expect("remove fixture");
    }
}

#[test]
fn clean_and_declared_exits_keep_no_abnormal_bundle() {
    for declared in [false, true] {
        let root = dir(if declared {
            "declared-complete"
        } else {
            "clean-complete"
        });
        let script = root.join("completed-abcdef.sh");
        let body = if declared {
            "echo '[e2e] test a::one ends this process with exit code 5'\nexit 5\n"
        } else {
            "echo 'test a::one ... ok'\necho 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.1s'\nexit 0\n"
        };
        std::fs::write(&script, body).expect("script");
        let log_dir = root.join("logs");
        let mut launcher = WineLauncher::new(
            Path::new("/bin/sh"),
            &script,
            Some(&log_dir),
            Duration::from_secs(5),
            Box::new(|_| {}),
        )
        .expect("launcher");
        let mut log = Log::default();
        let run = run_binary(
            &mut launcher,
            Some(vec!["a::one".to_owned()]),
            1,
            true,
            &mut log,
        )
        .expect("fake child");
        assert_eq!(run.processes, 1);
        assert!(!run.failed);
        assert_eq!(run.outcome, BinaryOutcome::Complete);
        assert_eq!(log.results.len(), 1);
        assert_eq!(log.results[0].verdict, crate::attribute::Verdict::Passed);
        assert_eq!(log.notes, [] as [String; 0]);
        assert!(
            !log_dir.exists(),
            "clean completion needs no diagnostic directory"
        );
        std::fs::remove_dir_all(root).expect("remove fixture");
    }
}

#[test]
fn completed_process_retention_error_preserves_the_verdict() {
    let root = dir("complete-retention-error");
    let script = root.join("completed-abcdef.sh");
    std::fs::write(&script, "mkdir -p \"logs/completed-$$.process-log\"\necho 'test a::one ... ok'\necho 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.1s'\nexit 5\n").expect("script");
    let log_dir = root.join("logs");
    let mut launcher = WineLauncher::new(
        Path::new("/bin/sh"),
        &script,
        Some(&log_dir),
        Duration::from_secs(5),
        Box::new(|_| {}),
    )
    .expect("launcher");
    let mut log = Log::default();
    let run = run_binary(
        &mut launcher,
        Some(vec!["a::one".to_owned()]),
        1,
        true,
        &mut log,
    )
    .expect("fake child");
    assert_eq!(run.processes, 1);
    assert!(!run.failed);
    assert_eq!(run.outcome, BinaryOutcome::Complete);
    assert_eq!(log.results.len(), 1);
    assert_eq!(log.results[0].verdict, crate::attribute::Verdict::Passed);
    assert_eq!(log.notes.len(), 1);
    assert!(log.notes[0].contains("exit code 5"));
    assert!(log.notes[0].contains("could not be kept"));
    assert!(log.notes[0].contains(".process-log"));
    std::fs::remove_dir_all(root).expect("remove fixture");
}

/// An oracle for Wine's `build_command_line`, independent of `fitting_prefix`.
///
/// `dlls/ntdll/unix/env.c`: every argument's quoted/escaped length, argv\[0\]
/// always quoted, one separator between every pair of arguments and none at
/// the ends (Wine's loop appends one after every argument, including the
/// last, then drops it). Reuses [`argument_units`] for one argument's
/// quoting, which `command_size_counts_windows_quoting_and_utf16_units`
/// checks against literal encoded strings, but composes the arguments the
/// way Wine's loop does rather than the way `fitting_prefix` does, so a
/// composition mistake in `fitting_prefix` (there was one: an extra
/// separator counted per argument) shows up as a mismatch against this
/// oracle too.
fn wine_command_line_units(wargv: &[String]) -> usize {
    wargv
        .iter()
        .enumerate()
        .map(|(i, arg)| argument_units(arg, i == 0))
        .sum::<usize>()
        + wargv.len().saturating_sub(1)
}

/// The wargv Wine's loader builds when it can load `exe`'s architecture directly.
///
/// The mapped path, quoted, ahead of the chosen names.
fn direct_wargv(exe: &Path, names: &[String], threads: u32, ignored: bool) -> Vec<String> {
    let image = format!(r"\\?\unix{}", exe.to_string_lossy().replace('/', "\\"));
    let mut wargv = vec![image];
    wargv.extend(test_arguments(Some(names), threads, ignored));
    wargv
}

/// The wargv Wine's loader builds when it falls back to `start.exe`.
///
/// Every i686 process, on a WOW64-only build: `exe`'s own path reappears
/// unconverted, two arguments after the image Wine actually loaded.
fn via_start_exe_wargv(exe: &Path, names: &[String], threads: u32, ignored: bool) -> Vec<String> {
    let mut wargv = vec![
        WINE_START_EXE.to_owned(),
        WINE_START_EXE_FLAG.to_owned(),
        exe.to_string_lossy().into_owned(),
    ];
    wargv.extend(test_arguments(Some(names), threads, ignored));
    wargv
}

/// Realistic long test-id strings shaped like this project's own.
///
/// A module name, `::`, and a long descriptive `snake_case` sentence, 18 to
/// 105 bytes (the shortest and longest this binary's own `--list` actually
/// printed).
fn long_test_ids(count: usize) -> Vec<String> {
    const WORDS: [&str; 10] = [
        "a", "draw", "into", "a", "target", "cleared", "every", "frame", "is", "built",
    ];
    (0..count)
        .map(|i| {
            let sentence: String = (0..6 + i % 8)
                .map(|w| WORDS[(i + w) % WORDS.len()])
                .collect::<Vec<_>>()
                .join("_");
            format!("module_{i:03}::{sentence}_before_its_frame_is_submitted_{i:03}")
        })
        .collect()
}

/// `long_test_ids`, each followed by two very short names a couple of units apart in length.
///
/// `fitting_prefix` selects names greedily, one at a time, and stops at the
/// first one that does not fit: a batch of only long realistic names stops
/// wherever their lengths happen to add up, tens of units short of a
/// process's real capacity, as often as not, because the next one in line
/// is too big to narrow the gap. The two models a selection has to fit
/// under here differ by only a few dozen units, far narrower than that
/// leftover slack usually is, so short names after every long one give the
/// selection a way to keep creeping up to the real ceiling wherever it
/// falls, rather than stopping well short of it.
fn long_test_ids_with_fine_filler(count: usize) -> Vec<String> {
    long_test_ids(count)
        .into_iter()
        .enumerate()
        .flat_map(|(i, long)| [long, format!("f{}", 2 * i), format!("f{}", 2 * i + 1)])
        .collect()
}

/// `fitting_prefix`'s selection must fit under whichever command line a host builds.
///
/// Not just the one this run assumed.
fn assert_selection_fits_either_way(exe: &Path, names: &[String], threads: u32, ignored: bool) {
    let count = fitting_prefix(exe, names, threads, ignored).expect("at least the first name fits");
    assert!(count >= 1, "nothing was selected for a non-empty batch");
    let selected = &names[..count];
    for (built_by, wargv) in [
        (
            "loading the binary directly",
            direct_wargv(exe, selected, threads, ignored),
        ),
        (
            "falling back to start.exe",
            via_start_exe_wargv(exe, selected, threads, ignored),
        ),
    ] {
        let units = wine_command_line_units(&wargv);
        assert!(
            units < COMMAND_LINE_UNITS,
            "fitting_prefix chose {count} of {} names, but Wine's real command line when {built_by} \
             would be {units} units, at or past the {COMMAND_LINE_UNITS}-unit limit",
            names.len(),
        );
    }
}

#[test]
fn fitting_prefix_fits_wines_real_command_line_including_its_start_exe_fallback() {
    // Shaped like the run that actually failed: a long path under a cargo
    // target's debug/deps, this project's own long test-id strings, the
    // i686 target (every i686 process takes Wine's start.exe fallback on a
    // WOW64-only build, so the direct-load spelling alone is not enough),
    // one thread and no `--ignored`.
    let exe = Path::new(
        "/home/build/mtld3d/windows/target/i686-pc-windows-msvc/debug/deps/e2e-1030f4ab05278ecb.exe",
    );
    let names = long_test_ids_with_fine_filler(680);
    assert_selection_fits_either_way(exe, &names, 1, false);

    // A handful of other shapes: a short path and many threads, an ignored
    // run, and a path containing a space (which the start.exe fallback's
    // own, separate exe-path argument must quote on its own, since nothing
    // upstream of it does).
    assert_selection_fits_either_way(Path::new("/suite.exe"), &names, 8, false);
    assert_selection_fits_either_way(exe, &names, 1, true);
    assert_selection_fits_either_way(Path::new("/test path/suite.exe"), &names, 4, false);
}
