use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};
use git2::{Repository, StatusOptions, StatusShow};
use std::path::Path;
use std::time::Duration;
use tauri::{Emitter, Manager};

#[cfg(target_os = "macos")]
use notify::{RecommendedWatcher, RecursiveMode};
#[cfg(target_os = "macos")]
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};

// 声明 Objective-C 函数
#[cfg(target_os = "macos")]
extern "C" {
    fn swift_show_panel(content: *const i8, title: *const i8, button_text: *const i8);
    fn swift_show_alert(title: *const i8, message: *const i8, button_text: *const i8);
    fn create_native_toolbar(window: *mut std::ffi::c_void);
    fn get_clicked_toolbar_button() -> *mut i8;
    fn set_toolbar_button_enabled(button_id: *const i8, enabled: i32);
    fn set_toolbar_button_title(button_id: *const i8, title: *const i8);
}

// SwiftUI 相关数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPanelRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowAlertRequest {
    pub title: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwiftUIResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

// SwiftUI 命令
#[tauri::command]
async fn open_swiftui_panel(request: OpenPanelRequest) -> Result<SwiftUIResponse, String> {
    println!("打开 SwiftUI 面板: {:?}", request);
    
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        
        let content = CString::new(request.content).map_err(|e| e.to_string())?;
        let title = CString::new(request.title.unwrap_or_else(|| "SwiftUI Panel".to_string())).map_err(|e| e.to_string())?;
        let button_text = CString::new(request.button_text.unwrap_or_else(|| "OK".to_string())).map_err(|e| e.to_string())?;
        
        unsafe {
            swift_show_panel(content.as_ptr(), title.as_ptr(), button_text.as_ptr());
        }
        
        Ok(SwiftUIResponse {
            status: "opened".to_string(),
            message: Some("原生 SwiftUI 面板已打开".to_string()),
            action: None,
        })
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Ok(SwiftUIResponse {
            status: "error".to_string(),
            message: Some("仅支持 macOS".to_string()),
            action: None,
        })
    }
}

#[tauri::command]
async fn show_native_alert(request: ShowAlertRequest) -> Result<SwiftUIResponse, String> {
    println!("显示原生警告: {:?}", request);
    
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        
        let title = CString::new(request.title).map_err(|e| e.to_string())?;
        let message = CString::new(request.message).map_err(|e| e.to_string())?;
        let button_text = CString::new(request.button_text.unwrap_or_else(|| "确定".to_string())).map_err(|e| e.to_string())?;
        
        unsafe {
            swift_show_alert(title.as_ptr(), message.as_ptr(), button_text.as_ptr());
        }
        
        Ok(SwiftUIResponse {
            status: "confirmed".to_string(),
            message: Some("用户已确认".to_string()),
            action: Some("alert_dismissed".to_string()),
        })
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Ok(SwiftUIResponse {
            status: "error".to_string(),
            message: Some("仅支持 macOS".to_string()),
            action: None,
        })
    }
}

// 声明搜索模块
mod search;

