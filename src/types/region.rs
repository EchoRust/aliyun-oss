use std::fmt;
use std::str::FromStr;

use crate::error::{ErrorContext, OssError, OssErrorKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Region {
    CnHangzhou,
    CnShanghai,
    CnNanjing,
    CnFuzhou,
    CnWuhan,
    CnQingdao,
    CnBeijing,
    CnZhangjiakou,
    CnHuhehaote,
    CnWulanchabu,
    CnShenzhen,
    CnHeyuan,
    CnGuangzhou,
    CnChengdu,
    CnZhongwei,
    CnHongkong,
    RgChinaMainland,

    ApNortheast1,
    ApNortheast2,
    ApSoutheast1,
    ApSoutheast3,
    ApSoutheast5,
    ApSoutheast6,
    ApSoutheast7,

    EuCentral1,
    EuWest1,
    EuWest2,
    UsWest1,
    UsEast1,
    NaSouth1,

    MeEast1,
    MeCentral1,

    CnHangzhouFinance,
    CnShanghaiFinance1,
    CnShenzhenFinance1,
    CnBeijingFinance1,

    CnNorth2Gov1,

    Custom { endpoint: String, region_id: String },
}

impl Region {
    pub fn custom(endpoint: impl Into<String>, region_id: impl Into<String>) -> Self {
        Self::Custom {
            endpoint: endpoint.into(),
            region_id: region_id.into(),
        }
    }

    pub fn region_id(&self) -> &str {
        match self {
            Self::CnHangzhou => "cn-hangzhou",
            Self::CnShanghai => "cn-shanghai",
            Self::CnNanjing => "cn-nanjing",
            Self::CnFuzhou => "cn-fuzhou",
            Self::CnWuhan => "cn-wuhan-lr",
            Self::CnQingdao => "cn-qingdao",
            Self::CnBeijing => "cn-beijing",
            Self::CnZhangjiakou => "cn-zhangjiakou",
            Self::CnHuhehaote => "cn-huhehaote",
            Self::CnWulanchabu => "cn-wulanchabu",
            Self::CnShenzhen => "cn-shenzhen",
            Self::CnHeyuan => "cn-heyuan",
            Self::CnGuangzhou => "cn-guangzhou",
            Self::CnChengdu => "cn-chengdu",
            Self::CnZhongwei => "cn-zhongwei",
            Self::CnHongkong => "cn-hongkong",
            Self::RgChinaMainland => "rg-china-mainland",

            Self::ApNortheast1 => "ap-northeast-1",
            Self::ApNortheast2 => "ap-northeast-2",
            Self::ApSoutheast1 => "ap-southeast-1",
            Self::ApSoutheast3 => "ap-southeast-3",
            Self::ApSoutheast5 => "ap-southeast-5",
            Self::ApSoutheast6 => "ap-southeast-6",
            Self::ApSoutheast7 => "ap-southeast-7",

            Self::EuCentral1 => "eu-central-1",
            Self::EuWest1 => "eu-west-1",
            Self::EuWest2 => "eu-west-2",
            Self::UsWest1 => "us-west-1",
            Self::UsEast1 => "us-east-1",
            Self::NaSouth1 => "na-south-1",

            Self::MeEast1 => "me-east-1",
            Self::MeCentral1 => "me-central-1",

            Self::CnHangzhouFinance => "cn-hangzhou-finance",
            Self::CnShanghaiFinance1 => "cn-shanghai-finance-1",
            Self::CnShenzhenFinance1 => "cn-shenzhen-finance-1",
            Self::CnBeijingFinance1 => "cn-beijing-finance-1",

            Self::CnNorth2Gov1 => "cn-north-2-gov-1",

            Self::Custom { region_id, .. } => region_id,
        }
    }

    pub fn external_endpoint(&self) -> &str {
        match self {
            Self::CnHangzhou => "oss-cn-hangzhou.aliyuncs.com",
            Self::CnShanghai => "oss-cn-shanghai.aliyuncs.com",
            Self::CnNanjing => "oss-cn-nanjing.aliyuncs.com",
            Self::CnFuzhou => "oss-cn-fuzhou.aliyuncs.com",
            Self::CnWuhan => "oss-cn-wuhan-lr.aliyuncs.com",
            Self::CnQingdao => "oss-cn-qingdao.aliyuncs.com",
            Self::CnBeijing => "oss-cn-beijing.aliyuncs.com",
            Self::CnZhangjiakou => "oss-cn-zhangjiakou.aliyuncs.com",
            Self::CnHuhehaote => "oss-cn-huhehaote.aliyuncs.com",
            Self::CnWulanchabu => "oss-cn-wulanchabu.aliyuncs.com",
            Self::CnShenzhen => "oss-cn-shenzhen.aliyuncs.com",
            Self::CnHeyuan => "oss-cn-heyuan.aliyuncs.com",
            Self::CnGuangzhou => "oss-cn-guangzhou.aliyuncs.com",
            Self::CnChengdu => "oss-cn-chengdu.aliyuncs.com",
            Self::CnZhongwei => "oss-cn-zhongwei.aliyuncs.com",
            Self::CnHongkong => "oss-cn-hongkong.aliyuncs.com",
            Self::RgChinaMainland => "oss-rg-china-mainland.aliyuncs.com",

            Self::ApNortheast1 => "oss-ap-northeast-1.aliyuncs.com",
            Self::ApNortheast2 => "oss-ap-northeast-2.aliyuncs.com",
            Self::ApSoutheast1 => "oss-ap-southeast-1.aliyuncs.com",
            Self::ApSoutheast3 => "oss-ap-southeast-3.aliyuncs.com",
            Self::ApSoutheast5 => "oss-ap-southeast-5.aliyuncs.com",
            Self::ApSoutheast6 => "oss-ap-southeast-6.aliyuncs.com",
            Self::ApSoutheast7 => "oss-ap-southeast-7.aliyuncs.com",

            Self::EuCentral1 => "oss-eu-central-1.aliyuncs.com",
            Self::EuWest1 => "oss-eu-west-1.aliyuncs.com",
            Self::EuWest2 => "oss-eu-west-2.aliyuncs.com",
            Self::UsWest1 => "oss-us-west-1.aliyuncs.com",
            Self::UsEast1 => "oss-us-east-1.aliyuncs.com",
            Self::NaSouth1 => "oss-na-south-1.aliyuncs.com",

            Self::MeEast1 => "oss-me-east-1.aliyuncs.com",
            Self::MeCentral1 => "oss-me-central-1.aliyuncs.com",

            Self::CnHangzhouFinance => "oss-cn-hzfinance.aliyuncs.com",
            Self::CnShanghaiFinance1 => "oss-cn-shanghai-finance-1-pub.aliyuncs.com",
            Self::CnShenzhenFinance1 => "oss-cn-szfinance.aliyuncs.com",
            Self::CnBeijingFinance1 => "oss-cn-beijing-finance-1-pub.aliyuncs.com",

            Self::CnNorth2Gov1 => "oss-cn-north-2-gov-1.aliyuncs.com",

            Self::Custom { endpoint, .. } => endpoint,
        }
    }

    pub fn internal_endpoint(&self) -> &str {
        match self {
            Self::CnHangzhou => "oss-cn-hangzhou-internal.aliyuncs.com",
            Self::CnShanghai => "oss-cn-shanghai-internal.aliyuncs.com",
            Self::CnNanjing => "oss-cn-nanjing-internal.aliyuncs.com",
            Self::CnFuzhou => "oss-cn-fuzhou-internal.aliyuncs.com",
            Self::CnWuhan => "oss-cn-wuhan-lr-internal.aliyuncs.com",
            Self::CnQingdao => "oss-cn-qingdao-internal.aliyuncs.com",
            Self::CnBeijing => "oss-cn-beijing-internal.aliyuncs.com",
            Self::CnZhangjiakou => "oss-cn-zhangjiakou-internal.aliyuncs.com",
            Self::CnHuhehaote => "oss-cn-huhehaote-internal.aliyuncs.com",
            Self::CnWulanchabu => "oss-cn-wulanchabu-internal.aliyuncs.com",
            Self::CnShenzhen => "oss-cn-shenzhen-internal.aliyuncs.com",
            Self::CnHeyuan => "oss-cn-heyuan-internal.aliyuncs.com",
            Self::CnGuangzhou => "oss-cn-guangzhou-internal.aliyuncs.com",
            Self::CnChengdu => "oss-cn-chengdu-internal.aliyuncs.com",
            Self::CnZhongwei => "oss-cn-zhongwei-internal.aliyuncs.com",
            Self::CnHongkong => "oss-cn-hongkong-internal.aliyuncs.com",
            Self::RgChinaMainland => "oss-rg-china-mainland.aliyuncs.com",

            Self::ApNortheast1 => "oss-ap-northeast-1-internal.aliyuncs.com",
            Self::ApNortheast2 => "oss-ap-northeast-2-internal.aliyuncs.com",
            Self::ApSoutheast1 => "oss-ap-southeast-1-internal.aliyuncs.com",
            Self::ApSoutheast3 => "oss-ap-southeast-3-internal.aliyuncs.com",
            Self::ApSoutheast5 => "oss-ap-southeast-5-internal.aliyuncs.com",
            Self::ApSoutheast6 => "oss-ap-southeast-6-internal.aliyuncs.com",
            Self::ApSoutheast7 => "oss-ap-southeast-7-internal.aliyuncs.com",

            Self::EuCentral1 => "oss-eu-central-1-internal.aliyuncs.com",
            Self::EuWest1 => "oss-eu-west-1-internal.aliyuncs.com",
            Self::EuWest2 => "oss-eu-west-2-internal.aliyuncs.com",
            Self::UsWest1 => "oss-us-west-1-internal.aliyuncs.com",
            Self::UsEast1 => "oss-us-east-1-internal.aliyuncs.com",
            Self::NaSouth1 => "oss-na-south-1-internal.aliyuncs.com",

            Self::MeEast1 => "oss-me-east-1-internal.aliyuncs.com",
            Self::MeCentral1 => "oss-me-central-1-internal.aliyuncs.com",

            Self::CnHangzhouFinance => "oss-cn-hzfinance-internal.aliyuncs.com",
            Self::CnShanghaiFinance1 => "oss-cn-shanghai-finance-1-pub-internal.aliyuncs.com",
            Self::CnShenzhenFinance1 => "oss-cn-szfinance-internal.aliyuncs.com",
            Self::CnBeijingFinance1 => "oss-cn-beijing-finance-1-pub-internal.aliyuncs.com",

            Self::CnNorth2Gov1 => "oss-cn-north-2-gov-1-internal.aliyuncs.com",

            Self::Custom { endpoint, .. } => endpoint,
        }
    }

    pub fn dual_stack_endpoint(&self) -> Option<&str> {
        match self {
            Self::CnHangzhou => Some("cn-hangzhou.oss.aliyuncs.com"),
            Self::CnShanghai => Some("cn-shanghai.oss.aliyuncs.com"),
            Self::CnQingdao => Some("cn-qingdao.oss.aliyuncs.com"),
            Self::CnBeijing => Some("cn-beijing.oss.aliyuncs.com"),
            Self::CnZhangjiakou => Some("cn-zhangjiakou.oss.aliyuncs.com"),
            Self::CnHuhehaote => Some("cn-huhehaote.oss.aliyuncs.com"),
            Self::CnWulanchabu => Some("cn-wulanchabu.oss.aliyuncs.com"),
            Self::CnShenzhen => Some("cn-shenzhen.oss.aliyuncs.com"),
            Self::CnHeyuan => Some("cn-heyuan.oss.aliyuncs.com"),
            Self::CnGuangzhou => Some("cn-guangzhou.oss.aliyuncs.com"),
            Self::CnChengdu => Some("cn-chengdu.oss.aliyuncs.com"),
            Self::CnHongkong => Some("cn-hongkong.oss.aliyuncs.com"),
            Self::EuCentral1 => Some("eu-central-1.oss.aliyuncs.com"),
            Self::CnHangzhouFinance => Some("cn-hangzhou-finance.oss.aliyuncs.com"),
            Self::CnShanghaiFinance1 => Some("cn-shanghai-finance.oss.aliyuncs.com"),
            Self::CnShenzhenFinance1 => Some("cn-shenzhen-finance.oss.aliyuncs.com"),
            Self::CnNorth2Gov1 => Some("cn-north-2-gov-1.oss.aliyuncs.com"),
            _ => None,
        }
    }

    pub fn internal_vip_cidrs(&self) -> &[&str] {
        match self {
            Self::CnHangzhou => &[
                "100.118.28.0/24",
                "100.114.102.0/24",
                "100.98.170.0/24",
                "100.118.31.0/24",
            ],
            Self::CnShanghai => &[
                "100.98.35.0/24",
                "100.98.110.0/24",
                "100.98.169.0/24",
                "100.118.102.0/24",
            ],
            Self::CnNanjing => &["100.114.142.0/24"],
            Self::CnFuzhou => &["100.115.21.0/24"],
            Self::CnWuhan => &["100.115.89.0/24"],
            Self::CnQingdao => &[
                "100.115.173.0/24",
                "100.99.113.0/24",
                "100.99.114.0/24",
                "100.99.115.0/24",
            ],
            Self::CnBeijing => &[
                "100.118.58.0/24",
                "100.118.167.0/24",
                "100.118.170.0/24",
                "100.118.171.0/24",
                "100.118.172.0/24",
                "100.118.173.0/24",
            ],
            Self::CnZhangjiakou => &[
                "100.118.90.0/24",
                "100.98.159.0/24",
                "100.114.0.0/24",
                "100.114.1.0/24",
            ],
            Self::CnHuhehaote => &[
                "100.118.195.0/24",
                "100.99.110.0/24",
                "100.99.111.0/24",
                "100.99.112.0/24",
            ],
            Self::CnWulanchabu => &[
                "100.114.11.0/24",
                "100.114.12.0/24",
                "100.114.100.0/24",
                "100.118.214.0/24",
            ],
            Self::CnShenzhen => &[
                "100.118.78.0/24",
                "100.118.203.0/24",
                "100.118.204.0/24",
                "100.118.217.0/24",
            ],
            Self::CnHeyuan => &["100.98.83.0/24", "100.118.174.0/24"],
            Self::CnGuangzhou => &["100.115.33.0/24", "100.114.101.0/24"],
            Self::CnChengdu => &[
                "100.115.155.0/24",
                "100.99.107.0/24",
                "100.99.108.0/24",
                "100.99.109.0/24",
            ],
            Self::CnZhongwei => &["100.115.157.0/27"],
            Self::CnHongkong => &[
                "100.115.61.0/24",
                "100.99.103.0/24",
                "100.99.104.0/24",
                "100.99.106.0/24",
            ],

            Self::ApNortheast1 => &["100.114.211.0/24", "100.114.114.0/25"],
            Self::ApNortheast2 => &["100.99.119.0/24"],
            Self::ApSoutheast1 => &[
                "100.118.219.0/24",
                "100.99.213.0/24",
                "100.99.116.0/24",
                "100.99.117.0/24",
            ],
            Self::ApSoutheast3 => &[
                "100.118.165.0/24",
                "100.99.125.0/24",
                "100.99.130.0/24",
                "100.99.131.0/24",
            ],
            Self::ApSoutheast5 => &["100.114.98.0/24"],
            Self::ApSoutheast6 => &["100.115.16.0/24"],
            Self::ApSoutheast7 => &["100.98.249.0/24"],

            Self::EuCentral1 => &["100.115.154.0/24"],
            Self::EuWest1 => &["100.114.114.128/25"],
            Self::EuWest2 => &["100.115.159.0/27"],
            Self::UsWest1 => &["100.115.107.0/24"],
            Self::UsEast1 => &[
                "100.115.60.0/24",
                "100.99.100.0/24",
                "100.99.101.0/24",
                "100.99.102.0/24",
            ],
            Self::NaSouth1 => &["100.115.112.0/27"],

            Self::MeEast1 => &["100.99.235.0/24"],

            Self::CnHangzhouFinance => &[
                "100.103.4.95/32",
                "100.103.5.142/32",
                "100.103.5.143/32",
                "100.103.5.144/32",
                "100.115.6.0/24",
            ],
            Self::CnShanghaiFinance1 => &["100.100.36.24/32", "100.100.36.8/32"],
            Self::CnShenzhenFinance1 => &["100.112.15.0/24", "100.100.80.70/32"],
            Self::CnBeijingFinance1 => &["100.112.52.0/24"],

            _ => &[],
        }
    }

    pub fn acceleration_endpoint(&self) -> String {
        "oss-accelerate.aliyuncs.com".to_string()
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::CnHangzhou,
            Self::CnShanghai,
            Self::CnNanjing,
            Self::CnFuzhou,
            Self::CnWuhan,
            Self::CnQingdao,
            Self::CnBeijing,
            Self::CnZhangjiakou,
            Self::CnHuhehaote,
            Self::CnWulanchabu,
            Self::CnShenzhen,
            Self::CnHeyuan,
            Self::CnGuangzhou,
            Self::CnChengdu,
            Self::CnZhongwei,
            Self::CnHongkong,
            Self::RgChinaMainland,
            Self::ApNortheast1,
            Self::ApNortheast2,
            Self::ApSoutheast1,
            Self::ApSoutheast3,
            Self::ApSoutheast5,
            Self::ApSoutheast6,
            Self::ApSoutheast7,
            Self::EuCentral1,
            Self::EuWest1,
            Self::EuWest2,
            Self::UsWest1,
            Self::UsEast1,
            Self::NaSouth1,
            Self::MeEast1,
            Self::MeCentral1,
            Self::CnHangzhouFinance,
            Self::CnShanghaiFinance1,
            Self::CnShenzhenFinance1,
            Self::CnBeijingFinance1,
            Self::CnNorth2Gov1,
        ]
    }
}

