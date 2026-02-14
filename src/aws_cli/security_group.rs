use crate::aws_cli::common::{AwsResource, Tag, run_aws_cli};
use crate::i18n::{I18n, Language};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SecurityGroupsResponse {
    security_groups: Vec<SecurityGroupInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SecurityGroupInfo {
    group_id: String,
    group_name: String,
    description: String,
    vpc_id: String,
    #[serde(default)]
    ip_permissions: Vec<IpPermission>,
    #[serde(default)]
    ip_permissions_egress: Vec<IpPermission>,
    #[serde(default)]
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct IpPermission {
    ip_protocol: String,
    #[serde(default)]
    from_port: Option<i32>,
    #[serde(default)]
    to_port: Option<i32>,
    #[serde(default)]
    ip_ranges: Vec<IpRange>,
    #[serde(default)]
    ipv6_ranges: Vec<Ipv6Range>,
    #[serde(default)]
    user_id_group_pairs: Vec<UserIdGroupPair>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct IpRange {
    cidr_ip: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Ipv6Range {
    cidr_ipv6: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UserIdGroupPair {
    group_id: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug)]
pub struct SecurityGroupDetail {
    pub name: String,
    pub id: String,
    pub description: String,
    pub vpc_id: String,
    pub inbound_rules: Vec<SecurityRule>,
    pub outbound_rules: Vec<SecurityRule>,
}

#[derive(Debug)]
pub struct SecurityRule {
    pub protocol: String,
    pub port_range: String,
    pub source_dest: String,
    pub description: String,
}

impl SecurityGroupDetail {
    pub fn to_markdown(&self, lang: Language) -> String {
        let i18n = I18n::new(lang);
        let display_name = if self.name.is_empty() || self.name == self.id {
            format!("NULL - {}", self.id)
        } else {
            format!("{} - {}", self.name, self.id)
        };
        let mut lines = vec![
            format!("## Security Group ({})\n", display_name),
            format!("| {} | {} |", i18n.item(), i18n.value()),
            "|:---|:---|".to_string(),
            format!("| {} | {} |", i18n.md_name(), display_name),
            format!("| {} | {} |", i18n.md_description(), self.description),
            format!("| VPC ID | {} |", self.vpc_id),
        ];

        if !self.inbound_rules.is_empty() {
            lines.push(format!("\n### {}", i18n.md_inbound_rules()));
            lines.push(format!(
                "| {} | {} | {} | {} |",
                i18n.md_protocol(),
                i18n.md_port_range(),
                i18n.md_source(),
                i18n.md_description()
            ));
            lines.push("|:---|:---|:---|:---|".to_string());
            for rule in &self.inbound_rules {
                lines.push(format!(
                    "| {} | {} | {} | {} |",
                    rule.protocol, rule.port_range, rule.source_dest, rule.description
                ));
            }
        }

        if !self.outbound_rules.is_empty() {
            lines.push(format!("\n### {}", i18n.md_outbound_rules()));
            lines.push(format!(
                "| {} | {} | {} | {} |",
                i18n.md_protocol(),
                i18n.md_port_range(),
                i18n.md_destination(),
                i18n.md_description()
            ));
            lines.push("|:---|:---|:---|:---|".to_string());
            for rule in &self.outbound_rules {
                lines.push(format!(
                    "| {} | {} | {} | {} |",
                    rule.protocol, rule.port_range, rule.source_dest, rule.description
                ));
            }
        }

        lines.join("\n") + "\n"
    }
}

pub fn list_security_groups() -> Vec<AwsResource> {
    let output = match run_aws_cli(&["ec2", "describe-security-groups", "--output", "json"]) {
        Some(o) => o,
        None => return Vec::new(),
    };

    parse_security_groups_list_output(&output).unwrap_or_default()
}

fn parse_security_groups_list_output(output: &str) -> Option<Vec<AwsResource>> {
    let response: SecurityGroupsResponse = match serde_json::from_str(output) {
        Ok(r) => r,
        Err(_) => return None,
    };

    Some(
        response
            .security_groups
            .into_iter()
            .map(|sg| {
                let name = sg
                    .tags
                    .iter()
                    .find(|t| t.key == "Name")
                    .map(|t| t.value.clone())
                    .unwrap_or_else(|| sg.group_name.clone());

                AwsResource {
                    name: format!("{} ({})", name, sg.group_name),
                    id: sg.group_id,
                    state: sg.vpc_id,
                    az: String::new(),
                    cidr: String::new(),
                }
            })
            .collect(),
    )
}

pub fn get_security_group_detail(sg_id: &str) -> Option<SecurityGroupDetail> {
    let output = run_aws_cli(&[
        "ec2",
        "describe-security-groups",
        "--group-ids",
        sg_id,
        "--output",
        "json",
    ])?;

    parse_security_group_detail_output(&output)
}

fn parse_security_group_detail_output(output: &str) -> Option<SecurityGroupDetail> {
    let response: SecurityGroupsResponse = serde_json::from_str(output).ok()?;
    let sg = response.security_groups.first()?;

    let name = sg
        .tags
        .iter()
        .find(|t| t.key == "Name")
        .map(|t| t.value.clone())
        .unwrap_or_else(|| sg.group_name.clone());

    let inbound_rules = parse_security_rules(&sg.ip_permissions);
    let outbound_rules = parse_security_rules(&sg.ip_permissions_egress);

    Some(SecurityGroupDetail {
        name,
        id: sg.group_id.clone(),
        description: sg.description.clone(),
        vpc_id: sg.vpc_id.clone(),
        inbound_rules,
        outbound_rules,
    })
}

fn parse_security_rules(permissions: &[IpPermission]) -> Vec<SecurityRule> {
    let mut rules = Vec::new();

    for perm in permissions {
        let protocol = match perm.ip_protocol.as_str() {
            "-1" => "All".to_string(),
            "tcp" => "TCP".to_string(),
            "udp" => "UDP".to_string(),
            "icmp" => "ICMP".to_string(),
            other => other.to_uppercase(),
        };

        let port_range = if perm.ip_protocol == "-1" {
            "All".to_string()
        } else if let (Some(from), Some(to)) = (perm.from_port, perm.to_port) {
            if from == to {
                from.to_string()
            } else {
                format!("{}-{}", from, to)
            }
        } else {
            "All".to_string()
        };

        for ip_range in &perm.ip_ranges {
            rules.push(SecurityRule {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source_dest: ip_range.cidr_ip.clone(),
                description: ip_range
                    .description
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            });
        }

        for ipv6_range in &perm.ipv6_ranges {
            rules.push(SecurityRule {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source_dest: ipv6_range.cidr_ipv6.clone(),
                description: ipv6_range
                    .description
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            });
        }

        for sg_pair in &perm.user_id_group_pairs {
            rules.push(SecurityRule {
                protocol: protocol.clone(),
                port_range: port_range.clone(),
                source_dest: format!("sg: {}", sg_pair.group_id),
                description: sg_pair
                    .description
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            });
        }
    }

    rules
}

#[cfg(test)]
mod tests {
    use super::{
        IpPermission, SecurityGroupDetail, SecurityRule, parse_security_group_detail_output,
        parse_security_groups_list_output, parse_security_rules,
    };
    use crate::i18n::Language;

    #[test]
    fn parse_security_rules_supports_ipv4_ipv6_and_sg_pair() {
        let json = r#"
            [
              {
                "IpProtocol":"tcp",
                "FromPort":80,
                "ToPort":80,
                "IpRanges":[{"CidrIp":"0.0.0.0/0","Description":"web"}],
                "Ipv6Ranges":[{"CidrIpv6":"::/0"}],
                "UserIdGroupPairs":[{"GroupId":"sg-1234","Description":"peer"}]
              }
            ]
        "#;
        let permissions: Vec<IpPermission> =
            serde_json::from_str(json).expect("deserialize permissions");
        let rules = parse_security_rules(&permissions);

        assert_eq!(rules.len(), 3);
        assert!(rules.iter().any(|r| r.source_dest == "0.0.0.0/0"));
        assert!(rules.iter().any(|r| r.source_dest == "::/0"));
        assert!(rules.iter().any(|r| r.source_dest == "sg: sg-1234"));
    }

    #[test]
    fn security_group_markdown_contains_inbound_and_outbound_sections() {
        let detail = SecurityGroupDetail {
            name: "sg-web".to_string(),
            id: "sg-1234".to_string(),
            description: "web sg".to_string(),
            vpc_id: "vpc-1111".to_string(),
            inbound_rules: vec![SecurityRule {
                protocol: "TCP".to_string(),
                port_range: "80".to_string(),
                source_dest: "0.0.0.0/0".to_string(),
                description: "web".to_string(),
            }],
            outbound_rules: vec![SecurityRule {
                protocol: "All".to_string(),
                port_range: "All".to_string(),
                source_dest: "0.0.0.0/0".to_string(),
                description: "-".to_string(),
            }],
        };

        let md = detail.to_markdown(Language::English);
        assert!(md.contains("## Security Group"));
        assert!(md.contains("Inbound Rules"));
        assert!(md.contains("Outbound Rules"));
    }

    #[test]
    fn parse_security_groups_list_output_prefers_name_tag_or_group_name() {
        let payload = r#"
        {
          "SecurityGroups": [
            {
              "GroupId": "sg-1",
              "GroupName": "web-default",
              "Description": "web",
              "VpcId": "vpc-1",
              "Tags": [{"Key":"Name","Value":"web-sg"}]
            },
            {
              "GroupId": "sg-2",
              "GroupName": "db-default",
              "Description": "db",
              "VpcId": "vpc-1",
              "Tags": []
            }
          ]
        }
        "#;
        let groups = parse_security_groups_list_output(payload).expect("groups");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].id, "sg-1");
        assert!(groups[0].name.contains("web-sg"));
        assert!(groups[1].name.contains("db-default"));
    }

    #[test]
    fn parse_security_group_detail_output_builds_detail_and_rules() {
        let payload = r#"
        {
          "SecurityGroups": [
            {
              "GroupId": "sg-1",
              "GroupName": "web",
              "Description": "web sg",
              "VpcId": "vpc-1",
              "IpPermissions": [
                {"IpProtocol":"tcp","FromPort":443,"ToPort":443,"IpRanges":[{"CidrIp":"0.0.0.0/0"}]}
              ],
              "IpPermissionsEgress": [
                {"IpProtocol":"-1","IpRanges":[{"CidrIp":"0.0.0.0/0"}]}
              ],
              "Tags": [{"Key":"Name","Value":"web-main"}]
            }
          ]
        }
        "#;
        let detail = parse_security_group_detail_output(payload).expect("detail");
        assert_eq!(detail.name, "web-main");
        assert_eq!(detail.id, "sg-1");
        assert_eq!(detail.inbound_rules.len(), 1);
        assert_eq!(detail.outbound_rules.len(), 1);
        assert_eq!(detail.inbound_rules[0].port_range, "443");
    }

    #[test]
    fn security_group_markdown_omits_rule_sections_when_empty() {
        let detail = SecurityGroupDetail {
            name: String::new(),
            id: "sg-0".to_string(),
            description: "empty".to_string(),
            vpc_id: "vpc-0".to_string(),
            inbound_rules: vec![],
            outbound_rules: vec![],
        };
        let md = detail.to_markdown(Language::English);
        assert!(md.contains("NULL - sg-0"));
        assert!(!md.contains("Inbound Rules"));
        assert!(!md.contains("Outbound Rules"));
    }
}
