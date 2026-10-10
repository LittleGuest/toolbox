use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::{Error, Result};
use encoding_rs::GB18030;
use ignore::{
    Match,
    gitignore::{Gitignore, GitignoreBuilder},
};

pub const OUTPUT_MARK: &str = "去空行";

pub const TEXT_EXTENSIONS: &[&str] = &[
    "md",
    "markdown",
    "mdx",
    "txt",
    "text",
    "log",
    "rst",
    "adoc",
    "org",
    "html",
    "htm",
    "xhtml",
    "xml",
    "svg",
    "xsl",
    "xslt",
    "xsd",
    "plist",
    "rss",
    "atom",
    "json",
    "json5",
    "jsonc",
    "jsonl",
    "ndjson",
    "geojson",
    "ipynb",
    "har",
    "yaml",
    "yml",
    "toml",
    "ini",
    "cfg",
    "conf",
    "config",
    "properties",
    "env",
    "editorconfig",
    "lock",
    "gitignore",
    "gitattributes",
    "dockerignore",
    "npmrc",
    "csv",
    "tsv",
    "psv",
    "srt",
    "vtt",
    "ass",
    "ssa",
    "diff",
    "patch",
    "tex",
    "bib",
    "rs",
    "js",
    "mjs",
    "cjs",
    "jsx",
    "ts",
    "tsx",
    "mts",
    "cts",
    "vue",
    "svelte",
    "astro",
    "css",
    "scss",
    "sass",
    "less",
    "styl",
    "pcss",
    "postcss",
    "py",
    "pyi",
    "rb",
    "erb",
    "php",
    "go",
    "java",
    "kt",
    "kts",
    "scala",
    "groovy",
    "gradle",
    "clj",
    "cljs",
    "edn",
    "ex",
    "exs",
    "erl",
    "hrl",
    "lua",
    "pl",
    "pm",
    "c",
    "h",
    "cc",
    "cpp",
    "cxx",
    "c++",
    "hpp",
    "hxx",
    "hh",
    "cs",
    "fs",
    "vb",
    "m",
    "mm",
    "swift",
    "dart",
    "r",
    "jl",
    "nim",
    "zig",
    "v",
    "sol",
    "asm",
    "s",
    "sh",
    "bash",
    "zsh",
    "fish",
    "ksh",
    "ps1",
    "psm1",
    "bat",
    "cmd",
    "awk",
    "sed",
    "sql",
    "graphql",
    "gql",
    "proto",
    "tf",
    "tfvars",
    "hcl",
    "nix",
    "dockerfile",
    "makefile",
    "mk",
    "cmake",
    "bazel",
    "bzl",
    "sbt",
    "pom",
    "coveragerc",
];

const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "ico", "icns", "tif", "tiff", "heic", "heif",
    "avif", "raw", "psd", "ai", "sketch", "fig", "blend", "xcf", "pdf", "doc", "docx", "xls",
    "xlsx", "ppt", "pptx", "odt", "ods", "odp", "rtf", "zip", "gz", "tgz", "bz2", "xz", "zst",
    "7z", "rar", "tar", "lz", "lzma", "jar", "war", "ear", "apk", "ipa", "deb", "rpm", "msi",
    "pkg", "snap", "mp3", "wav", "flac", "aac", "ogg", "oga", "opus", "m4a", "wma", "mid", "mp4",
    "mkv", "avi", "mov", "webm", "wmv", "flv", "m4v", "mpg", "mpeg", "rmvb", "exe", "dll", "so",
    "dylib", "bin", "o", "obj", "a", "lib", "class", "pyc", "pyo", "pyd", "wasm", "node", "rlib",
    "rmeta", "d", "pdb", "ttf", "otf", "woff", "woff2", "eot", "fnt", "db", "db3", "sqlite",
    "sqlite3", "mdb", "accdb", "realm", "iso", "dmg", "img", "vhd", "vmdk", "qcow2", "dat", "pack",
    "idx", "key", "keystore", "p12", "pfx", "cer", "der",
];

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "dist",
    "build",
    "vendor",
    "venv",
    "__pycache__",
    "Pods",
    "coverage",
];

const SNIFF_BYTES: usize = 8192;

const VCS_MARKERS: [(&str, VcsKind); 2] = [(".git", VcsKind::Git), (".svn", VcsKind::Svn)];

const VCS_DIRS: [&str; 4] = [".git", ".svn", ".hg", ".bzr"];

