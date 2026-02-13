pub use crate::aws_cli::asg_sdk::{get_asg_detail, list_auto_scaling_groups};

#[derive(Debug, Clone)]
pub struct ScalingPolicy {
    pub name: String,
    pub policy_type: String,
    pub adjustment_type: Option<String>,
    pub scaling_adjustment: Option<i32>,
    pub cooldown: Option<i32>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AsgDetail {
    pub name: String,
    pub arn: String,
    pub launch_template_name: Option<String>,
    pub launch_template_id: Option<String>,
    pub launch_config_name: Option<String>,
    pub min_size: i32,
    pub max_size: i32,
    pub desired_capacity: i32,
    pub default_cooldown: i32,
    pub availability_zones: Vec<String>,
    pub target_group_arns: Vec<String>,
    pub health_check_type: String,
    pub health_check_grace_period: i32,
    pub instances: Vec<String>,
    pub created_time: String,
    pub scaling_policies: Vec<ScalingPolicy>,
    pub tags: Vec<(String, String)>,
}

impl AsgDetail {
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("## Auto Scaling Group ({})\n", self.name),
            "| 항목 | 값 |".to_string(),
            "|:---|:---|".to_string(),
            format!("| 이름 | {} |", self.name),
        ];

        // Launch Template or Config
        if let Some(ref lt_name) = self.launch_template_name {
            if let Some(ref lt_id) = self.launch_template_id {
                lines.push(format!("| 시작 템플릿 | {} (`{}`) |", lt_name, lt_id));
            } else {
                lines.push(format!("| 시작 템플릿 | {} |", lt_name));
            }
        } else if let Some(ref lc_name) = self.launch_config_name {
            lines.push(format!("| 시작 구성 | {} |", lc_name));
        }

        lines.push(format!("| 최소 크기 | {} |", self.min_size));
        lines.push(format!("| 최대 크기 | {} |", self.max_size));
        lines.push(format!("| 원하는 용량 | {} |", self.desired_capacity));
        lines.push(format!("| 기본 쿨다운 | {}초 |", self.default_cooldown));
        lines.push(format!("| 헬스 체크 유형 | {} |", self.health_check_type));
        lines.push(format!(
            "| 헬스 체크 유예 기간 | {}초 |",
            self.health_check_grace_period
        ));
        lines.push(format!("| 생성일 | {} |", self.created_time));

        // Availability Zones
        if !self.availability_zones.is_empty() {
            lines.push(String::new());
            lines.push("### 가용 영역\n".to_string());
            for az in &self.availability_zones {
                lines.push(format!("- {}", az));
            }
        }

        // Instances
        if !self.instances.is_empty() {
            lines.push(String::new());
            lines.push(format!("### 인스턴스 ({} 개)\n", self.instances.len()));
            lines.push("| 인스턴스 ID |".to_string());
            lines.push("|:---|".to_string());
            for inst in &self.instances {
                lines.push(format!("| `{}` |", inst));
            }
        }

        // Target Groups
        if !self.target_group_arns.is_empty() {
            lines.push(String::new());
            lines.push("### 대상 그룹\n".to_string());
            for tg in &self.target_group_arns {
                let tg_name = tg.split('/').nth(1).unwrap_or(tg);
                lines.push(format!("- {}", tg_name));
            }
        }

        // Scaling Policies
        if !self.scaling_policies.is_empty() {
            lines.push(String::new());
            lines.push("### 조정 정책\n".to_string());
            lines.push("| 이름 | 유형 | 조정 유형 | 조정 값 | 쿨다운 |".to_string());
            lines.push("|:---|:---|:---|:---|:---|".to_string());
            for policy in &self.scaling_policies {
                let adj_type = policy.adjustment_type.as_deref().unwrap_or("-");
                let adj_val = policy
                    .scaling_adjustment
                    .map(|v| v.to_string())
                    .unwrap_or("-".to_string());
                let cooldown = policy
                    .cooldown
                    .map(|v| format!("{}초", v))
                    .unwrap_or("-".to_string());
                lines.push(format!(
                    "| {} | {} | {} | {} | {} |",
                    policy.name, policy.policy_type, adj_type, adj_val, cooldown
                ));
            }
        }

