use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

static ELECTRON_CACHE: std::sync::LazyLock<Mutex<HashMap<String, bool>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn is_electron_app(app_name: &str) -> bool {
    if let Ok(cache) = ELECTRON_CACHE.lock() {
        if let Some(&result) = cache.get(app_name) {
            return result;
        }
    }

    let result = check_electron(app_name);

    if let Ok(mut cache) = ELECTRON_CACHE.lock() {
        cache.insert(app_name.to_string(), result);
    }

    result
}

#[cfg(target_os = "macos")]
fn check_electron(app_name: &str) -> bool {
    let framework_paths = [
        format!("/Applications/{app_name}.app/Contents/Frameworks/Electron Framework.framework"),
        format!(
            "{}/Applications/{app_name}.app/Contents/Frameworks/Electron Framework.framework",
            std::env::var("HOME").unwrap_or_default()
        ),
    ];

    for path in &framework_paths {
        if std::path::Path::new(path).exists() {
            tracing::debug!("{app_name} is Electron (bundle check)");
            return true;
        }
    }

    if let Ok(output) = std::process::Command::new("ps")
        .args(["-eo", "comm,args"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lower = app_name.to_lowercase();
        for line in stdout.lines() {
            let ll = line.to_lowercase();
            if ll.contains(&lower)
                && (ll.contains("electron")
                    || ll.contains("--type=renderer")
                    || ll.contains("chromium"))
            {
                tracing::debug!("{app_name} is Electron (process check)");
                return true;
            }
        }
    }

    false
}

#[cfg(not(target_os = "macos"))]
fn check_electron(_app_name: &str) -> bool {
    false
}

pub fn find_cdp_port(app_name: &str) -> Option<u16> {
    if let Some(port) = load_port_cache(app_name) {
        if is_port_open(port) {
            return Some(port);
        }
        clear_port_cache(app_name);
    }

    let output = std::process::Command::new("ps")
        .args(["-eo", "args"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lower = app_name.to_lowercase();

    for line in stdout.lines() {
        if !line.to_lowercase().contains(&lower) {
            continue;
        }
        for part in line.split_whitespace() {
            if let Some(port_str) = part.strip_prefix("--remote-debugging-port=") {
                if let Ok(port) = port_str.parse::<u16>() {
                    if port > 0 {
                        tracing::debug!("found CDP port {port} for {app_name}");
                        save_port_cache(app_name, port);
                        return Some(port);
                    }
                }
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
pub fn relaunch_with_cdp(app_name: &str) -> Option<u16> {
    let binary = find_app_binary(app_name)?;
    let port = find_free_port()?;

    tracing::debug!("relaunching {app_name} with CDP on port {port}");

    let _ = std::process::Command::new("osascript")
        .args(["-e", &format!("tell application \"{app_name}\" to quit")])
        .output();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let result = std::process::Command::new(&binary)
        .arg(format!("--remote-debugging-port={port}"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    if result.is_err() {
        tracing::debug!("failed to relaunch {app_name}: {:?}", result.err());
        return None;
    }

    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if is_port_open(port) {
            tracing::debug!("{app_name} CDP ready on port {port}");
            save_port_cache(app_name, port);
            return Some(port);
        }
    }

    tracing::debug!("{app_name} CDP port {port} never became available");
    None
}

#[cfg(not(target_os = "macos"))]
pub fn relaunch_with_cdp(_app_name: &str) -> Option<u16> {
    None
}

#[cfg(target_os = "macos")]
fn find_app_binary(app_name: &str) -> Option<String> {
    let paths = [
        format!("/Applications/{app_name}.app/Contents/MacOS/{app_name}"),
        format!(
            "{}/Applications/{app_name}.app/Contents/MacOS/{app_name}",
            std::env::var("HOME").unwrap_or_default()
        ),
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            return Some(path.clone());
        }
    }

    let bundle = format!("/Applications/{app_name}.app/Contents/MacOS");
    if let Ok(entries) = std::fs::read_dir(&bundle) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Ok(meta) = p.metadata() {
                    if !meta.permissions().readonly() && meta.len() > 1000 {
                        return Some(p.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}

fn find_free_port() -> Option<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").ok()?;
    let port = listener.local_addr().ok()?.port();
    drop(listener);
    Some(port)
}

fn is_port_open(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        std::time::Duration::from_millis(200),
    )
    .is_ok()
}

fn port_cache_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(format!("{home}/.agent-click/cdp-ports.json"))
}

fn load_port_cache(app_name: &str) -> Option<u16> {
    let path = port_cache_path();
    let contents = std::fs::read_to_string(&path).ok()?;
    let cache: serde_json::Value = serde_json::from_str(&contents).ok()?;
    cache.get(app_name)?.as_u64().map(|p| p as u16)
}

fn save_port_cache(app_name: &str, port: u16) {
    let path = port_cache_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut cache: serde_json::Map<String, serde_json::Value> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    cache.insert(app_name.to_string(), serde_json::Value::Number(port.into()));

    let _ = std::fs::write(
        &path,
        serde_json::to_string_pretty(&cache).unwrap_or_default(),
    );
}

fn clear_port_cache(app_name: &str) {
    let path = port_cache_path();
    if let Ok(contents) = std::fs::read_to_string(&path) {
        if let Ok(mut cache) =
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&contents)
        {
            cache.remove(app_name);
            let _ = std::fs::write(
                &path,
                serde_json::to_string_pretty(&cache).unwrap_or_default(),
            );
        }
    }
}
