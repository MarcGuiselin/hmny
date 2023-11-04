use crate::task::cargo::CargoCommand;

use super::{Handle, Status, StatusSender};
use std::{
    env,
    io::{Error, ErrorKind, Result},
    path::Path,
    sync::Arc,
};
use tokio::{fs, process::Command, sync::Mutex};

/// Steps needed to build a wrap
/// See: https://github.com/polywrap/cli/blob/origin-dev/packages/cli/src/lib/defaults/build-strategies/wasm/rust/local/local.sh
///
/// Requirements:
/// - Command: `rustup target add wasm32-unknown-unknown`
/// - Command: `cargo install wasm-snip`
/// - Command: `cargo install wasm-bindgen-cli`
/// - Crate type of wrap should be `cdylib`
/// - Build with flags (see ../cargo/build_wraps.rs)
///
/// Step 1: Generate Wrap Manifest
/// - Command: `yarn polywrap build --strategy local --no-codegen --manifest-file {} --output-dir {}`
///
/// Step 2: Run wasm-bindgen over the module, replacing all placeholder __wbindgen_... imports
/// - Env: WASM_INTERFACE_TYPES=1
/// - Command: `wasm-bindgen "$1"/target/wasm32-unknown-unknown/release/module.wasm --out-dir "$2" --out-name bg_module.wasm`
///
/// Step 3: Run wasm-tools strip to remove the wasm-interface-types custom section
/// - Command: `wasm-tools strip "$2"/bg_module.wasm -d wasm-interface-types -o "$2"/strip_module.wasm`
/// - Command: `rm -rf "$2"/bg_module.wasm`
///
/// Step 4: Run wasm-snip to trip down the size of the binary, removing any dead code
/// - Command: `wasm-snip "$2"/strip_module.wasm -o "$2"/snipped_module.wasm`
/// - Command: `rm -rf "$2"/strip_module.wasm`
///
/// Step 5: Use wasm-opt to perform the "asyncify" post-processing step over all modules
/// - Env: ASYNCIFY_STACK_SIZE=24576
/// - Command: `wasm-opt --asyncify -Os "$2"/snipped_module.wasm -o "$2"/wrap.wasm`
/// - Command: `rm -rf "$2"/snipped_module.wasm`
#[derive(Debug)]
enum Step {
    Manifest,
    Bindgen,
    Strip,
    Snip,
    Opt,
    Done,
}

struct Inner {
    name: String,
    handle: Handle,
    error: Option<String>,
    step: Step,
}

pub fn start_task(update_sender: StatusSender, name: String) -> Handle {
    println!("Starting compile wrap task for '{}'", name);

    let handle = Handle::new();
    let inner = Arc::new(Mutex::new(Inner {
        name: name.clone(),
        handle: handle.clone(),
        error: None,
        step: Step::Bindgen,
    }));

    let inner_handle = Arc::clone(&inner);
    tauri::async_runtime::spawn(async move {
        match initiate(update_sender.clone(), &inner_handle, name).await {
            Ok(_) => {}
            Err(error) => {
                let mut inner = inner_handle.lock().await;
                inner.error = Some(error.to_string());
                update_sender.send(get_status(&inner)).await.unwrap();
            }
        }
    });

    handle
}

