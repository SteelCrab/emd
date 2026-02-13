use crate::i18n::{I18n, Language};
use serde::Deserialize;

pub use crate::aws_cli::ecr_sdk::{get_ecr_detail, list_ecr_repositories};

#[derive(Debug, Deserialize)]
pub(super) struct EcrRepositoriesResponse {
    pub(super) repositories: Vec<EcrRepository>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EcrRepository {
    pub(super) repository_name: String,
    pub(super) repository_uri: String,
    #[serde(default)]
    pub(super) image_tag_mutability: String,
    #[serde(default)]
    pub(super) encryption_configuration: Option<EcrEncryptionConfiguration>,
    #[serde(default)]
    pub(super) created_at: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EcrEncryptionConfiguration {
    pub(super) encryption_type: String,
    #[serde(default)]
    pub(super) kms_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EcrImagesResponse {
    pub(super) image_details: Vec<EcrImageDetail>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EcrImageDetail {
    #[allow(dead_code)]
    pub(super) image_digest: String,
}

#[derive(Debug)]
pub struct EcrDetail {
    pub name: String,
    pub uri: String,
    pub tag_mutability: String,
    pub encryption_type: String,
    pub kms_key: Option<String>,
    pub created_at: String,
    pub image_count: i32,
}

pub(super) fn mutability_label(tag_mutability: &str) -> &'static str {
    if tag_mutability == "IMMUTABLE" {
        "Immutable"
    } else {
        "Mutable"
    }
}

pub(super) fn format_created_at(created_at: Option<f64>) -> String {
    created_at
        .map(|ts| {
            let secs = ts as i64;
            chrono::DateTime::from_timestamp(secs, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "-".to_string())
        })
        .unwrap_or_else(|| "-".to_string())
}

impl EcrDetail {
    pub fn to_markdown(&self, lang: Language) -> String {
        let i18n = I18n::new(lang);
        let encryption_display = if self.encryption_type == "KMS" {
            if let Some(ref key) = self.kms_key {
                format!("AWS KMS ({})", key)
            } else {
                "AWS KMS".to_string()
            }
        } else {
            "AES-256".to_string()
        };

        let lines = vec![
            format!("## {} ({})\n", i18n.md_ecr_repository(), self.name),
            format!("| {} | {} |", i18n.item(), i18n.value()),
            "|:---|:---|".to_string(),
            format!("| {} | {} |", i18n.md_name(), self.name),
            format!("| URI | {} |", self.uri),
            format!("| {} | {} |", i18n.md_tag_mutability(), self.tag_mutability),
            format!("| {} | {} |", i18n.md_encryption(), encryption_display),
            format!("| {} | {} |", i18n.md_image_count(), self.image_count),
            format!("| {} | {} |", i18n.md_created_at(), self.created_at),
        ];

        lines.join("\n") + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::{EcrDetail, format_created_at, mutability_label};
    use crate::i18n::Language;

    fn sample_detail() -> EcrDetail {
        EcrDetail {
            name: "repo-a".to_string(),
            uri: "123456789012.dkr.ecr.ap-northeast-2.amazonaws.com/repo-a".to_string(),
            tag_mutability: "IMMUTABLE".to_string(),
            encryption_type: "AES256".to_string(),
            kms_key: None,
            created_at: "2026-02-13".to_string(),
            image_count: 3,
        }
    }

    #[test]
    fn scenario_ecr_mutability_immutable_label() {
        assert_eq!(mutability_label("IMMUTABLE"), "Immutable");
    }

    #[test]
    fn scenario_ecr_mutability_mutable_label() {
        assert_eq!(mutability_label("MUTABLE"), "Mutable");
    }

    #[test]
    fn scenario_ecr_created_at_format() {
        assert_eq!(format_created_at(Some(1_706_284_800.0)), "2024-01-26");
    }

    #[test]
    fn scenario_ecr_created_at_fallback() {
        assert_eq!(format_created_at(None), "-");
    }

    #[test]
    fn scenario_ecr_markdown_kms_key_render() {
        let mut detail = sample_detail();
        detail.encryption_type = "KMS".to_string();
        detail.kms_key = Some("arn:aws:kms:ap-northeast-2:123456789012:key/abcd".to_string());

        let markdown = detail.to_markdown(Language::Korean);
        assert!(markdown.contains("AWS KMS (arn:aws:kms:ap-northeast-2:123456789012:key/abcd)"));
    }

    #[test]
    fn scenario_ecr_markdown_aes_render() {
        let detail = sample_detail();
        let markdown = detail.to_markdown(Language::English);
        assert!(markdown.contains("AES-256"));
    }
}
