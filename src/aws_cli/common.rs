use serde::Deserialize;
use std::process::Command;
use std::sync::Mutex;

static REGION: Mutex<Option<String>> = Mutex::new(None);

pub fn set_region(region: &str) {
    if let Ok(mut r) = REGION.lock() {
        *r = Some(region.to_string());
    }
}

pub fn get_region_args() -> Vec<String> {
    if let Ok(r) = REGION.lock()
        && let Some(ref region) = *r
    {
        return vec!["--region".to_string(), region.clone()];
    }
    Vec::new()
}

// Async Runtime Helper
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static SDK_CONFIG: OnceLock<tokio::sync::Mutex<Option<aws_config::SdkConfig>>> = OnceLock::new();

pub fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

pub fn get_sdk_config() -> aws_config::SdkConfig {
    let runtime = get_runtime();

    // Check if we already have a cached config
    let has_config = if let Some(mutex) = SDK_CONFIG.get() {
        if let Ok(guard) = runtime.block_on(async { mutex.try_lock() }) {
            guard.is_some()
        } else {
            false
        }
    } else {
        false
    };

    if has_config {
        let mutex = SDK_CONFIG.get().unwrap();
        return runtime.block_on(async {
            let guard = mutex.lock().await;
            guard.as_ref().unwrap().clone()
        });
    }

    // Load new config - read credentials directly from file (skip SSO completely)
    let app_region = {
        let r = REGION.lock().unwrap();
        r.clone()
    };

    let config = runtime.block_on(async {
        // Read credentials directly from ~/.aws/credentials file
        let credentials = read_credentials_from_file().await;

        // Use app region if set, otherwise use default region (ap-southeast-1)
        let region_str = app_region.unwrap_or_else(|| "ap-southeast-1".to_string());
        let region = aws_config::Region::new(region_str);

        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest()).region(region);

        if let Some(creds) = credentials {
            loader = loader.credentials_provider(creds);
        }

        loader.load().await
    });

    config
}

/// Read AWS credentials directly from ~/.aws/credentials file
async fn read_credentials_from_file() -> Option<aws_credential_types::Credentials> {
    use std::fs;
    use std::path::PathBuf;

    let home = dirs::home_dir()?;
    let creds_path = home.join(".aws").join("credentials");

    let contents = fs::read_to_string(&creds_path).ok()?;

    let mut access_key = None;
    let mut secret_key = None;
    let mut in_default = false;

    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with("[default]") {
            in_default = true;
            continue;
        }
        if line.starts_with('[') {
            in_default = false;
            continue;
        }
        if in_default {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "aws_access_key_id" => access_key = Some(value.to_string()),
                    "aws_secret_access_key" => secret_key = Some(value.to_string()),
                    _ => {}
                }
            }
        }
    }

    match (access_key, secret_key) {
        (Some(ak), Some(sk)) => Some(aws_credential_types::Credentials::new(
            ak,
            sk,
            None,
            None,
            "credentials-file",
        )),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct AwsResource {
    pub name: String,
    pub id: String,
    pub state: String,
    pub az: String,
    pub cidr: String,
}

impl AwsResource {
    pub fn display(&self) -> String {
        if self.name.is_empty() {
            self.id.clone()
        } else {
            format!("{} ({})", self.name, self.id)
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Tag {
    pub key: String,
    pub value: String,
}

pub fn run_aws_cli(args: &[&str]) -> Option<String> {
    use std::io::Write;

    let region_args = get_region_args();
    let mut cmd = Command::new("aws");
    cmd.args(args);
    for arg in &region_args {
        cmd.arg(arg);
    }

    let cmd_str = format!("aws {} {}", args.join(" "), region_args.join(" "));
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/emd_debug.log")
    {
        let _ = writeln!(f, "[START] {}", cmd_str);
    }

    let output = cmd.output().ok()?;

    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/awsmd_debug.log")
    {
        let _ = writeln!(
            f,
            "[END] {} - success: {}, stdout_len: {}",
            cmd_str,
            output.status.success(),
            output.stdout.len()
        );
    }

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

pub fn check_aws_login() -> Result<String, String> {
    let output = Command::new("aws")
        .args(["sts", "get-caller-identity", "--output", "json"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let json = String::from_utf8_lossy(&o.stdout);
            let account = extract_json_value(&json, "Account").unwrap_or_default();
            let arn = extract_json_value(&json, "Arn").unwrap_or_default();
            Ok(format!("{} ({})", account, arn))
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            Err(format!(
                "AWS 로그인 필요: {}",
                err.lines().next().unwrap_or("")
            ))
        }
        Err(e) => Err(format!("AWS CLI 실행 실패: {}", e)),
    }
}

pub fn extract_json_value(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\": \"", key);
    if let Some(start) = json.find(&pattern) {
        let offset = start + pattern.len();
        if let Some(end) = json[offset..].find('"') {
            return Some(json[offset..offset + end].to_string());
        }
    }
    None
}

pub fn parse_name_tag(tags_json: &str) -> String {
    if let Some(start) = tags_json.find("\"Key\": \"Name\"")
        && let Some(value_start) = tags_json[start..].find("\"Value\": \"")
    {
        let offset = start + value_start + 10;
        if let Some(end) = tags_json[offset..].find('"') {
            return tags_json[offset..offset + end].to_string();
        }
    }
    if let Some(start) = tags_json.find("\"Value\": \"") {
        let offset = start + 10;
        if let Some(end) = tags_json[offset..].find('"') {
            let value = &tags_json[offset..offset + end];
            if tags_json[offset + end..].contains("\"Key\": \"Name\"") {
                return value.to_string();
            }
        }
    }
    String::new()
}

pub fn extract_tags(json: &str) -> Vec<(String, String)> {
    let mut tags = Vec::new();
    let mut search_start = 0;

    while let Some(key_pos) = json[search_start..].find("\"Key\": \"") {
        let key_start = search_start + key_pos + 8;
        if let Some(key_end) = json[key_start..].find('"') {
            let key = json[key_start..key_start + key_end].to_string();

            if let Some(val_pos) = json[key_start..].find("\"Value\": \"") {
                let val_start = key_start + val_pos + 10;
                if let Some(val_end) = json[val_start..].find('"') {
                    let value = json[val_start..val_start + val_end].to_string();
                    if !tags.iter().any(|(k, _)| k == &key) {
                        tags.push((key, value));
                    }
                }
            }
            search_start = key_start + key_end;
        } else {
            break;
        }
    }
    tags
}

pub fn parse_resources_from_json(json: &str, prefix: &str) -> Vec<AwsResource> {
    let mut resources = Vec::new();

    let mut search_start = 0;
    while let Some(pos) = json[search_start..].find(prefix) {
        let start = search_start + pos;
        if let Some(end) = json[start..].find('"') {
            let id = &json[start..start + end];
            if id.starts_with(prefix) && !id.contains(' ') {
                let section_end = json[start..]
                    .find(']')
                    .map(|p| start + p)
                    .unwrap_or(json.len());
                let tag_start = start;
                let tag_end = section_end;
                let tags_json = &json[tag_start..tag_end];
                let name = parse_name_tag(tags_json);

                resources.push(AwsResource {
                    name,
                    id: id.to_string(),
                    state: String::new(),
                    az: String::new(),
                    cidr: String::new(),
                });
            }
            search_start = start + end;
        } else {
            break;
        }
    }
    resources
}
