#[tauri::command]
pub fn select_folder() -> Result<String, String> {
    let output = Command::new("zenity")
        .arg("--file-selection")
        .arg("--directory")
        .arg("--title=Select Theme Folder")
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let path = String::from_utf8_lossy(&result.stdout).trim().to_string();
                if !path.is_empty() {
                    Ok(path)
                } else {
                    Err("No folder selected".to_string())
                }
            } else {
                Err("Failed to open folder dialog".to_string())
            }
        }
        Err(_) => {
            // Fallback to using nautilus or other file manager
            let output = Command::new("sh")
                .arg("-c")
                .arg("nautilus --select ~/")
                .output();
            
            match output {
                Ok(_) => Err("Please manually drag and drop a folder".to_string()),
                Err(_) => Err("No folder dialog available. Please use drag and drop.".to_string())
            }
        }
    }
}

#[tauri::command]
pub fn select_file(title: String, filters: Vec<serde_json::Value>) -> Result<String, String> {
    let output = Command::new("zenity")
        .arg("--file-selection")
        .arg(&format!("--title={}", title))
        .arg("--file-filter=Reskin Files (*.reskin) | *.reskin")
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let path = String::from_utf8_lossy(&result.stdout).trim().to_string();
                if !path.is_empty() {
                    Ok(path)
                } else {
                    Err("No file selected".to_string())
                }
            } else {
                Err("Failed to open file dialog".to_string())
            }
        }
        Err(_) => {
            Err("No file dialog available. Please use drag and drop.".to_string())
        }
    }
}

#[tauri::command]
fn ensure_reskin_folder() -> Result<(), String> {
	use std::fs;
	use std::env::temp_dir;
	let mut path = temp_dir();
	path.push("reskin");
	fs::create_dir_all(&path).map_err(|e| e.to_string())?;
	Ok(())
}

#[tauri::command]
fn theme_file_exists(theme_name: String) -> bool {
	use std::fs;
	use std::env::temp_dir;
	let mut path = temp_dir();
	path.push("reskin");
	path.push(format!("{}.reskin", theme_name));
	fs::metadata(&path).is_ok()
}