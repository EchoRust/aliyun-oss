use std::fmt;
use std::str::FromStr;

use crate::error::{ErrorContext, OssError, OssErrorKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Region {
    CnHangzhou,
    CnShanghai,
    CnNanjing,
    CnFuzhou,
    CnQingdao,
    CnBeijing,
    CnZhangjiakou,
    CnHohhot,
    CnWulanchabu,
    CnShenzhen,
    CnHeyuan,
    CnGuangzhou,
    CnChengdu,
    CnHongkong,
    UsWest1,
    UsEast1,
    ApSoutheast1,
    ApSoutheast2,
    ApSoutheast3,
    ApSoutheast5,
    ApSoutheast6,
    ApSoutheast7,
    ApSouth1,
    ApNortheast1,
    ApNortheast2,
    EuCentral1,
    EuWest1,
    MeEast1,
    MeCentral1,
    Custom { endpoint: String, region_id: String },
}

impl Region {
    pub fn custom(endpoint: impl Into<String>, region_id: impl Into<String>) -> Self {
        Self::Custom {
            endpoint: endpoint.into(),
            region_id: region_id.into(),
        }
    }

    pub fn external_endpoint(&self) -> &str {
        match self {
            Self::CnHangzhou => "oss-cn-hangzhou.aliyuncs.com",
            Self::CnShanghai => "oss-cn-shanghai.aliyuncs.com",
            Self::CnNanjing => "oss-cn-nanjing.aliyuncs.com",
            Self::CnFuzhou => "oss-cn-fuzhou.aliyuncs.com",
            Self::CnQingdao => "oss-cn-qingdao.aliyuncs.com",
            Self::CnBeijing => "oss-cn-beijing.aliyuncs.com",
            Self::CnZhangjiakou => "oss-cn-zhangjiakou.aliyuncs.com",
            Self::CnHohhot => "oss-cn-hohhot.aliyuncs.com",
            Self::CnWulanchabu => "oss-cn-wulanchabu.aliyuncs.com",
            Self::CnShenzhen => "oss-cn-shenzhen.aliyuncs.com",
            Self::CnHeyuan => "oss-cn-heyuan.aliyuncs.com",
            Self::CnGuangzhou => "oss-cn-guangzhou.aliyuncs.com",
            Self::CnChengdu => "oss-cn-chengdu.aliyuncs.com",
            Self::CnHongkong => "oss-cn-hongkong.aliyuncs.com",
            Self::UsWest1 => "oss-us-west-1.aliyuncs.com",
            Self::UsEast1 => "oss-us-east-1.aliyuncs.com",
            Self::ApSoutheast1 => "oss-ap-southeast-1.aliyuncs.com",
            Self::ApSoutheast2 => "oss-ap-southeast-2.aliyuncs.com",
            Self::ApSoutheast3 => "oss-ap-southeast-3.aliyuncs.com",
            Self::ApSoutheast5 => "oss-ap-southeast-5.aliyuncs.com",
            Self::ApSoutheast6 => "oss-ap-southeast-6.aliyuncs.com",
            Self::ApSoutheast7 => "oss-ap-southeast-7.aliyuncs.com",
            Self::ApSouth1 => "oss-ap-south-1.aliyuncs.com",
            Self::ApNortheast1 => "oss-ap-northeast-1.aliyuncs.com",
            Self::ApNortheast2 => "oss-ap-northeast-2.aliyuncs.com",
            Self::EuCentral1 => "oss-eu-central-1.aliyuncs.com",
            Self::EuWest1 => "oss-eu-west-1.aliyuncs.com",
            Self::MeEast1 => "oss-me-east-1.aliyuncs.com",
            Self::MeCentral1 => "oss-me-central-1.aliyuncs.com",
            Self::Custom { endpoint, .. } => endpoint,
        }
    }

    pub fn internal_endpoint(&self) -> &str {
        match self {
            Self::CnHangzhou => "oss-cn-hangzhou-internal.aliyuncs.com",
            Self::CnShanghai => "oss-cn-shanghai-internal.aliyuncs.com",
            Self::CnNanjing => "oss-cn-nanjing-internal.aliyuncs.com",
            Self::CnFuzhou => "oss-cn-fuzhou-internal.aliyuncs.com",
            Self::CnQingdao => "oss-cn-qingdao-internal.aliyuncs.com",
            Self::CnBeijing => "oss-cn-beijing-internal.aliyuncs.com",
            Self::CnZhangjiakou => "oss-cn-zhangjiakou-internal.aliyuncs.com",
            Self::CnHohhot => "oss-cn-hohhot-internal.aliyuncs.com",
            Self::CnWulanchabu => "oss-cn-wulanchabu-internal.aliyuncs.com",
            Self::CnShenzhen => "oss-cn-shenzhen-internal.aliyuncs.com",
            Self::CnHeyuan => "oss-cn-heyuan-internal.aliyuncs.com",
            Self::CnGuangzhou => "oss-cn-guangzhou-internal.aliyuncs.com",
            Self::CnChengdu => "oss-cn-chengdu-internal.aliyuncs.com",
            Self::CnHongkong => "oss-cn-hongkong-internal.aliyuncs.com",
            Self::UsWest1 => "oss-us-west-1-internal.aliyuncs.com",
            Self::UsEast1 => "oss-us-east-1-internal.aliyuncs.com",
            Self::ApSoutheast1 => "oss-ap-southeast-1-internal.aliyuncs.com",
            Self::ApSoutheast2 => "oss-ap-southeast-2-internal.aliyuncs.com",
            Self::ApSoutheast3 => "oss-ap-southeast-3-internal.aliyuncs.com",
            Self::ApSoutheast5 => "oss-ap-southeast-5-internal.aliyuncs.com",
            Self::ApSoutheast6 => "oss-ap-southeast-6-internal.aliyuncs.com",
            Self::ApSoutheast7 => "oss-ap-southeast-7-internal.aliyuncs.com",
            Self::ApSouth1 => "oss-ap-south-1-internal.aliyuncs.com",
            Self::ApNortheast1 => "oss-ap-northeast-1-internal.aliyuncs.com",
            Self::ApNortheast2 => "oss-ap-northeast-2-internal.aliyuncs.com",
            Self::EuCentral1 => "oss-eu-central-1-internal.aliyuncs.com",
            Self::EuWest1 => "oss-eu-west-1-internal.aliyuncs.com",
            Self::MeEast1 => "oss-me-east-1-internal.aliyuncs.com",
            Self::MeCentral1 => "oss-me-central-1-internal.aliyuncs.com",
            Self::Custom { endpoint, .. } => endpoint,
        }
    }

    pub fn region_id(&self) -> &str {
        match self {
            Self::CnHangzhou => "cn-hangzhou",
            Self::CnShanghai => "cn-shanghai",
            Self::CnNanjing => "cn-nanjing",
            Self::CnFuzhou => "cn-fuzhou",
            Self::CnQingdao => "cn-qingdao",
            Self::CnBeijing => "cn-beijing",
            Self::CnZhangjiakou => "cn-zhangjiakou",
            Self::CnHohhot => "cn-hohhot",
            Self::CnWulanchabu => "cn-wulanchabu",
            Self::CnShenzhen => "cn-shenzhen",
            Self::CnHeyuan => "cn-heyuan",
            Self::CnGuangzhou => "cn-guangzhou",
            Self::CnChengdu => "cn-chengdu",
            Self::CnHongkong => "cn-hongkong",
            Self::UsWest1 => "us-west-1",
            Self::UsEast1 => "us-east-1",
            Self::ApSoutheast1 => "ap-southeast-1",
            Self::ApSoutheast2 => "ap-southeast-2",
            Self::ApSoutheast3 => "ap-southeast-3",
            Self::ApSoutheast5 => "ap-southeast-5",
            Self::ApSoutheast6 => "ap-southeast-6",
            Self::ApSoutheast7 => "ap-southeast-7",
            Self::ApSouth1 => "ap-south-1",
            Self::ApNortheast1 => "ap-northeast-1",
            Self::ApNortheast2 => "ap-northeast-2",
            Self::EuCentral1 => "eu-central-1",
            Self::EuWest1 => "eu-west-1",
            Self::MeEast1 => "me-east-1",
            Self::MeCentral1 => "me-central-1",
            Self::Custom { region_id, .. } => region_id,
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
            Self::CnQingdao,
            Self::CnBeijing,
            Self::CnZhangjiakou,
            Self::CnHohhot,
            Self::CnWulanchabu,
            Self::CnShenzhen,
            Self::CnHeyuan,
            Self::CnGuangzhou,
            Self::CnChengdu,
            Self::CnHongkong,
            Self::UsWest1,
            Self::UsEast1,
            Self::ApSoutheast1,
            Self::ApSoutheast2,
            Self::ApSoutheast3,
            Self::ApSoutheast5,
            Self::ApSoutheast6,
            Self::ApSoutheast7,
            Self::ApSouth1,
            Self::ApNortheast1,
            Self::ApNortheast2,
            Self::EuCentral1,
            Self::EuWest1,
            Self::MeEast1,
            Self::MeCentral1,
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
            "cn-qingdao" => Ok(Self::CnQingdao),
            "cn-beijing" => Ok(Self::CnBeijing),
            "cn-zhangjiakou" => Ok(Self::CnZhangjiakou),
            "cn-hohhot" => Ok(Self::CnHohhot),
            "cn-wulanchabu" => Ok(Self::CnWulanchabu),
            "cn-shenzhen" => Ok(Self::CnShenzhen),
            "cn-heyuan" => Ok(Self::CnHeyuan),
            "cn-guangzhou" => Ok(Self::CnGuangzhou),
            "cn-chengdu" => Ok(Self::CnChengdu),
            "cn-hongkong" => Ok(Self::CnHongkong),
            "us-west-1" => Ok(Self::UsWest1),
            "us-east-1" => Ok(Self::UsEast1),
            "ap-southeast-1" => Ok(Self::ApSoutheast1),
            "ap-southeast-2" => Ok(Self::ApSoutheast2),
            "ap-southeast-3" => Ok(Self::ApSoutheast3),
            "ap-southeast-5" => Ok(Self::ApSoutheast5),
            "ap-southeast-6" => Ok(Self::ApSoutheast6),
            "ap-southeast-7" => Ok(Self::ApSoutheast7),
            "ap-south-1" => Ok(Self::ApSouth1),
            "ap-northeast-1" => Ok(Self::ApNortheast1),
            "ap-northeast-2" => Ok(Self::ApNortheast2),
            "eu-central-1" => Ok(Self::EuCentral1),
            "eu-west-1" => Ok(Self::EuWest1),
            "me-east-1" => Ok(Self::MeEast1),
            "me-central-1" => Ok(Self::MeCentral1),
            other => Err(OssError {
                kind: OssErrorKind::ValidationError,
                context: ErrorContext {
                    operation: Some(format!("parse Region from '{}'", other)),
                    ..Default::default()
                },
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
    }

    #[test]
    fn region_from_str_recognizes_known_regions() {
        assert_eq!("cn-hangzhou".parse::<Region>().unwrap(), Region::CnHangzhou);
        assert_eq!("cn-shanghai".parse::<Region>().unwrap(), Region::CnShanghai);
        assert_eq!(
            "ap-southeast-1".parse::<Region>().unwrap(),
            Region::ApSoutheast1
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
    }

    #[test]
    fn region_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Region>();
    }

    #[test]
    fn region_count() {
        assert_eq!(Region::all().len(), 29);
    }

    #[test]
    fn region_acceleration_endpoint() {
        assert_eq!(
            Region::CnHangzhou.acceleration_endpoint(),
            "oss-accelerate.aliyuncs.com"
        );
    }
}
