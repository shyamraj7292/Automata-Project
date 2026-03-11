/// Integration tests for automata-mini-compiler
///
/// Each test compiles a sample source file end-to-end and asserts
/// the compiler finishes without panicking.  Extend with output
/// assertions once a MARS runner is wired in.

use std::process::Command;

/// Helper: run the release binary on a given source file.
fn run_compiler(src: &str, optimize: bool) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_compiler"));
    cmd.arg(src);
    if optimize {
        cmd.arg("opt");
    }
    cmd.output().expect("failed to launch compiler binary")
}

#[test]
fn test_hello_world_compiles() {
    let out = run_compiler("test/in/helloworld.txt", false);
    assert!(
        out.status.success(),
        "helloworld failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_factorial_compiles() {
    let out = run_compiler("test/in/fact.txt", false);
    assert!(
        out.status.success(),
        "factorial failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_factorial_with_optimization() {
    let out = run_compiler("test/in/fact.txt", true);
    assert!(
        out.status.success(),
        "factorial (opt) failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_arrange_compiles() {
    let out = run_compiler("test/in/arrange.txt", false);
    assert!(
        out.status.success(),
        "arrange failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_merge_sort_compiles() {
    let out = run_compiler("test/in/merge_sort.txt", false);
    assert!(
        out.status.success(),
        "merge_sort failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn test_contest_compiles() {
    let out = run_compiler("test/in/contest.txt", false);
    assert!(
        out.status.success(),
        "contest failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