        // Tags
        if !self.tags.is_empty() {
            lines.push(String::new());
            lines.push("### 태그\n".to_string());
            lines.push("| 키 | 값 |".to_string());
            lines.push("|:---|:---|".to_string());
            for (key, value) in &self.tags {
                if key != "Name" {
                    lines.push(format!("| {} | {} |", key, value));
                }
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::{AsgDetail, ScalingPolicy};

    fn sample_asg_detail() -> AsgDetail {
        AsgDetail {
            name: "asg-prod".to_string(),
            arn: "arn:aws:autoscaling:ap-northeast-2:123456789012:autoScalingGroup:abcd:autoScalingGroupName/asg-prod".to_string(),
            launch_template_name: Some("lt-web".to_string()),
            launch_template_id: Some("lt-0123456789abcdef0".to_string()),
            launch_config_name: None,
            min_size: 1,
            max_size: 4,
            desired_capacity: 2,
            default_cooldown: 300,
            availability_zones: vec!["ap-northeast-2a".to_string(), "ap-northeast-2c".to_string()],
            target_group_arns: vec![
                "arn:aws:elasticloadbalancing:ap-northeast-2:123456789012:targetgroup/web-blue/1111111111111111".to_string(),
            ],
            health_check_type: "EC2".to_string(),
            health_check_grace_period: 120,
            instances: vec!["i-aaa111".to_string(), "i-bbb222".to_string()],
            created_time: "2026-02-13".to_string(),
            scaling_policies: vec![ScalingPolicy {
                name: "scale-out".to_string(),
                policy_type: "SimpleScaling".to_string(),
                adjustment_type: Some("ChangeInCapacity".to_string()),
                scaling_adjustment: Some(1),
                cooldown: Some(60),
            }],
            tags: vec![
                ("Name".to_string(), "asg-prod".to_string()),
                ("Env".to_string(), "prod".to_string()),
            ],
        }
    }

    #[test]
    fn scenario_asg_markdown_launch_template_render() {
        let detail = sample_asg_detail();
        let markdown = detail.to_markdown();
        assert!(markdown.contains("| 시작 템플릿 | lt-web (`lt-0123456789abcdef0`) |"));
    }

    #[test]
    fn scenario_asg_markdown_launch_config_fallback() {
        let mut detail = sample_asg_detail();
        detail.launch_template_name = None;
        detail.launch_template_id = None;
        detail.launch_config_name = Some("legacy-launch-config".to_string());
        let markdown = detail.to_markdown();
        assert!(markdown.contains("| 시작 구성 | legacy-launch-config |"));
    }

    #[test]
    fn scenario_asg_markdown_instances_section_count() {
        let detail = sample_asg_detail();
        let markdown = detail.to_markdown();
        assert!(markdown.contains("### 인스턴스 (2 개)"));
    }

    #[test]
    fn scenario_asg_markdown_target_group_name_extraction() {
        let detail = sample_asg_detail();
        let markdown = detail.to_markdown();
        assert!(markdown.contains("- web-blue"));
    }

    #[test]
    fn scenario_asg_markdown_scaling_policy_table() {
        let detail = sample_asg_detail();
        let markdown = detail.to_markdown();
        assert!(markdown.contains("| scale-out | SimpleScaling | ChangeInCapacity | 1 | 60초 |"));
    }

    #[test]
    fn scenario_asg_markdown_tag_name_filtered() {
        let detail = sample_asg_detail();
        let markdown = detail.to_markdown();
        assert!(markdown.contains("| Env | prod |"));
        assert!(!markdown.contains("| Name | asg-prod |"));
    }

    #[test]
    fn scenario_asg_markdown_without_optional_sections() {
        let mut detail = sample_asg_detail();
        detail.availability_zones.clear();
        detail.target_group_arns.clear();
        detail.instances.clear();
        detail.scaling_policies.clear();
        detail.tags.clear();

        let markdown = detail.to_markdown();
        assert!(!markdown.contains("### 가용 영역"));
        assert!(!markdown.contains("### 대상 그룹"));
        assert!(!markdown.contains("### 조정 정책"));
        assert!(!markdown.contains("### 태그"));
    }
}
