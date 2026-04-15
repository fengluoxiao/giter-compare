use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};
use git2::{Repository, StatusOptions, StatusShow};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::path::Path;
use std::time::Duration;
use tauri::Emitter;

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

// 全局文件监控器
struct FileWatcher {
    watcher: Option<notify_debouncer_mini::Debouncer<RecommendedWatcher>>,
    repo_path: Option<String>,
}

impl FileWatcher {
    fn new() -> Self {
        Self {
            watcher: None,
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
        .map_err(|e| format!("Failed to resolve revision: {}", e))?;

    let tree = obj
        .peel_to_tree()
        .map_err(|e| format!("Failed to peel to tree: {}", e))?;

    let entry = tree
        .get_path(Path::new(&file_path))
        .map_err(|e| format!("Failed to get path: {}", e))?;

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

#[tauri::command]
async fn stop_file_watcher(
    file_watcher: State<'_, Arc<Mutex<FileWatcher>>>,
) -> Result<(), String> {
    let mut watcher_guard = file_watcher.lock().map_err(|e| e.to_string())?;
    watcher_guard.watcher = None;
    watcher_guard.repo_path = None;
    Ok(())
}

pub fn run() {
    let file_watcher = Arc::new(Mutex::new(FileWatcher::new()));
    
    tauri::Builder::default()
        .manage(file_watcher)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
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
            stop_file_watcher
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
