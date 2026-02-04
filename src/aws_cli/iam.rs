use crate::aws_cli::common::{AwsResource, extract_json_value, run_aws_cli};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct IamRoleDetail {
    pub name: String,
    pub arn: String,
    pub role_id: String,
    pub path: String,
    pub create_date: String,
    pub description: String,
    pub max_session_duration: i64,
    pub assume_role_policy: String,
    pub attached_policies: Vec<AttachedPolicy>,
    pub inline_policies: Vec<InlinePolicy>,
    pub instance_profiles: Vec<String>,
    pub tags: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct AttachedPolicy {
    pub name: String,
    pub arn: String,
}

#[derive(Debug, Clone)]
pub struct InlinePolicy {
    pub name: String,
    pub document: String,
}

impl IamRoleDetail {
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("## IAM 역할 ({})\n", self.name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 역할 이름 | {} |", self.name),
            format!("| ARN | {} |", self.arn),
            format!("| 역할 ID | {} |", self.role_id),
            format!("| 경로 | {} |", self.path),
            format!("| 생성일 | {} |", self.create_date),
        ];

        if !self.description.is_empty() {
            lines.push(format!("| 설명 | {} |", self.description));
        }

        lines.push(format!(
            "| 최대 세션 기간 | {} 초 |",
            self.max_session_duration
        ));

        // 태그
        for (key, value) in &self.tags {
            lines.push(format!("| 태그-{} | {} |", key, value));
        }

        // 인스턴스 프로파일
        if !self.instance_profiles.is_empty() {
            lines.push(String::new());
            lines.push("### 인스턴스 프로파일\n".to_string());
            for profile in &self.instance_profiles {
                lines.push(format!("- {}", profile));
            }
        }

        // 신뢰 정책
        lines.push(String::new());
        lines.push("### 신뢰 관계 (Trust Policy)\n".to_string());
        lines.push("```json".to_string());
        lines.push(self.assume_role_policy.clone());
        lines.push("```".to_string());

        // 연결된 정책
        if !self.attached_policies.is_empty() {
            lines.push(String::new());
            lines.push("### 연결된 정책 (Attached Policies)\n".to_string());
            lines.push("| 정책 이름 | ARN |".to_string());
            lines.push("|:---|:---|".to_string());
            for policy in &self.attached_policies {
                lines.push(format!("| {} | {} |", policy.name, policy.arn));
            }
        }

        // 인라인 정책
        if !self.inline_policies.is_empty() {
            lines.push(String::new());
            lines.push("### 인라인 정책 (Inline Policies)\n".to_string());
            for policy in &self.inline_policies {
                lines.push(format!("#### {}\n", policy.name));
                lines.push("```json".to_string());
                lines.push(policy.document.clone());
                lines.push("```".to_string());
            }
        }

        lines.join("\n") + "\n"
    }
}

pub fn list_iam_roles() -> Vec<AwsResource> {
    let output = match run_aws_cli(&["iam", "list-roles", "--output", "json"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    parse_roles_resources(&output)
}

fn parse_roles_resources(json: &str) -> Vec<AwsResource> {
    let mut resources = Vec::new();

    #[derive(Deserialize)]
    struct RolesResponse {
        #[serde(rename = "Roles")]
        roles: Vec<Role>,
    }

    #[derive(Deserialize)]
    struct Role {
        #[serde(rename = "RoleName")]
        role_name: String,
        #[serde(rename = "Arn")]
        arn: String,
        #[serde(rename = "Path")]
        path: String,
    }

    if let Ok(response) = serde_json::from_str::<RolesResponse>(json) {
        for role in response.roles {
            // AWS 서비스 역할은 제외 (선택적)
            let display_name = if role.path == "/" {
                role.role_name.clone()
            } else {
                format!("{}{}", role.path, role.role_name)
            };

            resources.push(AwsResource {
                name: display_name,
                id: role.role_name,
                state: String::new(),
                az: String::new(),
                cidr: role.arn,
            });
        }
    }

    resources
}

pub fn get_iam_role_detail(role_name: &str) -> Option<IamRoleDetail> {
    // 역할 기본 정보
    let output = run_aws_cli(&["iam", "get-role", "--role-name", role_name, "--output", "json"])?;

    let name = extract_json_value(&output, "RoleName").unwrap_or_default();
    let arn = extract_json_value(&output, "Arn").unwrap_or_default();
    let role_id = extract_json_value(&output, "RoleId").unwrap_or_default();
    let path = extract_json_value(&output, "Path").unwrap_or_default();
    let create_date = extract_json_value(&output, "CreateDate").unwrap_or_default();
    let description = extract_json_value(&output, "Description").unwrap_or_default();
    let max_session_str = extract_json_value(&output, "MaxSessionDuration").unwrap_or_default();
    let max_session_duration = max_session_str.parse::<i64>().unwrap_or(3600);

    // AssumeRolePolicyDocument 파싱
    let assume_role_policy = extract_assume_role_policy(&output);

    // 태그
    let tags = extract_role_tags(&output);

    // 연결된 정책
    let attached_policies = get_attached_policies(role_name);

    // 인라인 정책
    let inline_policies = get_inline_policies(role_name);

    // 인스턴스 프로파일
    let instance_profiles = get_instance_profiles(role_name);

    Some(IamRoleDetail {
        name,
        arn,
        role_id,
        path,
        create_date,
        description,
        max_session_duration,
        assume_role_policy,
        attached_policies,
        inline_policies,
        instance_profiles,
        tags,
    })
}

fn extract_assume_role_policy(json: &str) -> String {
    // AssumeRolePolicyDocument는 URL 인코딩된 JSON
    if let Some(start) = json.find("\"AssumeRolePolicyDocument\":") {
        let after_key = &json[start + 27..];
        // JSON 객체 찾기
        if let Some(obj_start) = after_key.find('{') {
            let obj_json = &after_key[obj_start..];
            let mut depth = 0;
            let mut end_idx = 0;
            for (i, c) in obj_json.chars().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end_idx > 0 {
                let policy_json = &obj_json[..end_idx];
                // pretty print
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(policy_json) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                        return pretty;
                    }
                }
                return policy_json.to_string();
            }
        }
    }
    String::new()
}

