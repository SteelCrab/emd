use crate::aws_cli::{self, AsgDetail, AwsResource, Ec2Detail, EcrDetail};
use crate::blueprint::{
    Blueprint, BlueprintResource, BlueprintStore, ResourceType, load_blueprints, save_blueprints,
};
use crate::i18n::{I18n, Language};
use crate::settings::{AppSettings, load_settings, save_settings};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    BlueprintSelect,
    BlueprintDetail,
    BlueprintNameInput,
    BlueprintPreview,
    RegionSelect,
    ServiceSelect,
    Ec2Select,
    VpcSelect,
    SecurityGroupSelect,
    LoadBalancerSelect,
    EcrSelect,
    AsgSelect,
    Preview,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadingTask {
    None,
    RefreshEc2,
    RefreshVpc,
    RefreshPreview,
    RefreshSecurityGroup,
    RefreshLoadBalancer,
    RefreshEcr,
    RefreshAsg,
    LoadEc2,
    LoadVpc,
    LoadSecurityGroup,
    LoadLoadBalancer,
    LoadEcr,
    LoadAsg,
    LoadEc2Detail(String),
    LoadVpcDetail(String, u8), // (vpc_id, step: 0-6)
    LoadSecurityGroupDetail(String),
    LoadLoadBalancerDetail(String),
    LoadEcrDetail(String),
    LoadAsgDetail(String),

    LoadBlueprintResources(usize), // (current_resource_index)
}

#[derive(Debug, Clone, Default)]
pub struct LoadingProgress {
    pub vpc_info: bool,
    pub subnets: bool,
    pub igws: bool,
    pub nats: bool,
    pub route_tables: bool,
    pub eips: bool,
    pub dns_attrs: bool,
}

impl LoadingProgress {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub struct Region {
    pub code: &'static str,
    pub name_ko: &'static str,
    pub name_en: &'static str,
}

impl Region {
    pub fn name(&self, lang: Language) -> &'static str {
        match lang {
            Language::Korean => self.name_ko,
            Language::English => self.name_en,
        }
    }
}

pub const REGIONS: &[Region] = &[
    Region {
        code: "ap-northeast-2",
        name_ko: "서울",
        name_en: "Seoul",
    },
    Region {
        code: "ap-northeast-1",
        name_ko: "도쿄",
        name_en: "Tokyo",
    },
    Region {
        code: "ap-northeast-3",
        name_ko: "오사카",
        name_en: "Osaka",
    },
    Region {
        code: "ap-southeast-1",
        name_ko: "싱가포르",
        name_en: "Singapore",
    },
    Region {
        code: "ap-southeast-2",
        name_ko: "시드니",
        name_en: "Sydney",
    },
    Region {
        code: "ap-south-1",
        name_ko: "뭄바이",
        name_en: "Mumbai",
    },
    Region {
        code: "us-east-1",
        name_ko: "버지니아",
        name_en: "N. Virginia",
    },
    Region {
        code: "us-east-2",
        name_ko: "오하이오",
        name_en: "Ohio",
    },
    Region {
        code: "us-west-1",
        name_ko: "캘리포니아",
        name_en: "N. California",
    },
    Region {
        code: "us-west-2",
        name_ko: "오레곤",
        name_en: "Oregon",
    },
    Region {
        code: "eu-west-1",
        name_ko: "아일랜드",
        name_en: "Ireland",
    },
    Region {
        code: "eu-central-1",
        name_ko: "프랑크푸르트",
        name_en: "Frankfurt",
    },
];

// Service names (excluding exit which is handled separately)
pub const SERVICE_KEYS: &[&str] = &[
    "EC2",
    "Network",
    "Security Group",
    "Load Balancer",
    "ECR",
    "ASG",
];

pub struct App {
    pub screen: Screen,
    pub running: bool,
    pub loading: bool,
    pub loading_task: LoadingTask,
    pub loading_progress: LoadingProgress,
    pub login_info: Option<String>,
    pub login_error: Option<String>,
    pub selected_region: usize,
    pub selected_service: usize,
    pub selected_index: usize,
    pub message: String,