const EXCLUDE_FILE: &str = ".git/info/exclude";

const IGNORE_FILE: &str = ".gitignore";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VcsKind {
    Git,
    Svn,
}

impl VcsKind {
    pub fn tag(self) -> &'static str {
        match self {
            VcsKind::Git => "Git 工程",
            VcsKind::Svn => "SVN 工程",
        }
    }

    fn runs_gitignore(self) -> bool {
        matches!(self, VcsKind::Git)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcsInfo {
    pub kind: VcsKind,
    pub root: PathBuf,
}

#[derive(Clone, Debug, Default)]
pub struct ScanResult {
    pub files: Vec<String>,
    pub ignored: usize,
    pub ignored_dirs: usize,
}

impl ScanResult {
    pub fn ignored_total(&self) -> usize {
        self.ignored + self.ignored_dirs
    }
}

pub fn detect_vcs(path: &Path) -> Option<VcsInfo> {
    let start = if path.is_dir() { path } else { path.parent()? };

    start.ancestors().find_map(|dir| {
        is_repo_root(dir).map(|kind| VcsInfo {
            kind,
            root: dir.to_path_buf(),
        })
    })
}

fn is_repo_root(dir: &Path) -> Option<VcsKind> {
    VCS_MARKERS
        .iter()
        .find(|(marker, _)| dir.join(marker).exists())
        .map(|(_, kind)| *kind)
}

fn is_vcs_dir(name: &str) -> bool {
    VCS_DIRS.contains(&name)
}

struct IgnoreLayer {
    base: PathBuf,
    matcher: Option<Gitignore>,
}

fn build_layer(base: &Path, files: &[PathBuf]) -> Option<IgnoreLayer> {
    let mut builder = GitignoreBuilder::new(base);
    let mut loaded = false;

    for file in files {
        if file.is_file() && builder.add(file).is_none() {
            loaded = true;
        }
    }

    loaded.then(|| IgnoreLayer {
        base: base.to_path_buf(),
        matcher: builder.build().ok(),
    })
}

fn build_layer_at(dir: &Path, repo_kind: Option<VcsKind>) -> Option<IgnoreLayer> {
    let mut files = Vec::new();
    if repo_kind.is_some_and(VcsKind::runs_gitignore) {
        files.push(dir.join(EXCLUDE_FILE));
    }
    files.push(dir.join(IGNORE_FILE));
    build_layer(dir, &files)
}

fn dir_layer(dir: &Path) -> Option<IgnoreLayer> {
    build_layer_at(dir, is_repo_root(dir))
}

fn ancestor_layers(dir: &Path) -> Vec<IgnoreLayer> {
    let Some(info) = detect_vcs(dir) else {
        return Vec::new();
    };

    let mut chain: Vec<PathBuf> = Vec::new();
    let mut cursor = dir.parent();
    while let Some(parent) = cursor {
        if !parent.starts_with(&info.root) {
            break;
        }
        chain.push(parent.to_path_buf());
        if parent == info.root {
            break;
        }
        cursor = parent.parent();
    }
    chain.reverse();

    chain
        .iter()
        .filter_map(|base| {
            let kind = (*base == info.root).then_some(info.kind);
            build_layer_at(base, kind)
        })
        .collect()
}

fn is_ignored(layers: &[IgnoreLayer], path: &Path, is_dir: bool) -> bool {
    for layer in layers.iter().rev() {
        let Some(matcher) = &layer.matcher else {
            continue;
        };
        if !path.starts_with(&layer.base) {
            continue;
        }
        match matcher.matched_path_or_any_parents(path, is_dir) {
            Match::Ignore(_) => return true,
            Match::Whitelist(_) => return false,
            Match::None => {}
        }
    }
    false
}

struct Scanner {
    files: Vec<PathBuf>,
    ignored: usize,
    ignored_dirs: usize,
    layers: Vec<IgnoreLayer>,
}

impl Scanner {
    fn walk(&mut self, dir: &Path) {
        let pushed = match dir_layer(dir) {
            Some(layer) => {
                self.layers.push(layer);
                true
            }
            None => false,
        };

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                let path = entry.path();

                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if is_vcs_dir(&name) {
                        continue;
                    }
                    if is_ignored(&self.layers, &path, true) {
                        self.ignored_dirs += 1;
                        continue;
                    }
                    if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
                        continue;
                    }
                    self.walk(&path);
                } else if file_type.is_file() {
                    if is_generated_output(&path) || !is_processable(&path) {
                        continue;
                    }
                    if is_ignored(&self.layers, &path, false) {
                        self.ignored += 1;
                        continue;
                    }
                    self.files.push(path);
                }
            }
        }

        if pushed {
            self.layers.pop();
        }
    }
}

