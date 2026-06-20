use std::path::Path;

/// Helper function to determine if a file is likely a text file
pub fn is_text_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let text_extensions = ["txt", "py", "js", "ts", "json", "yaml", "yml", "md", "html", "css", "rs", "go", "java", "cpp", "c"];
    
    text_extensions.contains(&ext)
}