// Windows 平台隐藏 CMD 窗口
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffLine {
    pub line_number: usize,
    pub content: String,
    pub change_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub hunks: Vec<DiffHunk>,
    pub old_content: Vec<String>,
    pub new_content: Vec<String>,
    pub is_binary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitStatus {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub grammars: Vec<GrammarInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrammarInfo {
    pub language: String,
    pub scope_name: String,
    pub path: String,
}

// 全局文件监控器
#[cfg(target_os = "macos")]
struct FileWatcher {
    watcher: Option<notify_debouncer_mini::Debouncer<RecommendedWatcher>>,
    repo_path: Option<String>,
}

#[cfg(not(target_os = "macos"))]
struct FileWatcher {
    repo_path: Option<String>,
}

#[cfg(target_os = "macos")]
impl FileWatcher {
    fn new() -> Self {
        Self {
            watcher: None,
            repo_path: None,
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl FileWatcher {
    fn new() -> Self {
        Self {
            repo_path: None,
        }
    }
}

fn has_system_git() -> bool {
    let mut cmd = Command::new("git");
    cmd.arg("--version");
    
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    
    cmd.output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// 辅助函数：创建隐藏窗口的 Command（Windows）
fn create_hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    
    cmd
}

fn get_repo_status(repo_path: &str) -> Result<Vec<GitStatus>, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut status_opts = StatusOptions::new();
    status_opts.show(StatusShow::IndexAndWorkdir);
    status_opts.include_untracked(true);

    let statuses = repo
        .statuses(Some(&mut status_opts))
        .map_err(|e| format!("Failed to get statuses: {}", e))?;

    let mut result = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();

        let status_str = if status.is_index_new() || status.is_wt_new() {
            "Added"
        } else if status.is_index_modified() || status.is_wt_modified() {
            "Modified"
        } else if status.is_index_deleted() || status.is_wt_deleted() {
            "Deleted"
        } else if status.is_index_renamed() || status.is_wt_renamed() {
            "Renamed"
        } else {
            "Unknown"
        };

        result.push(GitStatus {
            path,
            status: status_str.to_string(),
        });
    }

    Ok(result)
}

#[tauri::command]
async fn get_working_tree_changes(repo_path: String) -> Result<Vec<GitStatus>, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut status_opts = StatusOptions::new();
    status_opts.show(StatusShow::Workdir);
    status_opts.include_untracked(true);

    let statuses = repo
        .statuses(Some(&mut status_opts))
        .map_err(|e| format!("Failed to get statuses: {}", e))?;

    let mut result = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();

        let status_str = if status.is_wt_new() {
            "Added"
        } else if status.is_wt_modified() {
            "Modified"
        } else if status.is_wt_deleted() {
            "Deleted"
        } else if status.is_wt_renamed() {
            "Renamed"
        } else {
            continue;
        };

        result.push(GitStatus {
            path,
            status: status_str.to_string(),
        });
    }

    Ok(result)
}

#[tauri::command]
async fn get_staged_changes(repo_path: String) -> Result<Vec<GitStatus>, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut status_opts = StatusOptions::new();
    status_opts.show(StatusShow::Index);

    let statuses = repo
        .statuses(Some(&mut status_opts))
        .map_err(|e| format!("Failed to get statuses: {}", e))?;

    let mut result = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();

        let status_str = if status.is_index_new() {
            "Added"
        } else if status.is_index_modified() {
            "Modified"
        } else if status.is_index_deleted() {
            "Deleted"
        } else if status.is_index_renamed() {
            "Renamed"
        } else {
            continue;
        };

        result.push(GitStatus {
            path,
            status: status_str.to_string(),
        });
    }

    Ok(result)
}

#[tauri::command]
async fn get_file_content_at_revision(repo_path: String, file_path: String, revision: String) -> Result<String, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let obj = repo
        .revparse_single(&revision)
        .map_err(|e| format!("Failed to resolve revision '{}': {}", revision, e))?;

    // 先 peel 到 commit，然后获取 commit 的 tree
    let commit = obj
        .peel_to_commit()
        .map_err(|e| format!("Failed to peel to commit: {}", e))?;

    let tree = commit
        .tree()
        .map_err(|e| format!("Failed to get commit tree: {}", e))?;

    let entry = tree
        .get_path(Path::new(&file_path))
        .map_err(|e| format!("Failed to get path '{}': {}", file_path, e))?;

    let blob = entry
        .to_object(&repo)
        .map_err(|e| format!("Failed to get object: {}", e))?
        .peel_to_blob()
        .map_err(|e| format!("Failed to peel to blob: {}", e))?;

    let content = blob.content();
    
    // 检查是否是二进制文件
    if content.iter().any(|&b| b == 0) {
        return Ok("[二进制文件]".to_string());
    }

    String::from_utf8(content.to_vec())
        .map_err(|e| format!("Failed to convert to string: {}", e))
}

#[tauri::command]
async fn read_file_content(file_path: String) -> Result<String, String> {
    let content = std::fs::read(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // 检查是否是二进制文件
    if content.iter().any(|&b| b == 0) {
        return Ok("[二进制文件]".to_string());
    }

    String::from_utf8(content)
        .map_err(|e| format!("Failed to convert to string: {}", e))
}

#[tauri::command]
async fn get_staged_file_content(repo_path: String, file_path: String) -> Result<String, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    // 获取暂存区（index）中的文件内容
    let index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;

    let index_entry = index.get_path(Path::new(&file_path), 0)
        .ok_or_else(|| format!("File not found in index: {}", file_path))?;

    let blob = repo.find_blob(index_entry.id)
        .map_err(|e| format!("Failed to find blob: {}", e))?;

    let content = blob.content();
    
    // 检查是否是二进制文件
    if content.iter().any(|&b| b == 0) {
        return Ok("[二进制文件]".to_string());
    }

    String::from_utf8(content.to_vec())
        .map_err(|e| format!("Failed to convert to string: {}", e))
}