pub fn scan(inputs: &[String]) -> ScanResult {
    let mut scanner = Scanner {
        files: Vec::new(),
        ignored: 0,
        ignored_dirs: 0,
        layers: Vec::new(),
    };
    let mut result = ScanResult::default();
    let mut seen: HashSet<String> = HashSet::new();

    for raw in inputs {
        let path = PathBuf::from(raw);
        if path.is_dir() {
            scanner.layers = ancestor_layers(&path);
            scanner.walk(&path);

            result.ignored += scanner.ignored;
            result.ignored_dirs += scanner.ignored_dirs;
            scanner.ignored = 0;
            scanner.ignored_dirs = 0;

            let mut found = std::mem::take(&mut scanner.files);
            found.sort();
            for file in found {
                let value = file.to_string_lossy().to_string();
                if seen.insert(value.clone()) {
                    result.files.push(value);
                }
            }
        } else if is_processable(&path) && !is_generated_output(&path) && seen.insert(raw.clone()) {
            result.files.push(raw.clone());
        }
    }

    result
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlankMode {
    RemoveAll,
    Collapse,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OutputMode {
    NewFile,
    Overwrite,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileState {
    Done,
    Unchanged,
    Failed,
}

#[derive(Clone, Debug)]
pub struct FileReport {
    pub path: String,
    pub output: Option<String>,
    pub state: FileState,
    pub removed: usize,
    pub detail: Option<String>,
}

impl FileReport {
    fn failed(path: String, detail: String) -> Self {
        Self {
            path,
            output: None,
            state: FileState::Failed,
            removed: 0,
            detail: Some(detail),
        }
    }
}

fn extension_of(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
}

pub fn is_text_extension(path: &Path) -> bool {
    extension_of(path)
        .map(|ext| TEXT_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

pub fn is_binary_extension(path: &Path) -> bool {
    extension_of(path)
        .map(|ext| BINARY_EXTENSIONS.contains(&ext.as_str()))
        .unwrap_or(false)
}

pub fn bytes_look_text(bytes: &[u8]) -> bool {
    let sample = &bytes[..bytes.len().min(SNIFF_BYTES)];
    if sample.is_empty() {
        return true;
    }
    if sample.contains(&0) {
        return false;
    }
    match std::str::from_utf8(sample) {
        Ok(_) => true,
        Err(error) if error.error_len().is_none() && error.valid_up_to() + 3 >= sample.len() => {
            true
        }
        Err(_) => {
            let suspicious = sample
                .iter()
                .filter(|byte| **byte < 0x20 && !(0x09..=0x0D).contains(&**byte))
                .count();
            suspicious * 100 < sample.len()
        }
    }
}

fn file_looks_text(path: &Path) -> bool {
    use std::io::Read as _;

    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut buffer = vec![0u8; SNIFF_BYTES];
    let Ok(read) = file.read(&mut buffer) else {
        return false;
    };
    buffer.truncate(read);
    bytes_look_text(&buffer)
}

pub fn is_processable(path: &Path) -> bool {
    if is_binary_extension(path) {
        return false;
    }
    if is_text_extension(path) {
        return true;
    }
    if !path.is_file() {
        return false;
    }
    file_looks_text(path)
}

fn is_generated_output(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.ends_with(".bak") || name.contains(OUTPUT_MARK)
}

pub fn collect_files(inputs: &[String]) -> Vec<String> {
    scan(inputs).files
}

pub fn output_path_for(src: &Path) -> PathBuf {
    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("output");
    let name = match src.extension().and_then(|s| s.to_str()) {
        Some(ext) if !ext.is_empty() => format!("{stem}.{OUTPUT_MARK}.{ext}"),
        _ => format!("{stem}.{OUTPUT_MARK}"),
    };
    match src.parent() {
        Some(parent) => parent.join(name),
        None => PathBuf::from(name),
    }
}

pub fn backup_path_for(src: &Path) -> PathBuf {
    let name = src
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "backup".to_string());
    let bak = format!("{name}.bak");
    match src.parent() {
        Some(parent) => parent.join(bak),
        None => PathBuf::from(bak),
    }
}

pub fn clean_content(content: &str, mode: BlankMode) -> (String, usize) {
    if content.is_empty() {
        return (String::new(), 0);
    }

    let eol = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let had_trailing = content.ends_with('\n');
    let mut body = content;
    if had_trailing {
        body = &body[..body.len() - 1];
    }
    if let Some(stripped) = body.strip_suffix('\r') {
        body = stripped;
    }
    let lines: Vec<&str> = body
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();

    let mut kept: Vec<&str> = Vec::with_capacity(lines.len());
    let mut removed = 0usize;

    match mode {
        BlankMode::RemoveAll => {
            for line in &lines {
                if line.trim().is_empty() {
                    removed += 1;
                } else {
                    kept.push(line);
                }
            }
        }
        BlankMode::Collapse => {
            let mut run = 0usize;
            for line in &lines {
                if line.trim().is_empty() {
                    run += 1;
                    continue;
                }
                if kept.is_empty() {
                    removed += run;
                } else if run > 0 {
                    kept.push("");
                    removed += run - 1;
                }
                run = 0;
                kept.push(line);
            }
            removed += run;
        }
    }

    let mut out = kept.join(eol);
    if had_trailing && !out.is_empty() {
        out.push_str(eol);
    }
    (out, removed)
}

fn decode_text(bytes: &[u8]) -> (String, bool) {
    let (body, bom) = match bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        Some(rest) => (rest, true),
        None => (bytes, false),
    };

    match std::str::from_utf8(body) {
        Ok(text) => (text.to_string(), bom),
        Err(_) => {
            let (decoded, _, _) = GB18030.decode(body);
            (decoded.into_owned(), bom)
        }
    }
}

pub fn process_one(src: &Path, mode: BlankMode, output_mode: OutputMode) -> FileReport {
    let path = src.to_string_lossy().to_string();

    if !src.is_file() {
        return FileReport::failed(path, "文件不存在或不是普通文件".to_string());
    }

    let bytes = match std::fs::read(src) {
        Ok(bytes) => bytes,
        Err(e) => return FileReport::failed(path, format!("读取失败：{e}")),
    };

    if !bytes_look_text(&bytes) {
        return FileReport::failed(path, "疑似二进制文件，已跳过".to_string());
    }

    let (text, bom) = decode_text(&bytes);
    let (cleaned, removed) = clean_content(&text, mode);

    if removed == 0 {
        return FileReport {
            path,
            output: None,
            state: FileState::Unchanged,
            removed: 0,
            detail: None,
        };
    }

    let mut payload = Vec::with_capacity(cleaned.len() + 3);
    if bom {
        payload.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }
    payload.extend_from_slice(cleaned.as_bytes());

    let target = match output_mode {
        OutputMode::NewFile => output_path_for(src),
        OutputMode::Overwrite => {
            let backup = backup_path_for(src);
            if let Err(e) = std::fs::write(&backup, &bytes) {
                return FileReport::failed(path, format!("备份失败：{e}"));
            }
            src.to_path_buf()
        }
    };

    if let Err(e) = std::fs::write(&target, &payload) {
        return FileReport::failed(path, format!("写入失败：{e}"));
    }

    FileReport {
        path,
        output: Some(target.to_string_lossy().to_string()),
        state: FileState::Done,
        removed,
        detail: None,
    }
}

pub fn process_files(
    inputs: &[String],
    mode: BlankMode,
    output_mode: OutputMode,
) -> Result<Vec<FileReport>> {
    let files = collect_files(inputs);
    if files.is_empty() {
        return Err(Error::msg("没有找到可处理的文本文件".to_string()));
    }

    Ok(files
        .iter()
        .map(|path| process_one(Path::new(path), mode, output_mode))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_all_blank_lines() {
        let (out, removed) = clean_content("a\n\nb\n\n\nc", BlankMode::RemoveAll);
        assert_eq!(out, "a\nb\nc");
        assert_eq!(removed, 3);
    }

    #[test]
    fn remove_all_keeps_trailing_newline() {
        let (out, removed) = clean_content("a\n\n\n", BlankMode::RemoveAll);
        assert_eq!(out, "a\n");
        assert_eq!(removed, 2);
    }

    #[test]
    fn collapse_keeps_single_separator() {
        let (out, removed) = clean_content("a\n\n\n\nb", BlankMode::Collapse);
        assert_eq!(out, "a\n\nb");
        assert_eq!(removed, 2);
    }

    #[test]
    fn collapse_trims_leading_and_trailing_blanks() {
        let (out, removed) = clean_content("\n\n\na\n\nb\n\n\n", BlankMode::Collapse);
        assert_eq!(out, "a\n\nb\n");
        assert_eq!(removed, 5);
    }

    #[test]
    fn whitespace_only_lines_count_as_blank() {
        let (out, removed) = clean_content("a\n   \n\t\nb", BlankMode::RemoveAll);
        assert_eq!(out, "a\nb");
        assert_eq!(removed, 2);
    }

    #[test]
    fn keeps_crlf_style() {
        let (out, removed) = clean_content("a\r\n\r\nb\r\n", BlankMode::RemoveAll);
        assert_eq!(out, "a\r\nb\r\n");
        assert_eq!(removed, 1);
    }

    #[test]
    fn indent_and_content_untouched() {
        let (out, _) = clean_content("  code()  \n\n\ttab", BlankMode::RemoveAll);
        assert_eq!(out, "  code()  \n\ttab");
    }

    #[test]
    fn empty_input_is_untouched() {
        let (out, removed) = clean_content("", BlankMode::RemoveAll);
        assert_eq!(out, "");
        assert_eq!(removed, 0);
    }

    #[test]
    fn no_blank_lines_reports_zero() {
        let (out, removed) = clean_content("a\nb\nc\n", BlankMode::RemoveAll);
        assert_eq!(out, "a\nb\nc\n");
        assert_eq!(removed, 0);
    }

    #[test]
    fn output_and_backup_paths() {
        assert_eq!(
            output_path_for(Path::new("/tmp/readme.md")),
            PathBuf::from("/tmp/readme.去空行.md")
        );
        assert_eq!(
            output_path_for(Path::new("/tmp/a.b.txt")),
            PathBuf::from("/tmp/a.b.去空行.txt")
        );
        assert_eq!(
            backup_path_for(Path::new("/tmp/readme.md")),
            PathBuf::from("/tmp/readme.md.bak")
        );
    }

    #[test]
    fn text_extension_detection() {
        assert!(is_text_extension(Path::new("a.md")));
        assert!(is_text_extension(Path::new("a.TXT")));
        assert!(is_text_extension(Path::new("a.Markdown")));
        assert!(is_text_extension(Path::new("a.html")));
        assert!(is_text_extension(Path::new("a.xml")));
        assert!(is_text_extension(Path::new("a.json")));
        assert!(is_text_extension(Path::new("a.rs")));
        assert!(is_text_extension(Path::new("a.js")));
        assert!(is_text_extension(Path::new("a.yaml")));
        assert!(is_text_extension(Path::new("a.toml")));
        assert!(!is_text_extension(Path::new("a.pdf")));
        assert!(!is_text_extension(Path::new("a.png")));
        assert!(!is_text_extension(Path::new("a")));
    }

    #[test]
    fn binary_extension_detection() {
        assert!(is_binary_extension(Path::new("a.png")));
        assert!(is_binary_extension(Path::new("a.PDF")));
        assert!(is_binary_extension(Path::new("a.zip")));
        assert!(is_binary_extension(Path::new("a.mkv")));
        assert!(!is_binary_extension(Path::new("a.txt")));
        assert!(!is_binary_extension(Path::new("a.rs")));
    }

    #[test]
    fn content_sniffing() {
        assert!(bytes_look_text(b"{\n  \"a\": 1\n}\n"));
        assert!(bytes_look_text("中文内容\n".as_bytes()));
        assert!(bytes_look_text("GBK 中文".as_bytes()));
        assert!(bytes_look_text(&[]));
        assert!(!bytes_look_text(&[
            0x89, 0x50, 0x4E, 0x47, 0x00, 0x0D, 0x0A
        ]));
        let noisy: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
        assert!(!bytes_look_text(&noisy));
        let gbk = vec![0xD6u8, 0xD0u8, 0xCEu8, 0xC4u8, 0x0Au8];
        assert!(bytes_look_text(&gbk));
    }

    #[test]
    fn unknown_extension_falls_back_to_sniff() {
        let dir = std::env::temp_dir().join("toolbox-blank-line-sniff");
        std::fs::create_dir_all(&dir).unwrap();

        let conf = dir.join("nginx.conf.unknown");
        std::fs::write(&conf, "server {\n\n  listen 80;\n}\n").unwrap();
        assert!(is_processable(&conf));

        let blob = dir.join("blob.weird");
        std::fs::write(&blob, [0u8, 1, 2, 3, 0, 5]).unwrap();
        assert!(!is_processable(&blob));

        let bare = dir.join("Makefile");
        std::fs::write(&bare, "all:\n\n\tcargo build\n").unwrap();
        assert!(is_processable(&bare));

        let named_png = dir.join("fake.png");
        std::fs::write(&named_png, "not really a png\n").unwrap();
        assert!(!is_processable(&named_png));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn collect_files_mixes_extensions_and_skips_binary() {
        let dir = std::env::temp_dir().join("toolbox-blank-line-collect");
        std::fs::remove_dir_all(&dir).ok();
        let nested = dir.join("nested");
        std::fs::create_dir_all(&nested).unwrap();

        std::fs::write(dir.join("page.html"), "<h1>x</h1>\n").unwrap();
        std::fs::write(dir.join("data.json"), "{}\n").unwrap();
        std::fs::write(nested.join("main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(nested.join("logo.png"), [0x89, 0x50, 0x4E, 0x47, 0x00]).unwrap();
        std::fs::create_dir_all(nested.join("node_modules")).unwrap();
        std::fs::write(nested.join("node_modules/dep.js"), "a\n").unwrap();

        let inputs = vec![dir.to_string_lossy().to_string()];
        let found = collect_files(&inputs);
        let names: Vec<String> = found
            .iter()
            .map(|path| {
                Path::new(path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        assert_eq!(names.len(), 3);
        assert!(names.contains(&"page.html".to_string()));
        assert!(names.contains(&"data.json".to_string()));
        assert!(names.contains(&"main.rs".to_string()));
        assert!(!names.contains(&"logo.png".to_string()));
        assert!(!names.contains(&"dep.js".to_string()));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn end_to_end_mixed_types_folder() {
        let dir = std::env::temp_dir().join("toolbox-blank-line-e2e");
        std::fs::remove_dir_all(&dir).ok();
        let nested = dir.join("src");
        std::fs::create_dir_all(&nested).unwrap();

        let samples: [(&Path, &str, &str); 7] = [
            (
                Path::new("index.html"),
                "<html>\n\n<body>\n\n\n<p>hi</p>\n</body>\n</html>\n",
                "<html>\n<body>\n<p>hi</p>\n</body>\n</html>\n",
            ),
            (
                Path::new("feed.xml"),
                "<?xml version=\"1.0\"?>\n\n<root>\n\n\t<item/>\n\n</root>\n",
                "<?xml version=\"1.0\"?>\n<root>\n\t<item/>\n</root>\n",
            ),
            (
                Path::new("data.json"),
                "{\n\n  \"a\": 1,\n\n\n  \"b\": [\n\n    2\n  ]\n}\n",
                "{\n  \"a\": 1,\n  \"b\": [\n    2\n  ]\n}\n",
            ),
            (
                Path::new("config.yaml"),
                "server:\n\n\n  port: 80\n\n  host: 0.0.0.0\n",
                "server:\n  port: 80\n  host: 0.0.0.0\n",
            ),
            (
                Path::new("main.rs"),
                "fn main() {\n\n    println!(\"hi\");\n\n}\n",
                "fn main() {\n    println!(\"hi\");\n}\n",
            ),
            (
                Path::new("app.js"),
                "const a = 1;\n\n\nconsole.log(a);\n",
                "const a = 1;\nconsole.log(a);\n",
            ),
            (Path::new("notes"), "line1\n\n\n\nline2\n", "line1\nline2\n"),
        ];

        for (name, content, _) in samples {
            let target = if name.ends_with("main.rs") {
                nested.join(name)
            } else {
                dir.join(name)
            };
            std::fs::write(target, content).unwrap();
        }

        let reports = process_files(
            &[dir.to_string_lossy().to_string()],
            BlankMode::RemoveAll,
            OutputMode::NewFile,
        )
        .unwrap();

        assert_eq!(reports.len(), 7);
        assert!(reports.iter().all(|r| r.state == FileState::Done));

        for (name, source, expected) in samples {
            let target = if name.ends_with("main.rs") {
                nested.join(name)
            } else {
                dir.join(name)
            };
            assert_eq!(std::fs::read_to_string(&target).unwrap(), source);

            let parent = if name.ends_with("main.rs") {
                nested.to_path_buf()
            } else {
                dir.to_path_buf()
            };
            let output = output_path_for(&target);
            assert_eq!(std::fs::read_to_string(&output).unwrap(), expected);
            assert_eq!(output.parent().unwrap().to_path_buf(), parent);
        }

        assert_eq!(
            std::fs::read_to_string(dir.join("index.去空行.html"))
                .unwrap()
                .lines()
                .count(),
            5
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn generated_outputs_are_skipped() {
        let dir = std::env::temp_dir().join("toolbox-blank-line-generated");
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("note.md"), "a\n\nb\n").unwrap();
        std::fs::write(dir.join("note.去空行.md"), "a\nb\n").unwrap();
        std::fs::write(dir.join("note.md.bak"), "a\n\nb\n").unwrap();

        let found = collect_files(&[dir.to_string_lossy().to_string()]);
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("note.md"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn process_one_refuses_binary_masquerade() {
        let dir = std::env::temp_dir().join("toolbox-blank-line-binary");
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("payload.txt");
        std::fs::write(&src, [0x00u8, 0x01, 0x02, 0x00, 0x03]).unwrap();

        let report = process_one(&src, BlankMode::RemoveAll, OutputMode::Overwrite);
        assert_eq!(report.state, FileState::Failed);
        assert!(report.detail.unwrap().contains("二进制"));
        assert_eq!(
            std::fs::read(&src).unwrap(),
            [0x00u8, 0x01, 0x02, 0x00, 0x03]
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn roundtrip_process_one() {
        let dir = std::env::temp_dir().join("toolbox-blank-line-test");
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("sample.md");
        std::fs::write(&src, "# 标题\n\n\n正文\n\n").unwrap();

        let report = process_one(&src, BlankMode::RemoveAll, OutputMode::NewFile);
        assert_eq!(report.state, FileState::Done);
        assert_eq!(report.removed, 3);
        let written = std::fs::read_to_string(dir.join("sample.去空行.md")).unwrap();
        assert_eq!(written, "# 标题\n正文\n");
        assert_eq!(
            std::fs::read_to_string(&src).unwrap(),
            "# 标题\n\n\n正文\n\n",
            "原文件不应被改动"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn overwrite_creates_backup() {
        let dir = std::env::temp_dir().join("toolbox-blank-line-backup");
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("sample.txt");
        std::fs::write(&src, "a\n\n\nb\n").unwrap();

        let report = process_one(&src, BlankMode::RemoveAll, OutputMode::Overwrite);
        assert_eq!(report.state, FileState::Done);
        assert_eq!(std::fs::read_to_string(&src).unwrap(), "a\nb\n");
        assert_eq!(
            std::fs::read_to_string(dir.join("sample.txt.bak")).unwrap(),
            "a\n\n\nb\n"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    fn touch(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, body).unwrap();
    }

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn scan_names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = scan(&[dir.to_string_lossy().to_string()])
            .files
            .iter()
            .map(|path| {
                Path::new(path)
                    .strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        names.sort();
        names
    }

    fn git_available() -> bool {
        std::process::Command::new("git")
            .arg("--version")
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn detect_vcs_finds_git_and_svn_roots() {
        let base = temp_root("toolbox-blank-line-vcs");
        let git = base.join("gitproj");
        let svn = base.join("svnproj");
        let plain = base.join("plain");

        std::fs::create_dir_all(git.join(".git")).unwrap();
        std::fs::create_dir_all(git.join("src/deep")).unwrap();
        std::fs::create_dir_all(svn.join(".svn")).unwrap();
        std::fs::create_dir_all(svn.join("trunk/deep")).unwrap();
        std::fs::create_dir_all(&plain).unwrap();

        let found = detect_vcs(&git.join("src/deep")).unwrap();
        assert_eq!(found.kind, VcsKind::Git);
        assert_eq!(found.root, git);
        assert_eq!(found.kind.tag(), "Git 工程");

        assert_eq!(
            detect_vcs(&svn.join("trunk/deep")).unwrap().kind,
            VcsKind::Svn
        );
        assert_eq!(
            detect_vcs(&svn.join("trunk/deep")).unwrap().kind.tag(),
            "SVN 工程"
        );
        assert!(detect_vcs(&plain).is_none());

        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn gitignore_applies_without_repo_marker() {
        let dir = temp_root("toolbox-blank-line-ignore-plain");
        touch(&dir.join(".gitignore"), "*.log\nout\n");
        touch(&dir.join("a.txt"), "a\n\nb\n");
        touch(&dir.join("b.log"), "b\n\nc\n");
        touch(&dir.join("out/c.txt"), "c\n");
        touch(&dir.join("keep/d.txt"), "d\n");

        let result = scan(&[dir.to_string_lossy().to_string()]);
        assert_eq!(scan_names(&dir), vec![".gitignore", "a.txt", "keep/d.txt"]);
        assert_eq!(result.ignored, 1);
        assert_eq!(result.ignored_dirs, 1);
        assert_eq!(result.ignored_total(), 2);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn nested_gitignore_negation_wins() {
        let dir = temp_root("toolbox-blank-line-ignore-nested");
        touch(&dir.join(".gitignore"), "*.log\n");
        touch(&dir.join("sub/.gitignore"), "!keep.log\n");
        touch(&dir.join("a.log"), "a\n");
        touch(&dir.join("sub/keep.log"), "k\n");
        touch(&dir.join("sub/other.log"), "o\n");

        assert_eq!(
            scan_names(&dir),
            vec![".gitignore", "sub/.gitignore", "sub/keep.log"]
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn parent_repo_rules_apply_to_selected_subdir() {
        let repo = temp_root("toolbox-blank-line-ignore-parent");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        touch(&repo.join(".gitignore"), "*.tmp\nbuild-out/\n");
        touch(&repo.join("src/a.tmp"), "a\n");
        touch(&repo.join("src/b.txt"), "b\n");
        touch(&repo.join("build-out/c.txt"), "c\n");

        let result = scan(&[repo.join("src").to_string_lossy().to_string()]);
        assert_eq!(scan_names(&repo.join("src")), vec!["b.txt"]);
        assert_eq!(result.ignored, 1);

        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn git_info_exclude_is_honored() {
        let repo = temp_root("toolbox-blank-line-ignore-exclude");
        touch(&repo.join(".git/info/exclude"), "*.secret\nprec.txt\n");
        touch(&repo.join(".gitignore"), "!prec.txt\n");
        touch(&repo.join("a.secret"), "a\n");
        touch(&repo.join("b.txt"), "b\n");
        touch(&repo.join("prec.txt"), "p\n");

        let result = scan(&[repo.to_string_lossy().to_string()]);
        assert_eq!(scan_names(&repo), vec![".gitignore", "b.txt", "prec.txt"]);
        assert_eq!(result.ignored, 1);

        std::fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn ignore_rules_match_real_git() {
        if !git_available() {
            return;
        }

        let repo = temp_root("toolbox-blank-line-ignore-parity");
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(&repo)
                .args(args)
                .output()
                .unwrap()
        };
        assert!(run(&["init", "-q", "."]).status.success());

        touch(
            &repo.join(".gitignore"),
            "# comment\n\n/build-out/\nout\n*.log\n!keep.log\n/root-only.txt\nnested/*.txt\ndocs/**/*.tmp\nsub/secret.txt\n**/gen/**\na?.txt\n[0-9].txt\ntrail/   \nspace\\ name.txt\n!prec.txt\n",
        );
        touch(&repo.join("sub/.gitignore"), "!secret.txt\n");
        touch(
            &repo.join(".git/info/exclude"),
            "excluded-by-info.txt\nprec.txt\n",
        );

        for name in [
            "root-only.txt",
            "a.txt",
            "a1.txt",
            "1.txt",
            "keep.log",
            "app.log",
            "space name.txt",
            "space  name.txt",
            "excluded-by-info.txt",
            "prec.txt",
            "out/x.txt",
            "logs/y.txt",
            "nested/a.txt",
            "nested/deep/a.txt",
            "deep/one/a.txt",
            "deep/two/a.txt",
            "docs/a/b.tmp",
            "docs/b.txt",
            "sub/secret.txt",
            "sub/public.txt",
            "test/t.txt",
            "gen/deep/g.txt",
            "build-out/z.txt",
            "trail/z.txt",
        ] {
            touch(&repo.join(name), "line\n\nline\n");
        }

        let listed = run(&["ls-files", "-o", "--exclude-standard", "-z"]);
        assert!(listed.status.success());
        let mut expected: Vec<String> = String::from_utf8_lossy(&listed.stdout)
            .split('\0')
            .filter(|path| !path.is_empty())
            .map(str::to_string)
            .collect();
        expected.sort();

        let result = scan(&[repo.to_string_lossy().to_string()]);
        assert_eq!(scan_names(&repo), expected);
        assert!(result.ignored > 0);
        assert!(result.ignored_dirs > 0);

        std::fs::remove_dir_all(&repo).ok();
    }
}
