use std::collections::HashSet;
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 扫描 PATH 环境变量下的所有可执行程序名称
pub fn get_path_executables() -> Vec<String> {
    let mut executables = HashSet::new();

    // 1. 获取系统 PATH 环境变量（自动处理 Linux/macOS 的 ':' 或 Windows 的 ';' 分隔符）
    if let Some(path_var) = env::var_os("PATH") {
        for path in env::split_paths(&path_var) {
            // 2. 尝试读取目录中的文件
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    // 3. 判断是否为文件以及是否具有可执行权限
                    if path.is_file() && is_executable(&path) {
                        if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                            executables.insert(file_name.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut list: Vec<String> = executables.into_iter().collect();
    list.sort(); // 排序以便 Tab 补全提示更规范
    list
}

/// 检查文件是否具有执行权限
fn is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        // 在 Unix/Linux/macOS 下判断文件 Mode 是否包含可执行位 (0o111)
        if let Ok(metadata) = path.metadata() {
            return metadata.permissions().mode() & 0o111 != 0;
        }
        false
    }

    #[cfg(windows)]
    {
        // 在 Windows 下检查后缀名是否为 .exe, .bat, .cmd 等
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let ext = ext.to_lowercase();
            return matches!(ext.as_str(), "exe" | "bat" | "cmd" | "com" | "ps1");
        }
        false
    }
}