fn parse_diff(diff_text: &str, _base_path: &str) -> Result<FileDiff, String> {
    let mut old_path = String::new();
    let mut new_path = String::new();
    let mut hunks = Vec::new();
    let mut old_content = Vec::new();
    let mut new_content = Vec::new();
    let mut is_binary = false;

    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line_num = 0;
    let mut new_line_num = 0;
    
    // 使用队列存储待处理的删除行（支持多行修改）
    let mut pending_removed: Vec<(DiffLine, usize)> = Vec::new();

    for line in diff_text.lines() {
        if line.contains("Binary files") || line.contains("GIT binary patch") {
            is_binary = true;
            continue;
        }

        // 跳过 "No newline at end of file" 提示
        if line.contains("No newline at end of file") {
            continue;
        }

        if line.starts_with("--- ") {
            old_path = line[4..].to_string();
            if old_path.starts_with("a/") {
                old_path = old_path[2..].to_string();
            }
        } else if line.starts_with("+++ ") {
            new_path = line[4..].to_string();
            if new_path.starts_with("b/") {
                new_path = new_path[2..].to_string();
            }
        } else if line.starts_with("@@") {
            // 处理所有待删除的行
            for (removed_line, _) in pending_removed.drain(..) {
                if let Some(ref mut hunk) = current_hunk {
                    hunk.lines.push(removed_line);
                }
            }
            
            if let Some(hunk) = current_hunk.take() {
                hunks.push(hunk);
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let old_range = &parts[1][1..];
                let new_range = &parts[2][1..];

                let old_parts: Vec<&str> = old_range.split(',').collect();
                let new_parts: Vec<&str> = new_range.split(',').collect();

                let old_start = old_parts[0].parse().unwrap_or(1);
                let old_lines = old_parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
                let new_start = new_parts[0].parse().unwrap_or(1);
                let new_lines = new_parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);

                old_line_num = old_start;
                new_line_num = new_start;

                current_hunk = Some(DiffHunk {
                    old_start,
                    old_lines,
                    new_start,
                    new_lines,
                    lines: Vec::new(),
                });
            }
        } else if !line.is_empty() && current_hunk.is_some() {
            let change_type = match line.chars().next() {
                Some('-') => "removed",
                Some('+') => "added",
                Some(' ') => "unchanged",
                _ => "unknown",
            };

            let content = if line.len() > 1 {
                line[1..].to_string()
            } else {
                String::new()
            };

            match change_type {
                "removed" => {
                    let diff_line = DiffLine {
                        line_number: old_line_num,
                        content: content.clone(),
                        change_type: "removed".to_string(),
                    };
                    pending_removed.push((diff_line, old_line_num));
                    old_line_num += 1;
                }
                "added" => {
                    // 检查是否有待处理的删除行
                    if let Some((removed_line, removed_num)) = pending_removed.pop() {
                        // 配对删除和添加
                        // 先添加删除行（旧内容）
                        let removed_content = removed_line.content.clone();
                        let removed_diff_line = DiffLine {
                            line_number: removed_num,
                            content: removed_content.clone(),
                            change_type: "removed".to_string(),
                        };
                        if let Some(ref mut hunk) = current_hunk {
                            hunk.lines.push(removed_diff_line);
                        }
                        old_content.push(removed_content);
                        
                        // 再添加新增行（新内容）
                        let added_content = content.clone();
                        let added_diff_line = DiffLine {
                            line_number: new_line_num,
                            content: added_content.clone(),
                            change_type: "added".to_string(),
                        };
                        if let Some(ref mut hunk) = current_hunk {
                            hunk.lines.push(added_diff_line);
                        }
                        new_content.push(added_content);
                        new_line_num += 1;
                    } else {
                        // 没有待处理的删除行，这是纯新增
                        let content_clone = content.clone();
                        let diff_line = DiffLine {
                            line_number: new_line_num,
                            content,
                            change_type: "added".to_string(),
                        };
                        if let Some(ref mut hunk) = current_hunk {
                            hunk.lines.push(diff_line);
                        }
                        new_content.push(content_clone);
                        new_line_num += 1;
                    }
                }
                _ => {
                    // 遇到未修改行，先处理所有待删除的行
                    for (removed_line, _) in pending_removed.drain(..) {
                        if let Some(ref mut hunk) = current_hunk {
                            hunk.lines.push(removed_line);
                        }
                    }
                    
                    let diff_line = DiffLine {
                        line_number: new_line_num,
                        content: content.clone(),
                        change_type: "unchanged".to_string(),
                    };
                    if let Some(ref mut hunk) = current_hunk {
                        hunk.lines.push(diff_line);
                    }
                    old_content.push(content.clone());
                    new_content.push(content);
                    old_line_num += 1;
                    new_line_num += 1;
                }
            }
        }
    }

    // 处理最后剩余的待删除行
    for (removed_line, _) in pending_removed.drain(..) {
        if let Some(ref mut hunk) = current_hunk {
            hunk.lines.push(removed_line);
        }
    }

    if let Some(hunk) = current_hunk {
        hunks.push(hunk);
    }

    Ok(FileDiff {
        old_path,
        new_path,
        hunks,
        old_content,
        new_content,
        is_binary,
    })
}

