use std::path::PathBuf;

pub struct File;

impl File {
    pub fn name(&self) -> String {
        let prefix = FILE_PREFIX[fastrand::usize(0..FILE_PREFIX_LEN)];
        let num = fastrand::usize(100000..999999);
        let ext = self.file_extension();
        format!("{prefix}_{num}.{ext}")
    }

    fn path_info(&self, platform: &str) -> String {
        PathBuf::new()
            .join(platform)
            .join(FOLDER[fastrand::usize(0..FOLDER_LEN)])
            .join(self.name())
            .to_string_lossy()
            .to_string()
    }

    pub fn path(&self) -> String {
        self.path_info(FILE_PATH[fastrand::usize(0..3)])
    }

    pub fn file_extension(&self) -> String {
        FILE_EXT[fastrand::usize(0..FILE_EXT_LEN)].into()
    }
}

static FILE_PATH: [&str; 3] = [
    "/home/Administrator",
    "/Users/Administrator",
    "C:\\Users\\Administrator",
];
static FILE_PATH_LEN: usize = FILE_PATH.len();

static FOLDER: [&str; 5] = ["Documents", "Pictures", "Downloads", "Music", "Videos"];
static FOLDER_LEN: usize = FOLDER.len();

static FILE_PREFIX: [&str; 6] = ["data", "doc", "zip", "page", "img", "text"];
static FILE_PREFIX_LEN: usize = FILE_PREFIX.len();

static FILE_EXT: [&str; 77] = [
    "txt", "doc", "docx", "pdf", "rtf", "odt", "tex", "md", "xls", "xlsx", "csv", "json", "xml",
    "sql", "db", "html", "htm", "css", "js", "php", "asp", "jsp", "jpg", "jpeg", "png", "gif",
    "bmp", "svg", "ico", "webp", "mp3", "wav", "flac", "aac", "ogg", "wma", "midi", "mp4", "avi",
    "mov", "mkv", "flv", "wmv", "webm", "exe", "msi", "app", "dmg", "sh", "bat", "jar", "zip",
    "rar", "7z", "tar", "gz", "iso", "c", "cpp", "py", "java", "cs", "go", "swift", "rs", "dll",
    "sys", "ini", "cfg", "log", "tmp", "torrent", "epub", "psd", "ai", "ppt", "pptx",
];
static FILE_EXT_LEN: usize = FILE_EXT.len();
