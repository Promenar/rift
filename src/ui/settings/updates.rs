// 简体中文界面修改：本文件的显示文案或相关界面测试已本地化。
//! A user-initiated release check. No timers or background checks.
pub enum Release {
    Current,
    Available(String),
}

pub type Result = std::result::Result<Release, String>;
pub type Completion = Box<dyn FnOnce(Result)>;

/// Checks off the main thread and passes the result to `done` on the checking thread.
pub fn start(done: impl FnOnce(Result) + Send + 'static) {
    std::thread::spawn(move || done(latest()));
}

fn latest() -> Result {
    let response = std::process::Command::new("/usr/bin/curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            "10",
            "--header",
            "Accept: application/vnd.github+json",
            "--user-agent",
            "Rift-update-check",
            "https://api.github.com/repos/acsandmann/rift/releases/latest",
        ])
        .output()
        .map_err(|_| {
            "无法连接。请检查网络连接后重试。".to_string()
        })?;
    if !response.status.success() {
        return Err("无法检查更新。请检查网络连接后重试。".into());
    }
    let release: serde_json::Value = serde_json::from_slice(&response.stdout)
        .map_err(|_| "版本服务返回了无法识别的响应。".to_string())?;
    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| "版本服务未返回版本号。".to_string())?;
    let latest = tag.trim_start_matches('v');
    let running = env!("CARGO_PKG_VERSION");
    if latest == running {
        Ok(Release::Current)
    } else {
        Ok(Release::Available(latest.to_owned()))
    }
}
