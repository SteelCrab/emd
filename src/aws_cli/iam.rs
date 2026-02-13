use crate::aws_cli::common::{extract_json_value, run_aws_cli};
use serde::Deserialize;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IamRoleDetail {
    pub name: String,
    pub arn: String,
    pub assume_role_policy: String,
    pub attached_policies: Vec<AttachedPolicy>,
    pub inline_policies: Vec<InlinePolicy>,
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

pub fn get_iam_role_detail(role_name: &str) -> Option<IamRoleDetail> {
    // 역할 기본 정보
    let output = run_aws_cli(&[
        "iam",
        "get-role",
        "--role-name",
        role_name,
        "--output",
        "json",
    ])?;
    let attached_policies = get_attached_policies(role_name);
    let inline_policies = get_inline_policies(role_name);
    build_iam_role_detail_from_output(&output, attached_policies, inline_policies)
}

fn extract_assume_role_policy(json: &str) -> String {
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    if let Some(policy) = v
        .get("Role")
        .and_then(|r| r.get("AssumeRolePolicyDocument"))
    {
        serde_json::to_string_pretty(policy).unwrap_or_default()
    } else {
        String::new()
    }
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

    parse_attached_policies_response(&output).unwrap_or_default()
}

fn get_inline_policies(role_name: &str) -> Vec<InlinePolicy> {
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

    let policy_names = match parse_role_policy_names_response(&output) {
        Some(names) => names,
        None => return Vec::new(),
    };
    build_inline_policies_from_names(policy_names, |policy_name| {
        get_inline_policy_document(role_name, policy_name)
    })
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

    parse_inline_policy_document_response(&output)
}

fn parse_attached_policies_response(output: &str) -> Option<Vec<AttachedPolicy>> {
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

    let response: Response = serde_json::from_str(output).ok()?;
    Some(
        response
            .attached_policies
            .into_iter()
            .map(|p| AttachedPolicy {
                name: p.policy_name,
                arn: p.policy_arn,
            })
            .collect(),
    )
}

fn parse_role_policy_names_response(output: &str) -> Option<Vec<String>> {
    #[derive(Deserialize)]
    struct Response {
        #[serde(rename = "PolicyNames")]
        policy_names: Vec<String>,
    }

    serde_json::from_str::<Response>(output)
        .ok()
        .map(|r| r.policy_names)
}

fn parse_inline_policy_document_response(output: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(output).ok()?;
    let policy = v.get("PolicyDocument")?;
    serde_json::to_string_pretty(policy).ok()
}

fn build_iam_role_detail_from_output(
    output: &str,
    attached_policies: Vec<AttachedPolicy>,
    inline_policies: Vec<InlinePolicy>,
) -> Option<IamRoleDetail> {
    let name = extract_json_value(output, "RoleName").unwrap_or_default();
    let arn = extract_json_value(output, "Arn").unwrap_or_default();
    let assume_role_policy = extract_assume_role_policy(output);

    if name.is_empty() || arn.is_empty() {
        return None;
    }

    Some(IamRoleDetail {
        name,
        arn,
        assume_role_policy,
        attached_policies,
        inline_policies,
    })
}

fn build_inline_policies_from_names<F>(
    policy_names: Vec<String>,
    mut fetch_doc: F,
) -> Vec<InlinePolicy>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut policies = Vec::new();

    for policy_name in policy_names {
        if let Some(doc) = fetch_doc(&policy_name) {
            policies.push(InlinePolicy {
                name: policy_name,
                document: doc,
            });
        }
    }

    policies
}

#[cfg(test)]
mod tests {
    use super::{
        build_iam_role_detail_from_output, build_inline_policies_from_names,
        extract_assume_role_policy, parse_attached_policies_response,
        parse_inline_policy_document_response, parse_role_policy_names_response,
    };

    #[test]
    fn extract_assume_role_policy_returns_pretty_json() {
        let payload = r#"
            {
              "Role": {
                "AssumeRolePolicyDocument": {
                  "Version": "2012-10-17",
                  "Statement": []
                }
              }
            }
        "#;
        let policy = extract_assume_role_policy(payload);
        assert!(policy.contains("\"Version\""));
        assert!(policy.contains("2012-10-17"));
    }

    #[test]
    fn extract_assume_role_policy_returns_empty_on_invalid_payload() {
        assert_eq!(extract_assume_role_policy("not-json"), "");
        assert_eq!(extract_assume_role_policy(r#"{"Role":{}}"#), "");
    }

    #[test]
    fn parse_attached_policies_response_maps_policy_fields() {
        let payload = r#"
            {
              "AttachedPolicies": [
                {"PolicyName": "ReadOnlyAccess", "PolicyArn": "arn:aws:iam::aws:policy/ReadOnlyAccess"}
              ]
            }
        "#;

        let policies = parse_attached_policies_response(payload).expect("parse policies");
        assert_eq!(policies.len(), 1);
        assert_eq!(policies[0].name, "ReadOnlyAccess");
    }

    #[test]
    fn parse_role_policy_names_response_reads_names() {
        let payload = r#"{"PolicyNames":["inline-a","inline-b"]}"#;
        let names = parse_role_policy_names_response(payload).expect("parse policy names");
        assert_eq!(names, vec!["inline-a".to_string(), "inline-b".to_string()]);
    }

    #[test]
    fn parse_inline_policy_document_response_returns_json_string() {
        let payload = r#"
            {
              "PolicyDocument": {
                "Version": "2012-10-17",
                "Statement": []
              }
            }
        "#;
        let doc = parse_inline_policy_document_response(payload).expect("parse inline doc");
        assert!(doc.contains("\"Version\""));
        assert!(doc.contains("2012-10-17"));
    }

    #[test]
    fn build_iam_role_detail_from_output_requires_name_and_arn() {
        let payload = r#"
            {
              "Role": {
                "RoleName": "role-a",
                "Arn": "arn:aws:iam::123456789012:role/role-a",
                "AssumeRolePolicyDocument": {"Version":"2012-10-17"}
              }
            }
        "#;
        let detail = build_iam_role_detail_from_output(payload, vec![], vec![]).expect("detail");
        assert_eq!(detail.name, "role-a");
        assert_eq!(detail.arn, "arn:aws:iam::123456789012:role/role-a");
        assert!(detail.assume_role_policy.contains("Version"));

        let invalid = build_iam_role_detail_from_output(r#"{"Role":{}}"#, vec![], vec![]);
        assert!(invalid.is_none());
    }

    #[test]
    fn build_inline_policies_from_names_skips_missing_documents() {
        let names = vec![
            "inline-a".to_string(),
            "inline-b".to_string(),
            "inline-c".to_string(),
        ];
        let policies = build_inline_policies_from_names(names, |name| match name {
            "inline-a" => Some("{\"Statement\":[]}".to_string()),
            "inline-c" => Some("{\"Statement\":[{\"Effect\":\"Allow\"}]}".to_string()),
            _ => None,
        });
        assert_eq!(policies.len(), 2);
        assert_eq!(policies[0].name, "inline-a");
        assert_eq!(policies[1].name, "inline-c");
    }
}
