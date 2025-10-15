use std::fs;
use std::path::PathBuf;
use dirs::home_dir;

pub fn install_icons(staging_dir: &str, theme_name: &str, home_dir: &str) -> Result<(), String> {
    let icons_dir = format!("{}/.local/share/icons", home_dir);
    let dest_dir = format!("{}/{}", icons_dir, theme_name);
    
    fs::create_dir_all(&icons_dir).map_err(|e| format!("Failed to create icons directory: {}", e))?;
        
    if std::path::Path::new(&dest_dir).exists() {
        fs::remove_dir_all(&dest_dir).map_err(|e| format!("Failed to remove existing icons: {}", e))?;
    }
    
    copy_dir_recursive(staging_dir, &dest_dir).map_err(|e| format!("Failed to install icons: {}", e))?;
    Ok(())
}

pub fn install_cursors(staging_dir: &str, theme_name: &str, home_dir: &str) -> Result<(), String> {
    let cursors_dir = format!("{}/.local/share/icons", home_dir);
    let dest_dir = format!("{}/{}", cursors_dir, theme_name);
    
    fs::create_dir_all(&cursors_dir).map_err(|e| format!("Failed to create cursors directory: {}", e))?;
        
    if std::path::Path::new(&dest_dir).exists() {
        fs::remove_dir_all(&dest_dir).map_err(|e| format!("Failed to remove existing cursors: {}", e))?;
    }
    
    copy_dir_recursive(staging_dir, &dest_dir).map_err(|e| format!("Failed to install cursors: {}", e))?;
    Ok(())
}

pub fn install_fonts(staging_dir: &str, theme_name: &str, home_dir: &str) -> Result<(), String> {
    let fonts_dir = format!("{}/.local/share/fonts/{}", home_dir, theme_name);
    fs::create_dir_all(&fonts_dir).map_err(|e| format!("Failed to create fonts directory: {}", e))?;
        
    if let Ok(entries) = fs::read_dir(staging_dir) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ["ttf", "otf", "woff", "woff2", "eot"].contains(&ext.to_str().unwrap_or("")) {
                    let dest_path = std::path::Path::new(&fonts_dir).join(entry.file_name());
                    fs::copy(entry.path(), dest_path).map_err(|e| format!("Failed to copy font: {}", e))?;
                }
            }
        }
    }
    
    Ok(())
}

pub fn copy_dir_recursive(src: &str, dst: &str) -> Result<(), std::io::Error> {
    use std::path::Path;
    
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path.to_string_lossy(), &dst_path.to_string_lossy())?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn apply_config_file(file_data: Vec<u8>, file_name: String, dest_path: String) -> Result<String, String> {
    let path = if dest_path.starts_with("~") {
        let mut home = home_dir().ok_or("Could not determine home directory")?;
        home.push(&dest_path[2..]);
        home
    } else {
        PathBuf::from(dest_path)
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    fs::write(&path, &file_data).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(format!("Configuration file {} applied to {:?}", file_name, path))
}

#[tauri::command]
pub fn backup_config_file(src_path: String) -> Result<String, String> {
    // Expand ~ to /home
    let src_path = shellexpand::tilde(&src_path).to_string();
    let src = PathBuf::from(src_path);

    // If the source file doesn't exist, throw an error
    if !src.exists() {
        return Err("Source file does not exist".into());
    }

    // Create a copy of the source file with the .bak extension
    let backup_path = src.with_extension("bak");
    fs::copy(&src, &backup_path)
        .map_err(|e| format!("Failed to backup file: {}", e))?;
    
    Ok("Backup created at {}".to_string())
}