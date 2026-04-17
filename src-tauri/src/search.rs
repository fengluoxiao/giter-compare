use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchMatch {
    pub line_number: usize,
    pub column: usize,
    pub line_content: String,
    pub matched_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub matches: Vec<SearchMatch>,
    pub match_count: usize,
}

/// 将通配符模式转换为正则表达式
fn wildcard_to_regex(pattern: &str) -> Result<regex::Regex, String> {
    // 转义正则特殊字符，但保留 *
    let mut escaped = String::new();
    for c in pattern.chars() {
        match c {
            '*' => escaped.push_str(".*"),
            '?' => escaped.push('.'),
            '.' | '+' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    
    regex::Regex::new(&format!("(?i){}", escaped))
        .map_err(|e| format!("Invalid regex pattern: {}", e))
}

/// 在单行文本中搜索匹配项
fn search_in_line(line: &str, regex: &regex::Regex) -> Vec<(usize, String)> {
    let mut matches = Vec::new();
    
    for mat in regex.find_iter(line) {
        matches.push((mat.start(), mat.as_str().to_string()));
    }
    
    matches
}

/// 在文件内容中搜索
fn search_in_content(content: &str, regex: &regex::Regex) -> Vec<SearchMatch> {
    let mut matches = Vec::new();
    
    for (line_idx, line) in content.lines().enumerate() {
        let line_matches = search_in_line(line, regex);
        
        for (col, matched_text) in line_matches {
            matches.push(SearchMatch {
                line_number: line_idx + 1, // 从 1 开始计数
                column: col + 1, // 从 1 开始计数
                line_content: line.to_string(),
                matched_text,
            });
        }
    }
    
    matches
}

/// 在文件中搜索
#[tauri::command]
pub async fn search_in_file(
    file_path: String,
    pattern: String,
) -> Result<SearchResult, String> {
    // 读取文件内容
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // 转换通配符为正则
    let regex = wildcard_to_regex(&pattern)?;
    
    // 搜索匹配项
    let matches = search_in_content(&content, &regex);
    let match_count = matches.len();
    
    Ok(SearchResult {
        file_path,
        matches,
        match_count,
    })
}

/// 在目录中递归搜索
#[tauri::command]
pub async fn search_in_directory(
    dir_path: String,
    pattern: String,
    file_pattern: Option<String>,
) -> Result<Vec<SearchResult>, String> {
    let mut results = Vec::new();
    let regex = wildcard_to_regex(&pattern)?;
    
    // 如果有文件模式，转换为正则
    let file_regex = file_pattern
        .map(|p| wildcard_to_regex(&p))
        .transpose()?;
    
    search_recursive(
        Path::new(&dir_path),
        &regex,
        file_regex.as_ref(),
        &mut results,
    )?;
    
    // 按匹配数排序，匹配数多的在前
    results.sort_by(|a, b| b.match_count.cmp(&a.match_count));
    
    Ok(results)
}

/// 递归搜索目录
fn search_recursive(
    path: &Path,
    regex: &regex::Regex,
    file_regex: Option<&regex::Regex>,
    results: &mut Vec<SearchResult>,
) -> Result<(), String> {
    // 检查文件/目录是否存在
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    
    // 如果是文件，直接搜索
    if path.is_file() {
        let path_str = path.to_string_lossy().to_string();
        
        // 检查是否匹配文件模式
        if let Some(file_re) = file_regex {
            if !file_re.is_match(&path_str) {
                return Ok(());
            }
        }
        
        // 跳过二进制文件和某些特殊文件
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if matches!(ext_str.as_ref(), 
                "exe" | "dll" | "so" | "dylib" | "bin" | 
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "ico" |
                "pdf" | "zip" | "tar" | "gz" | "rar" | "7z") {
                return Ok(());
            }
        }
        
        // 读取并搜索文件
        if let Ok(content) = fs::read_to_string(path) {
            let matches = search_in_content(&content, regex);
            
            if !matches.is_empty() {
                results.push(SearchResult {
                    file_path: path_str,
                    matches,
                    match_count: 0, // 会在后面重新计算
                });
            }
        }
        
        return Ok(());
    }
    
    // 如果是目录，递归遍历
    if path.is_dir() {
        // 跳过某些目录
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy().to_lowercase();
            if matches!(name_str.as_ref(), 
                "node_modules" | ".git" | "target" | "build" | 
                "dist" | "vendor" | ".svn" | "coverage") {
                return Ok(());
            }
        }
        
        let entries = fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
        
        for entry in entries.flatten() {
            let entry_path = entry.path();
            search_recursive(&entry_path, regex, file_regex, results)?;
        }
    }
    
    // 更新每个结果的 match_count
    for result in results.iter_mut() {
        result.match_count = result.matches.len();
    }
    
    Ok(())
}
