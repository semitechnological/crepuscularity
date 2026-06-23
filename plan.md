1. **Understand the Vulnerability**
   - The issue highlights command injection via `cmd /C <script>` in `crates/crepuscularity-cli/src/benchmark.rs:1088`.
   - When running on Windows (`#[cfg(not(unix))]`), `run_shell` creates a process using `Command::new("cmd")` and calls `.args(["/C", script])`.
   - `script` is a shell script from the benchmark configuration, typically expected to contain multiple commands or shell features.
   - Using `cmd /C` with an unescaped string can lead to unexpected command execution or injection if the script variable contains unexpected characters or if we want to ensure it runs as a script safely.
   - However, unlike passing separate arguments, `cmd /C` expects the entire command string. Passing multiple arguments or a single complex string to `cmd /C` in Rust `Command` can lead to quoting issues or unintended splitting because Rust tries to escape arguments for Windows.
   - To fix this securely and reliably without changing the functionality of running a user-provided script in `cmd`, it is safer to write the `script` content to a temporary `.bat` or `.cmd` file and execute that file instead, or use proper escaping if writing a file is not feasible. Writing a temporary batch file ensures the script is interpreted exactly as intended by Windows without `cmd /C` inline quoting issues.
   - Wait, `run_shell` is explicitly meant to run a shell script. On Unix it runs `sh -c script`. On Windows, the best equivalent is usually running `cmd.exe /C "..."`. But Rust's `Command` on Windows does its own escaping. If `script` contains `&`, `"`, etc., it can be mangled.
   - A better way to execute a complex shell script string in Windows with `cmd` safely using `Command` is to write it to a temporary `.bat` file or use `raw_arg` (which requires `std::os::windows::process::CommandExt`).
   - If we look at the issue: "Command Injection in `run_shell` via `cmd /C` ... Similar to the `sh -c` issue, `cmd /C` receives unescaped input. It can be remediated with proper command string formatting or a safer process launching mechanism."
   - Let's create a temporary batch file, write the script to it, and execute the batch file using `cmd /C temp.bat`. Or just execute the batch file directly.

2. **Develop the Fix**
   - In `run_shell`, specifically in the `#[cfg(not(unix))]` block:
   - Generate a temporary file name (e.g., using `tempfile` crate, or putting it in `workdir`).
   - We already have `tempfile` in dev-dependencies, but let's see if it's available in dependencies. No, `tempfile` is not in `dependencies` for `crepuscularity-cli`.
   - Without `tempfile`, we can create a `.bat` file in `workdir` (e.g., `.crepus_bench_script_<random>.bat` or using a timestamp/hash).
   - Alternatively, can we just use `CommandExt::raw_arg` to format the `cmd /C` string safely?
   - Actually, Rust's `Command` tries to escape arguments, which often breaks `cmd /C`. The standard way to fix this in Rust when you *want* `cmd` to interpret it as a command line is to use `cmd.raw_arg(format!("/C \"{}\"", script))`. But wait, the issue explicitly says "receives unescaped input. It can be remediated with proper command string formatting or a safer process launching mechanism."
   - Wait, another way: just write to a batch file in `workdir`.
   - Let's do:
     ```rust
     let script_path = workdir.join(format!("crepus_bench_{}.bat", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
     std::fs::write(&script_path, script).unwrap_or_default();
     let mut cmd = Command::new("cmd");
     cmd.args(["/C", script_path.to_str().unwrap()]).current_dir(workdir).envs(envs);
     ```
   - Make sure to clean up the batch file afterwards (both in `Ok` and `Err` paths).

3. **Verify the Fix**
   - Run `cargo check -p crepuscularity-cli`
   - Run `cargo test -p crepuscularity-cli`

4. **Complete Pre-commit Steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5. **Submit PR**
   - Create PR with title `🔒 [Fix command injection in run_shell on Windows]` and description formatted properly.