    // Settings & i18n
    pub settings: AppSettings,
    pub i18n: I18n,
    pub selected_setting: usize,
    pub selected_tab: usize, // 0: Main, 1: Settings

    // AWS Resources
    pub instances: Vec<AwsResource>,
    pub vpcs: Vec<AwsResource>,
    pub security_groups: Vec<AwsResource>,
    pub load_balancers: Vec<AwsResource>,
    pub ecr_repositories: Vec<AwsResource>,
    pub auto_scaling_groups: Vec<AwsResource>,

    // Selected EC2 Detail
    pub ec2_detail: Option<Ec2Detail>,
    // Selected Network Detail
    pub network_detail: Option<aws_cli::NetworkDetail>,
    // Selected Security Group Detail
    pub sg_detail: Option<aws_cli::SecurityGroupDetail>,
    // Selected Load Balancer Detail
    pub lb_detail: Option<aws_cli::LoadBalancerDetail>,
    // Selected ECR Detail
    pub ecr_detail: Option<EcrDetail>,
    // Selected ASG Detail
    pub asg_detail: Option<AsgDetail>,

    // Preview
    pub preview_content: String,
    pub preview_filename: String,
    pub preview_scroll: u16,
    pub preview_drag_start: Option<(u16, u16)>, // (x, y) for drag start position

