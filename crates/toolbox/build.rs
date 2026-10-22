use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let icon = manifest_dir.join("../../icons/toolbox.ico");
    println!("cargo:rerun-if-changed={}", icon.display());

    if !icon.exists() {
        println!("cargo:warning=未找到 {}，跳过图标嵌入", icon.display());
        return;
    }

    let Some(rc) = find_rc() else {
        println!("cargo:warning=未找到 rc.exe（Windows SDK 资源编译器），跳过图标嵌入");
        return;
    };

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("缺少 OUT_DIR"));
    let rc_file = out_dir.join("toolbox.rc");
    let res_file = out_dir.join("toolbox.res");

    let icon_path = icon.to_string_lossy().replace('\\', "/");
    if let Err(err) = std::fs::write(&rc_file, format!("1 ICON \"{icon_path}\"\n")) {
        println!("cargo:warning=写入 .rc 失败：{err}");
        return;
    }

    let output = match Command::new(&rc)
        .arg("/nologo")
        .arg("/fo")
        .arg(&res_file)
        .arg(&rc_file)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            println!("cargo:warning=调用 rc.exe 失败：{err}");
            return;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("cargo:warning=rc.exe 执行失败：{stderr}");
        return;
    }

    println!("cargo:rustc-link-arg-bin=toolbox={}", res_file.display());
}

fn find_rc() -> Option<PathBuf> {
    for key in ["RC_EXE", "RC"] {
        if let Some(path) = std::env::var_os(key) {
            let path = PathBuf::from(path);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    if let Some(path) = which_rc() {
        return Some(path);
    }

    let arch = match std::env::var("PROCESSOR_ARCHITECTURE").as_deref() {
        Ok("ARM64") => "arm64",
        _ => "x64",
    };

    for root in sdk_roots() {
        let bin = root.join("bin");
        let mut versions: Vec<PathBuf> = match std::fs::read_dir(&bin) {
            Ok(entries) => entries
                .filter_map(Result::ok)
                .map(|entry| entry.path().join(arch).join("rc.exe"))
                .filter(|candidate| candidate.is_file())
                .collect(),
            Err(_) => continue,
        };
        versions.sort_by(|a, b| b.cmp(a));
        if let Some(path) = versions.into_iter().next() {
            return Some(path);
        }
    }

    None
}

fn which_rc() -> Option<PathBuf> {
    let output = Command::new("where").args(["rc.exe"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

fn sdk_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for key in ["WindowsSdkDir", "WindowsSdkVerBinPath"] {
        if let Some(value) = std::env::var_os(key) {
            let path = PathBuf::from(value);
            roots.push(if key == "WindowsSdkVerBinPath" {
                path.ancestors()
                    .nth(3)
                    .map(Path::to_path_buf)
                    .unwrap_or(path)
            } else {
                path
            });
        }
    }

    for base in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(dir) = std::env::var_os(base) {
            roots.push(PathBuf::from(dir).join("Windows Kits").join("10"));
        }
    }

    roots.into_iter().filter(|root| root.is_dir()).collect()
}
