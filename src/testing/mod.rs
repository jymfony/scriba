mod inject_helpers;
pub mod uuid;

use ansi_term::Color;
pub use inject_helpers::inject_helpers;
use sha1::{Digest, Sha1};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{env, fs};
use swc_common::Mark;
use swc_ecma_parser::{EsConfig, Syntax};
use swc_ecma_transforms_base::{fixer, hygiene};
use swc_ecma_transforms_testing::{HygieneVisualizer, Tester};
use swc_ecma_visit::{Fold, FoldWith};
use tempfile::tempdir_in;
use testing::find_executable;

fn make_tr<F, P>(op: F, tester: &mut Tester<'_>) -> impl Fold
where
    F: FnOnce(&mut Tester<'_>) -> P,
    P: Fold,
{
    op(tester)
}

fn calc_hash(s: &str) -> String {
    let mut hasher = Sha1::default();
    hasher.update(s.as_bytes());
    let sum = hasher.finalize();

    hex::encode(sum)
}

pub fn compile_tr<F, P>(tr: F, input: &str) -> String
where
    F: FnOnce(&mut Tester<'_>) -> P,
    P: Fold,
{
    Tester::run(|tester| {
        let tr = tr(tester);
        let module = tester.apply_transform(
            tr,
            "input.js",
            Syntax::Es(EsConfig {
                jsx: false,
                fn_bind: false,
                decorators: true,
                decorators_before_export: false,
                export_default_from: false,
                import_attributes: true,
                allow_super_outside_method: false,
                allow_return_outside_function: true,
                auto_accessors: true,
                explicit_resource_management: true,
            }),
            input,
        )?;

        let module = module.fold_with(&mut fixer::fixer(Some(&tester.comments)));

        let src = tester.print(&module, &tester.comments.clone());
        Ok(src)
    })
}

/// Execute `jest` after transpiling `input` using `tr`.
pub fn exec_tr<F, P>(test_name: &str, syntax: Syntax, tr: F, input: &str)
where
    F: FnOnce(&mut Tester<'_>) -> P,
    P: Fold,
{
    Tester::run(|tester| {
        let tr = make_tr(tr, tester);

        let module = tester.apply_transform(
            tr,
            "input.js",
            syntax,
            &format!(
                "it('should work', async function () {{
                    {}
                }})",
                input
            ),
        )?;
        match ::std::env::var("PRINT_HYGIENE") {
            Ok(ref s) if s == "1" => {
                let hygiene_src = tester.print(
                    &module.clone().fold_with(&mut HygieneVisualizer),
                    &tester.comments.clone(),
                );
                println!("----- Hygiene -----\n{}", hygiene_src);
            }
            _ => {}
        }

        let mut module = module
            .fold_with(&mut hygiene::hygiene())
            .fold_with(&mut fixer::fixer(Some(&tester.comments)));

        let src_without_helpers = tester.print(&module, &tester.comments.clone());
        module = module.fold_with(&mut inject_helpers(Mark::fresh(Mark::root())));

        let src = tester.print(&module, &tester.comments.clone());

        println!(
            "\t>>>>> {} <<<<<\n{}\n\t>>>>> {} <<<<<\n{}",
            Color::Green.paint("Orig"),
            input,
            Color::Green.paint("Code"),
            src_without_helpers
        );

        exec_with_node_test_runner(test_name, &src)
    })
}

fn exec_with_node_test_runner(test_name: &str, src: &str) -> Result<(), ()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("testing")
        .join(test_name);

    create_dir_all(&root).expect("failed to create parent directory for temp directory");

    let hash = calc_hash(src);
    let success_cache = root.join(format!("{}.success", hash));

    if env::var("CACHE_TEST").unwrap_or_default() == "1" {
        println!("Trying cache as `CACHE_TEST` is `1`");

        if success_cache.exists() {
            println!("Cache: success");
            return Ok(());
        }
    }

    let tmp_dir = tempdir_in(&root).expect("failed to create a temp directory");
    create_dir_all(&tmp_dir).unwrap();

    let path = tmp_dir.path().join(format!("{}.test.mjs", test_name));

    let mut tmp = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)
        .expect("failed to create a temp file");
    write!(tmp, "{}", src).expect("failed to write to temp file");
    tmp.flush().unwrap();

    let test_runner_path = find_executable("mocha").expect("failed to find `mocha` from path");

    let mut base_cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(&test_runner_path);
        c
    } else {
        Command::new(&test_runner_path)
    };

    let output = base_cmd
        .arg(&format!("{}", path.display()))
        .arg("--color")
        .current_dir(root)
        .output()
        .expect("failed to run mocha");

    println!(">>>>> {} <<<<<", Color::Red.paint("Stdout"));
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!(">>>>> {} <<<<<", Color::Red.paint("Stderr"));
    println!("{}", String::from_utf8_lossy(&output.stderr));

    if output.status.success() {
        fs::write(&success_cache, "").unwrap();
        return Ok(());
    }
    let dir_name = path.display().to_string();
    ::std::mem::forget(tmp_dir);
    panic!("Execution failed: {dir_name}")
}