    // Blueprint
    pub blueprint_store: BlueprintStore,
    pub selected_blueprint_index: usize,
    pub current_blueprint: Option<Blueprint>,
    pub blueprint_mode: bool,
    pub blueprint_resource_index: usize,
    pub input_buffer: String,
    pub blueprint_markdown_parts: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        let blueprint_store = load_blueprints();
        let settings = load_settings();
        let i18n = I18n::new(settings.language);
        Self {
            screen: Screen::Login,
            running: true,
            loading: false,
            loading_task: LoadingTask::None,
            loading_progress: LoadingProgress::default(),
            login_info: None,
            login_error: None,
            selected_region: 0,
            selected_service: 0,
            selected_index: 0,
            message: String::new(),

            settings,
            i18n,
            selected_setting: 0,
            selected_tab: 0,

            instances: Vec::new(),
            vpcs: Vec::new(),
            security_groups: Vec::new(),
            load_balancers: Vec::new(),
            ecr_repositories: Vec::new(),
            auto_scaling_groups: Vec::new(),
            ec2_detail: None,
            network_detail: None,
            sg_detail: None,
            lb_detail: None,
            ecr_detail: None,
            asg_detail: None,

            preview_content: String::new(),
            preview_filename: String::new(),
            preview_scroll: 0,
            preview_drag_start: None,

            blueprint_store,
            selected_blueprint_index: 0,
            current_blueprint: None,
            blueprint_mode: false,
            blueprint_resource_index: 0,
            input_buffer: String::new(),
            blueprint_markdown_parts: Vec::new(),
        }
    }

    pub fn check_login(&mut self) {
        match aws_cli::check_aws_login() {
            Ok(info) => {
                self.login_info = Some(info);
                self.screen = Screen::BlueprintSelect;
            }
            Err(e) => {
                self.login_error = Some(e);
            }
        }
    }

    pub fn select_region(&mut self) {
        let region = REGIONS[self.selected_region].code;
        aws_cli::set_region(region);
        self.screen = Screen::ServiceSelect;
    }

    pub fn save_file(&mut self) -> Result<(), std::io::Error> {
        crate::output::save_markdown(&self.preview_filename, &self.preview_content)?;
        self.message = format!("{}: {}", self.i18n.save_complete(), self.preview_filename);
        Ok(())
    }

    // Settings methods
    pub fn toggle_language(&mut self) {
        self.settings.language = self.settings.language.toggle();
        self.i18n = I18n::new(self.settings.language);
        self.save_settings();
    }

    pub fn save_settings(&mut self) {
        if save_settings(&self.settings).is_ok() {
            self.message = self.i18n.settings_saved().to_string();
        }
    }

    // Blueprint methods
    pub fn save_blueprints(&mut self) {
        if save_blueprints(&self.blueprint_store).is_err() {
            self.message = "Save failed".to_string();
        } else {
            self.message = self.i18n.blueprint_saved().to_string();
        }
    }

    pub fn create_blueprint(&mut self, name: String) {
        let blueprint = Blueprint::new(name);
        self.blueprint_store.add_blueprint(blueprint);
        self.selected_blueprint_index = self.blueprint_store.blueprints.len() - 1;
        self.save_blueprints();
    }

    pub fn delete_blueprint(&mut self, index: usize) {
        self.blueprint_store.remove_blueprint(index);
        if self.selected_blueprint_index >= self.blueprint_store.blueprints.len()
            && self.selected_blueprint_index > 0
        {
            self.selected_blueprint_index -= 1;
        }
        self.save_blueprints();
        self.message = self.i18n.blueprint_deleted().to_string();
    }

    pub fn add_resource_to_current_blueprint(&mut self, resource: BlueprintResource) {
        if let Some(ref mut blueprint) = self.current_blueprint {
            blueprint.add_resource(resource);
            // Update in store
            if let Some(stored) = self
                .blueprint_store
                .get_blueprint_mut(self.selected_blueprint_index)
            {
                *stored = blueprint.clone();
            }
            self.save_blueprints();
            self.message = self.i18n.resource_added().to_string();
        }
    }

    pub fn remove_resource_from_current_blueprint(&mut self, index: usize) {
        if let Some(ref mut blueprint) = self.current_blueprint {
            blueprint.remove_resource(index);
            // Update in store
            if let Some(stored) = self
                .blueprint_store
                .get_blueprint_mut(self.selected_blueprint_index)
            {
                *stored = blueprint.clone();
            }
            self.save_blueprints();
            self.message = self.i18n.resource_deleted().to_string();
        }
    }

    pub fn move_resource_up(&mut self, index: usize) -> bool {
        if index == 0 {
            return false;
        }
        if let Some(ref mut blueprint) = self.current_blueprint
            && index < blueprint.resources.len()
        {
            blueprint.resources.swap(index, index - 1);
            // Update in store
            if let Some(stored) = self
                .blueprint_store
                .get_blueprint_mut(self.selected_blueprint_index)
            {
                *stored = blueprint.clone();
            }
            self.save_blueprints();
            return true;
        }
        false
    }

    pub fn move_resource_down(&mut self, index: usize) -> bool {
        if let Some(ref mut blueprint) = self.current_blueprint
            && index + 1 < blueprint.resources.len()
        {
            blueprint.resources.swap(index, index + 1);
            // Update in store
            if let Some(stored) = self
                .blueprint_store
                .get_blueprint_mut(self.selected_blueprint_index)
            {
                *stored = blueprint.clone();
            }
            self.save_blueprints();
            return true;
        }
        false
    }

    pub fn get_current_resource_type(&self) -> Option<ResourceType> {
        if self.ec2_detail.is_some() {
            Some(ResourceType::Ec2)
        } else if self.network_detail.is_some() {
            Some(ResourceType::Network)
        } else if self.sg_detail.is_some() {
            Some(ResourceType::SecurityGroup)
        } else if self.lb_detail.is_some() {
            Some(ResourceType::LoadBalancer)
        } else if self.ecr_detail.is_some() {
            Some(ResourceType::Ecr)
        } else if self.asg_detail.is_some() {
            Some(ResourceType::Asg)
        } else {
            None
        }
    }

    pub fn get_current_resource_info(&self) -> Option<(String, String)> {
        if let Some(ref detail) = self.ec2_detail {
            Some((detail.instance_id.clone(), detail.name.clone()))
        } else if let Some(ref detail) = self.network_detail {
            Some((detail.id.clone(), detail.name.clone()))
        } else if let Some(ref detail) = self.sg_detail {
            Some((detail.id.clone(), detail.name.clone()))
        } else if let Some(ref detail) = self.lb_detail {
            Some((detail.arn.clone(), detail.name.clone()))
        } else if let Some(ref detail) = self.ecr_detail {
            Some((detail.name.clone(), detail.name.clone()))
        } else {
            self.asg_detail
                .as_ref()
                .map(|detail| (detail.name.clone(), detail.name.clone()))
        }
    }

    pub fn get_current_region(&self) -> String {
        REGIONS[self.selected_region].code.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{App, LoadingProgress, REGIONS, Region};
    use crate::aws_cli::{
        AsgDetail, EcrDetail, Ec2Detail, EipDetail, LoadBalancerDetail, NatDetail, NetworkDetail,
        RouteTableDetail, ScalingPolicy, SecurityGroupDetail, SecurityRule, TargetGroupInfo,
    };
    use crate::aws_cli::iam::{AttachedPolicy, IamRoleDetail, InlinePolicy};
    use crate::blueprint::{Blueprint, BlueprintResource, ResourceType};

    fn sample_ec2_detail() -> Ec2Detail {
        Ec2Detail {
            name: "web-a".to_string(),
            instance_id: "i-0123456789abcdef0".to_string(),
            instance_type: "t3.micro".to_string(),
            ami: "ubuntu-22.04".to_string(),
            platform: "Linux".to_string(),
            architecture: "x86_64".to_string(),
            key_pair: "main-key".to_string(),
            vpc: "vpc-main".to_string(),
            subnet: "subnet-a".to_string(),
            az: "ap-northeast-2a".to_string(),
            public_ip: "1.1.1.1".to_string(),
            private_ip: "10.0.0.10".to_string(),
            security_groups: vec!["sg-web".to_string()],
            state: "running".to_string(),
            ebs_optimized: true,
            monitoring: "Enabled".to_string(),
            iam_role: Some("role-web".to_string()),
            iam_role_detail: Some(IamRoleDetail {
                name: "role-web".to_string(),
                arn: "arn:aws:iam::123456789012:role/role-web".to_string(),
                assume_role_policy: "{}".to_string(),
                attached_policies: vec![AttachedPolicy {
                    name: "ReadOnlyAccess".to_string(),
                    arn: "arn:aws:iam::aws:policy/ReadOnlyAccess".to_string(),
                }],
                inline_policies: vec![InlinePolicy {
                    name: "inline-policy".to_string(),
                    document: "{}".to_string(),
                }],
            }),
            launch_time: "2026-02-13".to_string(),
            tags: vec![("Name".to_string(), "web-a".to_string())],
            volumes: vec![],
            user_data: None,
        }
    }

    fn sample_network_detail() -> NetworkDetail {
        NetworkDetail {
            name: "main-vpc".to_string(),
            id: "vpc-1111aaaa".to_string(),
            cidr: "10.0.0.0/16".to_string(),
            state: "available".to_string(),
            subnets: vec![],
            igws: vec![],
            nats: vec![NatDetail {
                name: "nat-a".to_string(),
                id: "nat-1234".to_string(),
                state: "available".to_string(),
                connectivity_type: "public".to_string(),
                availability_mode: "regional".to_string(),
                auto_scaling_ips: "enabled".to_string(),
                auto_provision_zones: "enabled".to_string(),
                public_ip: "1.1.1.1".to_string(),
                allocation_id: "eipalloc-1234".to_string(),
                subnet_id: "subnet-a".to_string(),
                tags: vec![],
            }],
            route_tables: vec![RouteTableDetail {
                name: "rt-main".to_string(),
                id: "rtb-1234".to_string(),
                routes: vec![],
                associations: vec![],
            }],
            eips: vec![EipDetail {
                name: "eip-a".to_string(),
                public_ip: "1.1.1.1".to_string(),
                instance_id: String::new(),
                private_ip: String::new(),
            }],
            dns_support: true,
            dns_hostnames: true,
            tags: vec![("Name".to_string(), "main-vpc".to_string())],
        }
    }

    fn sample_sg_detail() -> SecurityGroupDetail {
        SecurityGroupDetail {
            name: "sg-web".to_string(),
            id: "sg-1234".to_string(),
            description: "web sg".to_string(),
            vpc_id: "vpc-1111aaaa".to_string(),
            inbound_rules: vec![SecurityRule {
                protocol: "TCP".to_string(),
                port_range: "80".to_string(),
                source_dest: "0.0.0.0/0".to_string(),
                description: "-".to_string(),
            }],
            outbound_rules: vec![],
        }
    }

    fn sample_lb_detail() -> LoadBalancerDetail {
        LoadBalancerDetail {
            name: "alb-main".to_string(),
            arn: "arn:aws:elasticloadbalancing:ap-northeast-2:123456789012:loadbalancer/app/alb-main/1234".to_string(),
            dns_name: "alb-main.example.com".to_string(),
            lb_type: "application".to_string(),
            scheme: "internet-facing".to_string(),
            vpc_id: "vpc-1111aaaa".to_string(),
            ip_address_type: "ipv4".to_string(),
            state: "active".to_string(),
            availability_zones: vec!["ap-northeast-2a".to_string()],
            security_groups: vec!["sg-1234".to_string()],
            listeners: vec![],
            target_groups: vec![TargetGroupInfo {
                name: "tg-main".to_string(),
                arn: "arn:aws:elasticloadbalancing:ap-northeast-2:123456789012:targetgroup/tg-main/1234".to_string(),
                protocol: "HTTP".to_string(),
                port: 80,
                target_type: "instance".to_string(),
                health_check_protocol: "HTTP".to_string(),
                health_check_path: "/health".to_string(),
                healthy_threshold: 2,
                unhealthy_threshold: 3,
                targets: vec![],
            }],
        }
    }

    fn sample_ecr_detail() -> EcrDetail {
        EcrDetail {
            name: "repo-a".to_string(),
            uri: "123456789012.dkr.ecr.ap-northeast-2.amazonaws.com/repo-a".to_string(),
            tag_mutability: "MUTABLE".to_string(),
            encryption_type: "AES256".to_string(),
            kms_key: None,
            created_at: "2026-02-13".to_string(),
            image_count: 2,
        }
    }

    fn sample_asg_detail() -> AsgDetail {
        AsgDetail {
            name: "asg-main".to_string(),
            arn: "arn:aws:autoscaling:ap-northeast-2:123456789012:autoScalingGroup:abcd:autoScalingGroupName/asg-main".to_string(),
            launch_template_name: Some("lt-main".to_string()),
            launch_template_id: Some("lt-1234".to_string()),
            launch_config_name: None,
            min_size: 1,
            max_size: 3,
            desired_capacity: 2,
            default_cooldown: 300,
            availability_zones: vec!["ap-northeast-2a".to_string()],
            target_group_arns: vec![],
            health_check_type: "EC2".to_string(),
            health_check_grace_period: 120,
            instances: vec![],
            created_time: "2026-02-13".to_string(),
            scaling_policies: vec![ScalingPolicy {
                name: "scale-out".to_string(),
                policy_type: "SimpleScaling".to_string(),
                adjustment_type: Some("ChangeInCapacity".to_string()),
                scaling_adjustment: Some(1),
                cooldown: Some(60),
            }],
            tags: vec![],
        }
    }

    #[test]
    fn loading_progress_reset_works() {
        let mut p = LoadingProgress {
            vpc_info: true,
            subnets: true,
            igws: true,
            nats: true,
            route_tables: true,
            eips: true,
            dns_attrs: true,
        };
        p.reset();
        assert!(!p.vpc_info);
        assert!(!p.subnets);
        assert!(!p.igws);
        assert!(!p.nats);
        assert!(!p.route_tables);
        assert!(!p.eips);
        assert!(!p.dns_attrs);
    }

    #[test]
    fn region_name_uses_language() {
        let r = Region {
            code: "us-east-1",
            name_ko: "버지니아",
            name_en: "N. Virginia",
        };
        assert_eq!(r.name(crate::i18n::Language::Korean), "버지니아");
        assert_eq!(r.name(crate::i18n::Language::English), "N. Virginia");
    }

    #[test]
    fn get_current_resource_type_and_info_priority() {
        let mut app = App::new();

        app.ec2_detail = Some(sample_ec2_detail());
        assert_eq!(app.get_current_resource_type(), Some(ResourceType::Ec2));
        assert_eq!(
            app.get_current_resource_info(),
            Some(("i-0123456789abcdef0".to_string(), "web-a".to_string()))
        );

        app.ec2_detail = None;
        app.network_detail = Some(sample_network_detail());
        assert_eq!(app.get_current_resource_type(), Some(ResourceType::Network));

        app.network_detail = None;
        app.sg_detail = Some(sample_sg_detail());
        assert_eq!(
            app.get_current_resource_type(),
            Some(ResourceType::SecurityGroup)
        );

        app.sg_detail = None;
        app.lb_detail = Some(sample_lb_detail());
        assert_eq!(
            app.get_current_resource_type(),
            Some(ResourceType::LoadBalancer)
        );

        app.lb_detail = None;
        app.ecr_detail = Some(sample_ecr_detail());
        assert_eq!(app.get_current_resource_type(), Some(ResourceType::Ecr));

        app.ecr_detail = None;
        app.asg_detail = Some(sample_asg_detail());
        assert_eq!(app.get_current_resource_type(), Some(ResourceType::Asg));
    }

    #[test]
    fn get_current_region_returns_selected_code() {
        let mut app = App::new();
        app.selected_region = REGIONS.len().saturating_sub(1);
        assert_eq!(app.get_current_region(), REGIONS[REGIONS.len() - 1].code);
    }

    #[test]
    fn move_resource_up_down_respects_bounds() {
        let mut app = App::new();
        let mut blueprint = Blueprint::new("bp".to_string());
        blueprint.add_resource(BlueprintResource {
            resource_type: ResourceType::Ec2,
            region: "ap-northeast-2".to_string(),
            resource_id: "i-1".to_string(),
            resource_name: "web-1".to_string(),
        });
        blueprint.add_resource(BlueprintResource {
            resource_type: ResourceType::Ec2,
            region: "ap-northeast-2".to_string(),
            resource_id: "i-2".to_string(),
            resource_name: "web-2".to_string(),
        });

        app.current_blueprint = Some(blueprint.clone());
        app.blueprint_store.add_blueprint(blueprint);
        app.selected_blueprint_index = 0;

        assert!(!app.move_resource_up(0));
        assert!(app.move_resource_down(0));
        assert!(!app.move_resource_down(99));
    }

    #[test]
    fn toggle_language_updates_settings_and_i18n() {
        let mut app = App::new();
        let before = app.settings.language;
        app.toggle_language();
        assert_ne!(app.settings.language, before);
        assert_eq!(app.i18n.lang, app.settings.language);
    }

    #[test]
    fn add_and_remove_resource_on_current_blueprint() {
        let mut app = App::new();
        let blueprint = Blueprint::new("bp".to_string());
        app.blueprint_store.add_blueprint(blueprint.clone());
        app.current_blueprint = Some(blueprint);
        app.selected_blueprint_index = 0;

        app.add_resource_to_current_blueprint(BlueprintResource {
            resource_type: ResourceType::Ecr,
            region: "ap-northeast-2".to_string(),
            resource_id: "repo-a".to_string(),
            resource_name: "repo-a".to_string(),
        });
        assert_eq!(
            app.current_blueprint
                .as_ref()
                .map(|bp| bp.resources.len())
                .unwrap_or_default(),
            1
        );

        app.remove_resource_from_current_blueprint(0);
        assert_eq!(
            app.current_blueprint
                .as_ref()
                .map(|bp| bp.resources.len())
                .unwrap_or_default(),
            0
        );
    }
}
