// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use std::fs::File;
// use std::io::Write;
// use std::path::PathBuf;
use zip::write::FileOptions;
use zip::ZipWriter;
use tauri::Emitter;

// TODO 压缩完后，打开zip文件所在文件夹即可，不用解压
// TODO 要能压缩文件，不止是文件夹
// TODO 要能拖拽进入app
#[tauri::command]
async fn compress_folder(
    source_dir: String,
    output_file: String,
    window: tauri::Window
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let file = File::create(&output_file).map_err(|e| format!("创建输出文件失败: {}", e))?;
        let mut zip = ZipWriter::new(file);

        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)  // 改用 Stored 方法
            .unix_permissions(0o755);

        let total_files = walkdir::WalkDir::new(&source_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .count();

        let mut processed = 0;

        for entry in walkdir::WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path.strip_prefix(&source_dir)
                .map_err(|e| format!("路径处理失败: {}", e))?
                .to_str()
                .ok_or("无效的UTF-8路径")?;

            if path.is_file() {
                zip.start_file(name, options)
                    .map_err(|e| format!("添加文件到ZIP失败: {}", e))?;
                let mut f = File::open(path)
                    .map_err(|e| format!("打开源文件失败: {}", e))?;
                std::io::copy(&mut f, &mut zip)
                    .map_err(|e| format!("写入ZIP文件失败: {}", e))?;
            } else if !name.is_empty() {
                zip.add_directory(name, options)
                    .map_err(|e| format!("添加目录到ZIP失败: {}", e))?;
            }
            processed += 1;
            let progress = (processed as f64 / total_files as f64) * 100.0;
            if let Err(e) = window.emit("compress_progress", progress) {
                eprintln!("发送进度失败: {}", e);
            }
        }

        // 确保正确关闭ZIP文件
        zip.finish().map_err(|e| format!("完成ZIP文件失败: {}", e))?;
        Ok(format!("成功压缩到: {}", output_file))
    })
    .await
    .map_err(|e: tokio::task::JoinError| format!("任务执行失败: {}", e))?
    .map_err(|e: String| e.to_string())
}

#[allow(dead_code)]
#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    opener::open(path).map_err(|e| format!("无法打开文件: {}", e))?;
    Ok(())
}

use tauri::Manager; // 确保导入 Manager trait

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, compress_folder, open_file])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                // 方法1：使用 webview_windows() 获取第一个窗口
                if let Some(window) = app.webview_windows().values().next() {
                    window.open_devtools();
                    window.eval("console.log('开发者工具已开启')").unwrap();
                }

                // 或者方法2：使用窗口标签名获取（需要确认你的窗口标签）
                // if let Ok(window) = app.get_window("main") {
                //     window.open_devtools();
                // }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