impl FromStr for Region {
    type Err = OssError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cn-hangzhou" => Ok(Self::CnHangzhou),
            "cn-shanghai" => Ok(Self::CnShanghai),
            "cn-nanjing" => Ok(Self::CnNanjing),
            "cn-fuzhou" => Ok(Self::CnFuzhou),
            "cn-wuhan-lr" => Ok(Self::CnWuhan),
            "cn-qingdao" => Ok(Self::CnQingdao),
            "cn-beijing" => Ok(Self::CnBeijing),
            "cn-zhangjiakou" => Ok(Self::CnZhangjiakou),
            "cn-huhehaote" => Ok(Self::CnHuhehaote),
            "cn-wulanchabu" => Ok(Self::CnWulanchabu),
            "cn-shenzhen" => Ok(Self::CnShenzhen),
            "cn-heyuan" => Ok(Self::CnHeyuan),
            "cn-guangzhou" => Ok(Self::CnGuangzhou),
            "cn-chengdu" => Ok(Self::CnChengdu),
            "cn-zhongwei" => Ok(Self::CnZhongwei),
            "cn-hongkong" => Ok(Self::CnHongkong),
            "rg-china-mainland" => Ok(Self::RgChinaMainland),

            "ap-northeast-1" => Ok(Self::ApNortheast1),
            "ap-northeast-2" => Ok(Self::ApNortheast2),
            "ap-southeast-1" => Ok(Self::ApSoutheast1),
            "ap-southeast-3" => Ok(Self::ApSoutheast3),
            "ap-southeast-5" => Ok(Self::ApSoutheast5),
            "ap-southeast-6" => Ok(Self::ApSoutheast6),
            "ap-southeast-7" => Ok(Self::ApSoutheast7),