fn diff_without_git(old_content: &str, new_content: &str) -> FileDiff {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();
    
    let mut diff_lines = Vec::new();
    let max_len = old_lines.len().max(new_lines.len());
    
    for i in 0..max_len {
        let old_line = old_lines.get(i);
        let new_line = new_lines.get(i);
        
        match (old_line, new_line) {
            (Some(old), Some(new)) if old == new => {
                diff_lines.push(DiffLine {
                    line_number: i + 1,
                    content: old.to_string(),
                    change_type: "unchanged".to_string(),
                });
            }
            (Some(old), Some(_new)) => {
                // 同一行有修改，标记为 changed
                diff_lines.push(DiffLine {
                    line_number: i + 1,
                    content: old.to_string(),
                    change_type: "changed".to_string(),
                });
            }
            (Some(old), None) => {
                diff_lines.push(DiffLine {
                    line_number: i + 1,
                    content: old.to_string(),
                    change_type: "removed".to_string(),
                });
            }
            (None, Some(new)) => {
                diff_lines.push(DiffLine {
                    line_number: i + 1,
                    content: new.to_string(),
                    change_type: "added".to_string(),
                });
            }
            (None, None) => {}
        }
    }

    let old_content_vec = old_lines.iter().map(|s| s.to_string()).collect();
    let new_content_vec = new_lines.iter().map(|s| s.to_string()).collect();

    FileDiff {
        old_path: String::new(),
        new_path: String::new(),
        hunks: vec![DiffHunk {
            old_start: 1,
            old_lines: old_lines.len(),
            new_start: 1,
            new_lines: new_lines.len(),
            lines: diff_lines,
        }],
        old_content: old_content_vec,
        new_content: new_content_vec,
        is_binary: false,
    }
}

fn compare_strings_impl(old_content: &str, new_content: &str) -> Result<FileDiff, String> {
    // 如果内容完全相同，直接返回无差异结果
    if old_content == new_content {
        let lines: Vec<String> = old_content.lines().map(|s| s.to_string()).collect();
        return Ok(FileDiff {
            old_path: "old".to_string(),
            new_path: "new".to_string(),
            hunks: vec![],
            old_content: lines.clone(),
            new_content: lines,
            is_binary: false,
        });
    }

    let temp_dir = std::env::temp_dir();
    let old_file_path = temp_dir.join("git_compare_old.tmp");
    let new_file_path = temp_dir.join("git_compare_new.tmp");

    std::fs::write(&old_file_path, old_content)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    std::fs::write(&new_file_path, new_content)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    let output = if has_system_git() {
        let mut cmd = create_hidden_command("git");
        cmd.args(&[
            "diff",
            "--no-index",
            "--unified=10000",
            old_file_path.to_str().unwrap(),
            new_file_path.to_str().unwrap(),
        ]);
        cmd.output()
            .map_err(|e| format!("Failed to execute git diff: {}", e))?
    } else {
        return Ok(diff_without_git(old_content, new_content));
    };

    let _ = std::fs::remove_file(&old_file_path);
    let _ = std::fs::remove_file(&new_file_path);

    let diff_text = String::from_utf8_lossy(&output.stdout);
    parse_diff(&diff_text, "")
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_directory: bool,
    pub is_file: bool,
}

#[tauri::command]
async fn get_git_diff(repo_path: String) -> Result<Vec<FileDiff>, String> {
    let statuses = get_repo_status(&repo_path)?;
    let mut diffs = Vec::new();

    for status in statuses {
        if status.status == "Modified" || status.status == "Added" || status.status == "Deleted" {
            let old_path = format!("{}/{}", repo_path, status.path);
            let new_path = old_path.clone();

            match get_file_diff(old_path, new_path).await {
                Ok(diff) => diffs.push(diff),
                Err(e) => eprintln!("Error getting diff for {}: {}", status.path, e),
            }
        }
    }

    Ok(diffs)
}

#[tauri::command]
async fn get_file_diff(old_path: String, new_path: String) -> Result<FileDiff, String> {
    let mut cmd = create_hidden_command("git");
    cmd.args(&["diff", "--no-index", "--unified=10000", &old_path, &new_path]);
    let output = cmd.output()
        .map_err(|e| format!("Failed to execute git diff: {}", e))?;

    let diff_text = String::from_utf8_lossy(&output.stdout);
    parse_diff(&diff_text, "")
}

