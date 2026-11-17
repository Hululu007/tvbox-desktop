// 一键外部播放器：把直链交给本机 PotPlayer / VLC / mpv / IINA 等播放
// 两种方式：1) 设置页配置命令模板（{url} 占位，支持 --fullscreen 等参数）
//           2) 未配置时按常见安装位置自动探测
use crate::error::{AppError, AppResult};
use std::path::PathBuf;

/// 自动探测候选：返回 (显示名, 程序路径)
pub fn auto_candidates() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut check = |label: &str, path: PathBuf| {
        if path.is_file() {
            out.push((label.to_string(), path.to_string_lossy().to_string()));
        }
    };

    if cfg!(windows) {
        let pf = std::env::var("ProgramFiles").unwrap_or_default();
        let pf86 = std::env::var("ProgramFiles(x86)").unwrap_or_default();
        let up = std::env::var("USERPROFILE").unwrap_or_default();
        // PotPlayer（国内最常用）
        for base in [&pf, &pf86] {
            check("PotPlayer", PathBuf::from(base).join("DAUM/PotPlayer/PotPlayerMini64.exe"));
            check("PotPlayer", PathBuf::from(base).join("DAUM/PotPlayer64/PotPlayerMini64.exe"));
            check("PotPlayer", PathBuf::from(base).join("DAUM/PotPlayer/PotPlayerMini.exe"));
        }
        // VLC
        for base in [&pf, &pf86] {
            check("VLC", PathBuf::from(base).join("VideoLAN/VLC/vlc.exe"));
        }
        // mpv（含 scoop 安装）
        check("mpv", PathBuf::from(&pf).join("mpv/mpv.exe"));
        check(
            "mpv",
            PathBuf::from(&up).join("scoop/apps/mpv/current/mpv.exe"),
        );
    } else if cfg!(target_os = "macos") {
        check("IINA", PathBuf::from("/Applications/IINA.app/Contents/MacOS/iina"));
        check("mpv", PathBuf::from("/Applications/mpv.app/Contents/MacOS/mpv"));
        check("VLC", PathBuf::from("/Applications/VLC.app/Contents/MacOS/VLC"));
    } else {
        // Linux：常见 PATH 位置
        for (label, name) in [
            ("mpv", "mpv"),
            ("VLC", "vlc"),
            ("Celluloid", "celluloid"),
            ("SMPlayer", "smplayer"),
        ] {
            let p = PathBuf::from("/usr/bin").join(name);
            if p.is_file() {
                out.push((label.to_string(), p.to_string_lossy().to_string()));
            }
        }
    }
    out
}

/// 展示用：探测到的播放器列表（设置页「检测本机播放器」按钮）
pub fn detect_players() -> Vec<String> {
    auto_candidates()
        .into_iter()
        .map(|(label, path)| format!("{label}（{path}）"))
        .collect()
}

/// 从命令模板构造 (程序, 参数列表)
/// 模板示例：C:\Tools\mpv.exe --fullscreen {url}
/// - 含 {url} 的 token 替换为播放地址
/// - 不含 {url} 时地址追加在末尾
fn parse_template(tpl: &str, url: &str) -> AppResult<(String, Vec<String>)> {
    let tokens: Vec<&str> = tpl.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(AppError::msg("外部播放器命令模板为空"));
    }
    let program = tokens[0].to_string();
    let mut args: Vec<String> = Vec::new();
    let mut has_url = false;
    for t in &tokens[1..] {
        if t.contains("{url}") {
            has_url = true;
            args.push(t.replace("{url}", url));
        } else {
            args.push((*t).to_string());
        }
    }
    if !has_url {
        args.push(url.to_string());
    }
    Ok((program, args))
}

/// 用外部播放器打开直链。返回实际使用的播放器名。
/// template：设置页的命令模板；空则自动探测（程序路径 + 直链参数）。
pub fn open_with_player(url: &str, template: Option<&str>) -> AppResult<String> {
    if url.trim().is_empty() {
        return Err(AppError::msg("播放地址为空"));
    }

    // 1) 命令模板优先
    if let Some(tpl) = template.map(str::trim).filter(|s| !s.is_empty()) {
        let (program, args) = parse_template(tpl, url)?;
        if !PathBuf::from(&program).is_file() {
            return Err(AppError::msg(format!(
                "外部播放器程序不存在：{program}（可在设置页修改）"
            )));
        }
        return spawn_player(&program, &args).map(|_| "自定义播放器".to_string());
    }

    // 2) 自动探测
    for (label, program) in auto_candidates() {
        let spawn = {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x0800_0000;
                std::process::Command::new(&program)
                    .arg(url)
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn()
            }
            #[cfg(not(target_os = "windows"))]
            {
                std::process::Command::new(&program).arg(url).spawn()
            }
        };
        if spawn.is_ok() {
            return Ok(label);
        }
    }
    Err(AppError::msg(
        "未找到可用的外部播放器（支持 PotPlayer / VLC / mpv），可在设置页配置「外部播放器命令」",
    ))
}

fn spawn_player(program: &str, args: &[String]) -> AppResult<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new(program)
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| AppError::msg(format!("启动外部播放器失败: {e}")))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new(program)
            .args(args)
            .spawn()
            .map_err(|e| AppError::msg(format!("启动外部播放器失败: {e}")))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template_with_placeholder() {
        let (p, a) = parse_template(
            r#"C:\Tools\mpv.exe --fullscreen {url} --no-border"#,
            "https://a.com/1.m3u8",
        )
        .unwrap();
        assert_eq!(p, r#"C:\Tools\mpv.exe"#);
        assert_eq!(a, vec!["--fullscreen", "https://a.com/1.m3u8", "--no-border"]);
    }

    #[test]
    fn test_parse_template_without_placeholder() {
        let (p, a) = parse_template("C:\\PotPlayer\\PotPlayerMini64.exe", "https://a.com/1.mp4").unwrap();
        assert_eq!(p, "C:\\PotPlayer\\PotPlayerMini64.exe");
        assert_eq!(a, vec!["https://a.com/1.mp4"]);
    }

    #[test]
    fn test_empty_template_and_url() {
        assert!(parse_template("   ", "https://a.com/1.mp4").is_err());
        assert!(open_with_player("", None).is_err());
        assert!(open_with_player("   ", None).is_err());
    }

    #[test]
    fn test_missing_template_player() {
        let r = open_with_player("https://a.com/1.m3u8", Some("Z:/no/such/player.exe {url}"));
        assert!(r.is_err());
    }
}