            "eu-central-1" => Ok(Self::EuCentral1),
            "eu-west-1" => Ok(Self::EuWest1),
            "eu-west-2" => Ok(Self::EuWest2),
            "us-west-1" => Ok(Self::UsWest1),
            "us-east-1" => Ok(Self::UsEast1),
            "na-south-1" => Ok(Self::NaSouth1),

            "me-east-1" => Ok(Self::MeEast1),
            "me-central-1" => Ok(Self::MeCentral1),

            "cn-hangzhou-finance" => Ok(Self::CnHangzhouFinance),
            "cn-shanghai-finance-1" => Ok(Self::CnShanghaiFinance1),
            "cn-shenzhen-finance-1" => Ok(Self::CnShenzhenFinance1),
            "cn-beijing-finance-1" => Ok(Self::CnBeijingFinance1),

            "cn-north-2-gov-1" => Ok(Self::CnNorth2Gov1),

            other => Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: Box::new(ErrorContext {
                    operation: Some(format!("parse Region from '{}'", other)),
                    ..Default::default()
                }),
                source: None,
            }),
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.region_id())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn region_cn_hangzhou_external_endpoint() {
        assert_eq!(
            Region::CnHangzhou.external_endpoint(),
            "oss-cn-hangzhou.aliyuncs.com"
        );
    }

    #[test]
    fn region_cn_hangzhou_internal_endpoint() {
        assert_eq!(
            Region::CnHangzhou.internal_endpoint(),
            "oss-cn-hangzhou-internal.aliyuncs.com"
        );
    }

    #[test]
    fn region_cn_hangzhou_region_id() {
        assert_eq!(Region::CnHangzhou.region_id(), "cn-hangzhou");
    }

    #[test]
    fn region_cn_huhehaote_region_id() {
        assert_eq!(Region::CnHuhehaote.region_id(), "cn-huhehaote");
    }

    #[test]
    fn all_regions_have_unique_external_endpoints() {
        let mut endpoints = HashSet::new();
        for region in Region::all() {
            assert!(
                endpoints.insert(region.external_endpoint().to_string()),
                "Duplicate endpoint for {:?}",
                region
            );
        }
    }

    #[test]
    fn custom_region_allows_arbitrary_endpoint() {
        let region = Region::custom("oss-custom.example.com", "custom-id");
        assert_eq!(region.external_endpoint(), "oss-custom.example.com");
        assert_eq!(region.region_id(), "custom-id");
        assert!(region.dual_stack_endpoint().is_none());
        assert!(region.internal_vip_cidrs().is_empty());
    }

    #[test]
    fn region_from_str_recognizes_known_regions() {
        assert_eq!("cn-hangzhou".parse::<Region>().unwrap(), Region::CnHangzhou);
        assert_eq!("cn-shanghai".parse::<Region>().unwrap(), Region::CnShanghai);
        assert_eq!(
            "cn-huhehaote".parse::<Region>().unwrap(),
            Region::CnHuhehaote
        );
        assert_eq!(
            "ap-southeast-1".parse::<Region>().unwrap(),
            Region::ApSoutheast1
        );
        assert_eq!(
            "eu-central-1".parse::<Region>().unwrap(),
            Region::EuCentral1
        );
        assert_eq!(
            "cn-hangzhou-finance".parse::<Region>().unwrap(),
            Region::CnHangzhouFinance
        );
        assert_eq!(
            "cn-north-2-gov-1".parse::<Region>().unwrap(),
            Region::CnNorth2Gov1
        );
    }

    #[test]
    fn region_from_str_unknown_returns_error() {
        assert!("unknown-region".parse::<Region>().is_err());
    }

    #[test]
    fn region_display_returns_region_id() {
        assert_eq!(Region::CnBeijing.to_string(), "cn-beijing");
        assert_eq!(Region::ApNortheast1.to_string(), "ap-northeast-1");
        assert_eq!(Region::EuCentral1.to_string(), "eu-central-1");
        assert_eq!(Region::CnZhongwei.to_string(), "cn-zhongwei");
    }

    #[test]
    fn region_dual_stack_supported() {
        assert_eq!(
            Region::CnHangzhou.dual_stack_endpoint(),
            Some("cn-hangzhou.oss.aliyuncs.com")
        );
        assert_eq!(
            Region::CnBeijing.dual_stack_endpoint(),
            Some("cn-beijing.oss.aliyuncs.com")
        );
        assert_eq!(
            Region::CnHongkong.dual_stack_endpoint(),
            Some("cn-hongkong.oss.aliyuncs.com")
        );
        assert_eq!(
            Region::EuCentral1.dual_stack_endpoint(),
            Some("eu-central-1.oss.aliyuncs.com")
        );
    }

    #[test]
    fn region_dual_stack_not_supported() {
        assert_eq!(Region::CnNanjing.dual_stack_endpoint(), None);
        assert_eq!(Region::CnWuhan.dual_stack_endpoint(), None);
        assert_eq!(Region::ApSoutheast1.dual_stack_endpoint(), None);
        assert_eq!(Region::UsWest1.dual_stack_endpoint(), None);
        assert_eq!(Region::RgChinaMainland.dual_stack_endpoint(), None);
    }

    #[test]
    fn region_internal_vip_cidrs() {
        assert_eq!(Region::CnHangzhou.internal_vip_cidrs().len(), 4);
        assert_eq!(Region::CnBeijing.internal_vip_cidrs().len(), 6);
        assert_eq!(Region::ApSoutheast1.internal_vip_cidrs().len(), 4);
        assert!(Region::RgChinaMainland.internal_vip_cidrs().is_empty());
        assert!(Region::MeCentral1.internal_vip_cidrs().is_empty());
    }

    #[test]
    fn region_finance_cloud_endpoints() {
        assert_eq!(
            Region::CnHangzhouFinance.external_endpoint(),
            "oss-cn-hzfinance.aliyuncs.com"
        );
        assert_eq!(
            Region::CnHangzhouFinance.internal_endpoint(),
            "oss-cn-hzfinance-internal.aliyuncs.com"
        );
        assert!(Region::CnHangzhouFinance.dual_stack_endpoint().is_some());
    }

    #[test]
    fn region_gov_cloud_endpoints() {
        assert_eq!(
            Region::CnNorth2Gov1.external_endpoint(),
            "oss-cn-north-2-gov-1.aliyuncs.com"
        );
        assert_eq!(
            Region::CnNorth2Gov1.dual_stack_endpoint(),
            Some("cn-north-2-gov-1.oss.aliyuncs.com")
        );
    }

    #[test]
    fn region_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Region>();
    }

    #[test]
    fn region_count() {
        assert_eq!(Region::all().len(), 37);
    }

    #[test]
    fn region_acceleration_endpoint() {
        assert_eq!(
            Region::CnHangzhou.acceleration_endpoint(),
            "oss-accelerate.aliyuncs.com"
        );
    }

    #[test]
    fn region_rg_china_mainland() {
        assert_eq!(Region::RgChinaMainland.region_id(), "rg-china-mainland");
        assert_eq!(
            Region::RgChinaMainland.external_endpoint(),
            "oss-rg-china-mainland.aliyuncs.com"
        );
        assert!(Region::RgChinaMainland.dual_stack_endpoint().is_none());
        assert!(Region::RgChinaMainland.internal_vip_cidrs().is_empty());
    }
}