async fn initiate(
    update_sender: StatusSender,
    inner_handle: &Arc<Mutex<Inner>>,
    name: String,
) -> Result<()> {
    let wasm_source_path = format!("../../target/wasm32-unknown-unknown/release/{}.wasm", name);
    let work_dir = format!("../../target/polywrap/artifacts/{}", name);
    let output_dir = format!("../../target/polywrap/release/{}", name);
    let wasm_output_path = format!("{}/wrap.wasm", output_dir);
    let bindgen_output_path = format!("{}/bg_module.wasm", work_dir);
    let strip_output_path = format!("{}/strip_module.wasm", work_dir);
    let snip_output_path = format!("{}/snipped_module.wasm", work_dir);

    // Step 1
    send_status(&update_sender, &inner_handle, Step::Manifest).await;
    delete_directory_contents(&work_dir).await?;
    delete_directory_contents(&output_dir).await?;
    fs::create_dir_all(&output_dir).await?;
    let yarn_executable_path = which::which("yarn")
        .map_err(|_| Error::new(ErrorKind::Other, "`yarn` command not found in PATH."))?;
    let package_path = {
        #[derive(serde::Deserialize)]
        struct Metadata {
            packages: Vec<Package>,
        }

        #[derive(serde::Deserialize)]
        struct Package {
            name: String,
            manifest_path: String,
        }

        let output = CargoCommand::new("metadata")
            .args(&["--no-deps"])
            .command
            .output()
            .await?;

        let metadata: Metadata = serde_json::from_slice(&output.stdout).unwrap();
        let package = metadata
            .packages
            .into_iter()
            .find(|package| package.name == name)
            .ok_or(Error::new(
                ErrorKind::Other,
                format!("Could not find package '{}' in cargo metadata", name),
            ))?;
        Path::new(&package.manifest_path)
            .parent()
            .unwrap()
            .to_path_buf()
    };
    let output_dir_full = env::current_dir()?.join(&output_dir);
    let _ = Command::new(yarn_executable_path)
        .args([
            "polywrap",
            "build",
            // This is a hack right now: running with "local" strategy will always fail on windows
            // This is good because I only need the manifest, and it will fail right after generating it
            // TODO: Find a proper way to generate only the manifest or hack something together
            "--strategy",
            "local",
            "--no-codegen",
            "--output-dir",
            output_dir_full.to_str().unwrap(),
        ])
        .current_dir(package_path)
        .output()
        .await?;

    // Step 2
    run_command(
        Command::new("wasm-bindgen")
            .arg(&wasm_source_path)
            .args(["--out-dir", &work_dir])
            .args(["--out-name", "bg_module.wasm"])
            .env("WASM_INTERFACE_TYPES", "1"),
        Step::Bindgen,
    )
    .await?;

    // Step 3
    send_status(&update_sender, &inner_handle, Step::Strip).await;
    run_command(
        Command::new("wasm-tools")
            .args(["strip", &bindgen_output_path])
            .args(["-d", "wasm-interface-types"])
            .args(["-o", &strip_output_path]),
        Step::Strip,
    )
    .await?;

    // Step 4
    send_status(&update_sender, &inner_handle, Step::Snip).await;
    run_command(
        Command::new("wasm-snip")
            .arg(&strip_output_path)
            .args(["-o", &snip_output_path]),
        Step::Snip,
    )
    .await?;

    // Step 5
    send_status(&update_sender, &inner_handle, Step::Opt).await;
    let _ = fs::remove_file(&wasm_output_path).await;
    env::set_var("ASYNCIFY_STACK_SIZE", "24576");
    wasm_opt::OptimizationOptions::new_optimize_for_size()
        .add_pass(wasm_opt::Pass::Asyncify)
        .run(&snip_output_path, &wasm_output_path)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    // Done
    send_status(&update_sender, &inner_handle, Step::Done).await;

    Ok(())
}

async fn run_command(command: &mut Command, step: Step) -> Result<()> {
    let output = command.output().await?;

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Other,
            format!(
                "Compile wrap step {:?} failed with exit code: {}",
                step, output.status
            ),
        ))
    }
}

async fn send_status(update_sender: &StatusSender, inner_handle: &Arc<Mutex<Inner>>, step: Step) {
    let mut inner = inner_handle.lock().await;
    inner.step = step;
    update_sender.send(get_status(&inner)).await.unwrap();
}

async fn delete_directory_contents(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if let Ok(mut entries) = fs::read_dir(&path).await {
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(path).await?;
            } else {
                fs::remove_file(path).await?;
            }
        }
    }

    Ok(())
}

fn get_status(inner: &Inner) -> Status {
    let title = format!("Compiling Wrap {}", inner.name);
    let status = None;
    let ratio = match inner.step {
        Step::Manifest => 0.0,
        Step::Bindgen => 0.2,
        Step::Strip => 0.4,
        Step::Snip => 0.6,
        Step::Opt => 0.8,
        Step::Done => 1.0,
    };

    Status {
        handle: inner.handle.clone(),
        title,
        status,
        done_ratio: ratio,
        doing_ratio: ratio,
        error: inner.error.clone(),
    }
}
