use std::{env, fs, process::Command};

fn might_be_unix_tz(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|x| x.is_ascii_alphanumeric() || ['-', '+', '/', '_'].contains(&x))
}

fn zone_from_env<F: FnMut(&str) -> bool>(mut is_valid: F) -> Option<String> {
    let tz = env::var("TZ").ok()?;
    if is_valid(&tz) {
        Some(tz)
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
fn get_local_zone_unix<F>(mut validate: F) -> Option<String>
where
    F: (FnMut(&str) -> bool),
{
    for path in &["/etc/timezone", "/var/db/zoneinfo"] {
        if let Ok(tz) = fs::read_to_string(path) {
            let tz = tz.trim();
            if validate(tz) {
                return Some(tz.into());
            }
        }
    }

    None
}

pub fn get_local_zone<F: FnMut(&str) -> bool>(mut is_valid: F) -> Option<String> {
    let mut validate = move |name: &str| might_be_unix_tz(name) && is_valid(name);

    if let Some(tz) = zone_from_env(&mut validate) {
        return Some(tz);
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(tz) = get_local_zone_unix(&mut validate) {
            return Some(tz);
        }
    }

    if let Ok(link) = fs::read_link("/etc/localtime")
        .and_then(|x| x.canonicalize())
        .map(|x| x.display().to_string())
    {
        if let Some((_, tz)) = link.split_once("/zoneinfo/") {
            if validate(tz) {
                return Some(tz.into());
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some((_, tz)) = link.split_once("/zoneinfo.default/") {
                if validate(tz) {
                    return Some(tz.into());
                }
            }
        }
    }

    // Running $data "+%Z" returns the timezone not the exact local
    if let Ok(output) = Command::new("sh").arg("-c").arg("date \"+%Z\"").output() {
        let tz = String::from_utf8(output.stdout).unwrap_or("".to_string());
        let tz = tz.trim();
        if !tz.is_empty() && validate(tz) {
            return Some(tz.to_string());
        }
    }

    None
}