#[tauri::command]
async fn read_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let mut entries = Vec::new();
    
    match std::fs::read_dir(&path) {
        Ok(dir_entries) => {
            for entry in dir_entries {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_directory = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    let is_file = entry.file_type().map(|ft| ft.is_file()).unwrap_or(false);
                    
                    entries.push(DirEntry {
                        name,
                        is_directory,
                        is_file,
                    });
                }
            }
            Ok(entries)
        }
        Err(e) => Err(format!("Failed to read directory: {}", e)),
    }
}

#[tauri::command]
async fn compare_strings(old_content: String, new_content: String) -> Result<FileDiff, String> {
    compare_strings_impl(&old_content, &new_content)
}

#[tauri::command]
async fn get_all_tracked_files(repo_path: String) -> Result<String, String> {
    let mut cmd = create_hidden_command("git");
    cmd.args(&["-C", &repo_path, "ls-files"]);
    let output = cmd.output()
        .map_err(|e| format!("Failed to execute git ls-files: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

use tauri::State;

#[cfg(target_os = "macos")]
#[tauri::command]
async fn start_file_watcher(
    repo_path: String,
    window: tauri::Window,
    file_watcher: State<'_, Arc<Mutex<FileWatcher>>>,
) -> Result<(), String> {
    println!("Starting file watcher for: {}", repo_path);
    
    let mut watcher_guard = file_watcher.lock().map_err(|e| e.to_string())?;

    // 如果已经有监控器，先停止
    if watcher_guard.watcher.is_some() {
        println!("Stopping existing watcher");
        watcher_guard.watcher = None;
    }

    let window_clone = window.clone();
    let repo_path_clone = repo_path.clone();

    // 创建新的监控器
    let debouncer_result = new_debouncer(
        Duration::from_millis(300),
        move |result: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            match result {
                Ok(events) => {
                    println!("File events detected: {} events", events.len());
                    let has_changes = events.iter().any(|e| {
                        matches!(e.kind, DebouncedEventKind::Any)
                    });

                    if has_changes {
                        println!("Emitting file-changed event");
                        // 发送文件变更事件到前端
                        let emit_result = window_clone.emit("file-changed", serde_json::json!({
                            "repo_path": &repo_path_clone
                        }));
                        if let Err(e) = emit_result {
                            eprintln!("Failed to emit event: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Watch error: {:?}", e);
                }
            }
        },
    );
    
    let mut debouncer = debouncer_result.map_err(|e| format!("Failed to create debouncer: {}", e))?;

    // 监控整个仓库目录
    debouncer
        .watcher()
        .watch(Path::new(&repo_path), RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch directory: {}", e))?;

    println!("File watcher started successfully");
    watcher_guard.watcher = Some(debouncer);
    watcher_guard.repo_path = Some(repo_path);

    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
async fn start_file_watcher(
    _repo_path: String,
    _window: tauri::Window,
    file_watcher: State<'_, Arc<Mutex<FileWatcher>>>,
) -> Result<(), String> {
    println!("File watcher not supported on this platform");
    let mut watcher_guard = file_watcher.lock().map_err(|e| e.to_string())?;
    watcher_guard.repo_path = Some(_repo_path);
    Ok(())
}

#[cfg(target_os = "macos")]
#[tauri::command]
async fn stop_file_watcher(
    file_watcher: State<'_, Arc<Mutex<FileWatcher>>>,
) -> Result<(), String> {
    let mut watcher_guard = file_watcher.lock().map_err(|e| e.to_string())?;
    watcher_guard.watcher = None;
    watcher_guard.repo_path = None;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
async fn stop_file_watcher(
    file_watcher: State<'_, Arc<Mutex<FileWatcher>>>,
) -> Result<(), String> {
    let mut watcher_guard = file_watcher.lock().map_err(|e| e.to_string())?;
    watcher_guard.repo_path = None;
    Ok(())
}

// 文件系统命令 - 用于插件管理
#[tauri::command]
fn get_home_dir() -> Result<String, String> {
    dirs::home_dir()
        .ok_or_else(|| "Could not get home directory".to_string())
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
fn write_file(path: String, content: String) -> Result<(), String> {
    // 确保父目录存在
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
fn copy_dir(from: String, to: String) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    fn copy_recursive(from: &Path, to: &Path) -> Result<(), String> {
        fs::create_dir_all(to).map_err(|e| format!("Failed to create directory: {}", e))?;

        for entry in fs::read_dir(from).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let from_path = entry.path();
            let to_path = to.join(entry.file_name());

            if from_path.is_dir() {
                copy_recursive(&from_path, &to_path)?;
            } else {
                fs::copy(&from_path, &to_path).map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }
        Ok(())
    }

    copy_recursive(Path::new(&from), Path::new(&to))
}

#[tauri::command]
fn remove_dir(path: String) -> Result<(), String> {
    std::fs::remove_dir_all(&path).map_err(|e| format!("Failed to remove directory: {}", e))
}

#[tauri::command]
async fn get_git_branches(repo_path: String) -> Result<Vec<String>, String> {
    let repo = Repository::open(&repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut branches = Vec::new();

    // 获取本地分支
    let branch_names = repo.branches(None)
        .map_err(|e| format!("Failed to get branches: {}", e))?;

    for branch_result in branch_names {
        let (branch, branch_type) = branch_result
            .map_err(|e| format!("Failed to read branch: {}", e))?;

        if branch_type == git2::BranchType::Local {
            let name = branch.name()
                .map_err(|e| format!("Failed to get branch name: {}", e))?
                .unwrap_or("")
                .to_string();
            if !name.is_empty() {
                branches.push(name);
            }
        }
    }

    // 获取当前分支并放到第一个
    if let Ok(head) = repo.head() {
        if let Some(name) = head.shorthand() {
            let name = name.to_string();
            branches.retain(|b| b != &name);
            branches.insert(0, name);
        }
    }

    Ok(branches)
}

#[tauri::command]
async fn get_current_branch(repo_path: String) -> Result<String, String> {
    let repo = Repository::open(&repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;

    let branch_name = head.shorthand()
        .unwrap_or("unknown")
        .to_string();

    Ok(branch_name)
}

#[derive(serde::Serialize)]
struct CommitInfo {
    hash: String,
    short_hash: String,
    message: String,
    author: String,
    date: String,
}

#[tauri::command]
async fn get_commit_history(repo_path: String, limit: Option<usize>) -> Result<Vec<CommitInfo>, String> {
    let repo = Repository::open(&repo_path)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let mut revwalk = repo.revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    revwalk.push_head()
        .map_err(|e| format!("Failed to push HEAD: {}", e))?;

    let limit = limit.unwrap_or(50);
    let mut commits = Vec::new();

    for (i, oid_result) in revwalk.enumerate() {
        if i >= limit {
            break;
        }

        let oid = oid_result.map_err(|e| format!("Failed to get oid: {}", e))?;
        let commit = repo.find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        let hash = oid.to_string();
        let short_hash = hash.chars().take(7).collect::<String>();

        let message = commit.message()
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();

        let time = commit.time();
        let timestamp = time.seconds();
        let datetime = chrono::DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let date_str = datetime.format("%Y-%m-%d %H:%M").to_string();

        commits.push(CommitInfo {
            hash,
            short_hash,
            message,
            author: author_name,
            date: date_str,
        });
    }

    Ok(commits)
}

// 导入 VSCode 语法高亮插件
#[tauri::command]
async fn import_vscode_plugin(
    plugin_path: String,
    app_handle: tauri::AppHandle,
) -> Result<PluginInfo, String> {
    use std::fs;
    use std::path::Path;

    let plugin_dir = Path::new(&plugin_path);
    if !plugin_dir.exists() || !plugin_dir.is_dir() {
        return Err("Invalid plugin directory".to_string());
    }

    // 读取 package.json
    let package_json_path = plugin_dir.join("package.json");
    let package_content = fs::read_to_string(&package_json_path)
        .map_err(|e| format!("Failed to read package.json: {}", e))?;
    let package: serde_json::Value = serde_json::from_str(&package_content)
        .map_err(|e| format!("Failed to parse package.json: {}", e))?;

    let plugin_id = package["name"].as_str().unwrap_or("unknown").to_string();
    let plugin_name = package["displayName"].as_str()
        .or_else(|| package["name"].as_str())
        .unwrap_or("Unknown Plugin")
        .to_string();
    let plugin_description = package["description"].as_str().unwrap_or("").to_string();
    let plugin_version = package["version"].as_str().unwrap_or("1.0.0").to_string();

    // 查找语法文件
    let grammars_dir = plugin_dir.join("syntaxes");
    let mut grammars = Vec::new();

    if grammars_dir.exists() {
        for entry in fs::read_dir(&grammars_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().map(|e| e == "json" || e == "tmLanguage").unwrap_or(false) {
                let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
                let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let grammar: serde_json::Value = serde_json::from_str(&content)
                    .or_else(|_| -> Result<_, String> {
                        // 尝试解析 plist XML 格式
                        Ok(serde_json::json!({
                            "scopeName": format!("source.{}", file_name),
                            "name": file_name.clone()
                        }))
                    })?;

                let scope_name = grammar["scopeName"].as_str()
                    .or_else(|| grammar["scope_name"].as_str())
                    .unwrap_or(&format!("source.{}", file_name))
                    .to_string();
                let language = grammar["name"].as_str()
                    .unwrap_or(&file_name)
                    .to_string();

                grammars.push(GrammarInfo {
                    language,
                    scope_name,
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }

    // 复制插件到应用数据目录
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let plugins_dir = app_data_dir.join("plugins");
    let target_dir = plugins_dir.join(&plugin_id);

    fs::create_dir_all(&plugins_dir).map_err(|e| e.to_string())?;

    // 如果已存在，先删除
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).map_err(|e| e.to_string())?;
    }

    // 复制插件文件
    fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), String> {
        fs::create_dir_all(to).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let from_path = entry.path();
            let to_path = to.join(entry.file_name());
            if from_path.is_dir() {
                copy_dir_recursive(&from_path, &to_path)?;
            } else {
                fs::copy(&from_path, &to_path).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    copy_dir_recursive(plugin_dir, &target_dir)?;

    Ok(PluginInfo {
        id: plugin_id,
        name: plugin_name,
        description: plugin_description,
        version: plugin_version,
        grammars,
    })
}

// 获取已安装的插件列表
#[tauri::command]
fn get_installed_plugins(app_handle: tauri::AppHandle) -> Result<Vec<PluginInfo>, String> {
    use std::fs;

    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let plugins_dir = app_data_dir.join("plugins");

    let mut plugins = Vec::new();

    if plugins_dir.exists() {
        for entry in fs::read_dir(&plugins_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let plugin_dir = entry.path();
            if plugin_dir.is_dir() {
                let package_json_path = plugin_dir.join("package.json");
                if let Ok(content) = fs::read_to_string(&package_json_path) {
                    if let Ok(package) = serde_json::from_str::<serde_json::Value>(&content) {
                        let plugin_id = package["name"].as_str().unwrap_or("unknown").to_string();
                        let plugin_name = package["displayName"].as_str()
                            .or_else(|| package["name"].as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        let plugin_description = package["description"].as_str().unwrap_or("").to_string();
                        let plugin_version = package["version"].as_str().unwrap_or("1.0.0").to_string();

                        // 查找语法文件
                        let grammars_dir = plugin_dir.join("syntaxes");
                        let mut grammars = Vec::new();

                        if grammars_dir.exists() {
                            if let Ok(entries) = fs::read_dir(&grammars_dir) {
                                for entry in entries.flatten() {
                                    let path = entry.path();
                                    if path.extension().map(|e| e == "json" || e == "tmLanguage").unwrap_or(false) {
                                        let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
                                        grammars.push(GrammarInfo {
                                            language: file_name.clone(),
                                            scope_name: format!("source.{}", file_name),
                                            path: path.to_string_lossy().to_string(),
                                        });
                                    }
                                }
                            }
                        }

                        plugins.push(PluginInfo {
                            id: plugin_id,
                            name: plugin_name,
                            description: plugin_description,
                            version: plugin_version,
                            grammars,
                        });
                    }
                }
            }
        }
    }

    Ok(plugins)
}

// 删除插件
#[tauri::command]
fn remove_plugin(plugin_id: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    use std::fs;

    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let plugin_dir = app_data_dir.join("plugins").join(&plugin_id);

    if plugin_dir.exists() {
        fs::remove_dir_all(&plugin_dir).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// 打开系统设置（完全磁盘访问权限页面）
#[tauri::command]
fn open_system_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
            .spawn()
            .map_err(|e| format!("Failed to open system settings: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows/Linux 暂不支持此功能
        Err("This feature is only available on macOS".to_string())
    }
}

// 在终端中打开指定路径
#[tauri::command]
fn open_in_terminal(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg("cmd")
            .arg("/K")
            .arg("cd")
            .arg("/d")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // 尝试常见的终端模拟器
        let terminals = ["gnome-terminal", "konsole", "xterm", "terminator"];
        for terminal in &terminals {
            if Command::new("which").arg(terminal).output().map(|o| o.status.success()).unwrap_or(false) {
                Command::new(terminal)
                    .arg("--working-directory")
                    .arg(&path)
                    .spawn()
                    .map_err(|e| format!("Failed to open terminal: {}", e))?;
                return Ok(());
            }
        }
        Err("No supported terminal emulator found".to_string())
    }
}

// 在资源管理器中打开指定路径
#[tauri::command]
fn open_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open Finder: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open Explorer: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextMenuItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub is_separator: bool,
    pub is_disabled: bool,
}

#[tauri::command]
async fn show_context_menu(
    window: tauri::Window,
    items: Vec<ContextMenuItem>,
    _x: f64,
    _y: f64,
) -> Result<(), String> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, ContextMenu};

    let menu = Menu::new(&window).map_err(|e| e.to_string())?;

    for item in items {
        if item.is_separator {
            let separator = PredefinedMenuItem::separator(&window).map_err(|e| e.to_string())?;
            menu.append(&separator).map_err(|e| e.to_string())?;
        } else {
            let menu_item = MenuItem::with_id(
                &window,
                item.id.clone(),
                &item.label,
                true,
                None::<&str>,
            )
            .map_err(|e| e.to_string())?;
            menu.append(&menu_item).map_err(|e| e.to_string())?;
        }
    }

    // 在指定位置显示菜单
    menu.popup(window)
        .map_err(|e: tauri::Error| e.to_string())?;

    Ok(())
}

// 原生工具栏命令
#[tauri::command]
async fn create_native_toolbar_command(window: tauri::Window) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::c_void;
        
        // 在主线程执行
        window.clone().run_on_main_thread(move || {
            unsafe {
                // 获取 NSWindow 指针
                let ns_window = window.ns_window().unwrap() as *mut c_void;
                create_native_toolbar(ns_window);
            }
        }).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Err("仅支持 macOS".to_string())
    }
}

#[tauri::command]
async fn set_toolbar_button_enabled_command(button_id: String, enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        let id_cstring = CString::new(button_id).map_err(|e| e.to_string())?;
        unsafe {
            set_toolbar_button_enabled(id_cstring.as_ptr(), if enabled { 1 } else { 0 });
        }
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Err("仅支持 macOS".to_string())
    }
}

#[tauri::command]
async fn set_toolbar_button_title_command(button_id: String, title: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        let id_cstring = CString::new(button_id).map_err(|e| e.to_string())?;
        let title_cstring = CString::new(title).map_err(|e| e.to_string())?;
        unsafe {
            set_toolbar_button_title(id_cstring.as_ptr(), title_cstring.as_ptr());
        }
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Err("仅支持 macOS".to_string())
    }
}

// 轮询获取点击的工具栏按钮
#[tauri::command]
async fn poll_toolbar_button_click() -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        unsafe {
            let ptr = get_clicked_toolbar_button();
            if ptr.is_null() {
                Ok(None)
            } else {
                use std::ffi::CStr;
                let c_str = CStr::from_ptr(ptr);
                let result = c_str.to_str().map_err(|e| e.to_string())?.to_string();
                libc::free(ptr as *mut libc::c_void);
                Ok(Some(result))
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Ok(None)
    }
}

pub fn run() {
    let file_watcher = Arc::new(Mutex::new(FileWatcher::new()));

    let mut builder = tauri::Builder::default()
        .manage(file_watcher)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init());
    
    // 仅在 macOS 上加载 liquid-glass 插件
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_liquid_glass::init());
    }
    
    builder
        .setup(|app| {
            // 存储 AppHandle 以便在回调中使用
            let _app_handle = app.handle().clone();
            
            // 设置窗口效果
            #[cfg(target_os = "macos")]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    // 使用 tauri-plugin-liquid-class 设置液态玻璃效果
                    // macOS 14+ 支持液态玻璃效果
                    // 使用默认标题栏样式
                    let _ = window.set_title_bar_style(tauri::TitleBarStyle::default());
                    
                    // NSToolbar 将由前端通过插件创建
                    println!("窗口已创建，等待前端创建 NSToolbar...");
                }
            }

            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    // Windows 11 支持云母/亚克力效果
                    let _ = window.set_decorations(true);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_swiftui_panel,
            show_native_alert,
            create_native_toolbar_command,
            set_toolbar_button_enabled_command,
            set_toolbar_button_title_command,
            poll_toolbar_button_click,
            get_git_diff,
            get_file_diff,
            get_working_tree_changes,
            get_staged_changes,
            get_file_content_at_revision,
            read_file_content,
            get_staged_file_content,
            compare_strings,
            get_all_tracked_files,
            read_directory,
            start_file_watcher,
            stop_file_watcher,
            get_home_dir,
            read_file,
            write_file,
            copy_dir,
            remove_dir,
            import_vscode_plugin,
            get_installed_plugins,
            remove_plugin,
            open_system_settings,
            open_in_terminal,
            open_in_explorer,
            search::search_in_file,
            search::search_in_directory,
            show_context_menu,
            get_git_branches,
            get_current_branch,
            get_commit_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