fn extract_role_tags(json: &str) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    #[derive(Deserialize)]
    struct Tag {
        #[serde(rename = "Key")]
        key: String,
        #[serde(rename = "Value")]
        value: String,
    }

    // Tags 배열 찾기
    if let Some(start) = json.find("\"Tags\":") {
        let after_key = &json[start + 7..];
        if let Some(arr_start) = after_key.find('[') {
            let arr_json = &after_key[arr_start..];
            if let Some(arr_end) = arr_json.find(']') {
                let tags_str = &arr_json[..arr_end + 1];
                if let Ok(parsed_tags) = serde_json::from_str::<Vec<Tag>>(tags_str) {
                    for tag in parsed_tags {
                        tags.push((tag.key, tag.value));
                    }
                }
            }
        }
    }

    tags
}

fn get_attached_policies(role_name: &str) -> Vec<AttachedPolicy> {
    let output = match run_aws_cli(&[
        "iam",
        "list-attached-role-policies",
        "--role-name",
        role_name,
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    #[derive(Deserialize)]
    struct Response {
        #[serde(rename = "AttachedPolicies")]
        attached_policies: Vec<Policy>,
    }

    #[derive(Deserialize)]
    struct Policy {
        #[serde(rename = "PolicyName")]
        policy_name: String,
        #[serde(rename = "PolicyArn")]
        policy_arn: String,
    }

    if let Ok(response) = serde_json::from_str::<Response>(&output) {
        return response
            .attached_policies
            .into_iter()
            .map(|p| AttachedPolicy {
                name: p.policy_name,
                arn: p.policy_arn,
            })
            .collect();
    }

    Vec::new()
}

fn get_inline_policies(role_name: &str) -> Vec<InlinePolicy> {
    // 인라인 정책 목록
    let output = match run_aws_cli(&[
        "iam",
        "list-role-policies",
        "--role-name",
        role_name,
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    #[derive(Deserialize)]
    struct Response {
        #[serde(rename = "PolicyNames")]
        policy_names: Vec<String>,
    }

    let policy_names = match serde_json::from_str::<Response>(&output) {
        Ok(r) => r.policy_names,
        Err(_) => return Vec::new(),
    };

    let mut policies = Vec::new();

    for policy_name in policy_names {
        if let Some(doc) = get_inline_policy_document(role_name, &policy_name) {
            policies.push(InlinePolicy {
                name: policy_name,
                document: doc,
            });
        }
    }

    policies
}

fn get_inline_policy_document(role_name: &str, policy_name: &str) -> Option<String> {
    let output = run_aws_cli(&[
        "iam",
        "get-role-policy",
        "--role-name",
        role_name,
        "--policy-name",
        policy_name,
        "--output",
        "json",
    ])?;

    // PolicyDocument 추출
    if let Some(start) = output.find("\"PolicyDocument\":") {
        let after_key = &output[start + 17..];
        if let Some(obj_start) = after_key.find('{') {
            let obj_json = &after_key[obj_start..];
            let mut depth = 0;
            let mut end_idx = 0;
            for (i, c) in obj_json.chars().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end_idx > 0 {
                let policy_json = &obj_json[..end_idx];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(policy_json) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                        return Some(pretty);
                    }
                }
                return Some(policy_json.to_string());
            }
        }
    }

    None
}

fn get_instance_profiles(role_name: &str) -> Vec<String> {
    let output = match run_aws_cli(&[
        "iam",
        "list-instance-profiles-for-role",
        "--role-name",
        role_name,
        "--output",
        "json",
    ]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    #[derive(Deserialize)]
    struct Response {
        #[serde(rename = "InstanceProfiles")]
        instance_profiles: Vec<Profile>,
    }

    #[derive(Deserialize)]
    struct Profile {
        #[serde(rename = "InstanceProfileName")]
        instance_profile_name: String,
    }

    if let Ok(response) = serde_json::from_str::<Response>(&output) {
        return response
            .instance_profiles
            .into_iter()
            .map(|p| p.instance_profile_name)
            .collect();
    }

    Vec::new()
}
