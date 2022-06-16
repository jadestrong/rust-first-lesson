mod abi;
pub use abi::*;
use base64::{encode_config, URL_SAFE_NO_PAD, decode_config};
use photon_rs::transform::SamplingFilter;
use prost::Message;

use self::resize::SampleFilter;

/// 给 ImageSpec 结构体设置一个 new 方法
impl ImageSpec {
    pub fn new(specs: Vec<Spec>) -> Self {
        Self { specs }
    }
}

/// 从 ImageSpec 转换为 String
impl From<&ImageSpec> for String {
    fn from(image_sepc: &ImageSpec) -> Self {
        let data = image_sepc.encode_to_vec();
        encode_config(data, URL_SAFE_NO_PAD)
    }
}

/// 从 str 转换为 ImageSpec
impl TryFrom<&str> for ImageSpec {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let data = decode_config(value, URL_SAFE_NO_PAD)?;
        Ok(ImageSpec::decode(&data[..])?)
    }
}

/// 给 filter mod 下的 Filter 实现方法，将其转换成字符串类型
impl filter::Filter {
    pub fn to_str(&self) -> Option<&'static str> {
        match self {
            filter::Filter::Unspecified => None,
            filter::Filter::Oceanic => Some("oceanic"),
            filter::Filter::Islands => Some("islands"),
            filter::Filter::Marine => Some("marine"),
        }
    }
}

impl From<resize::SampleFilter> for SamplingFilter {
    fn from(v: resize::SampleFilter) -> Self {
        match v {
            SampleFilter::Undefined => SamplingFilter::Nearest,
            SampleFilter::Nearest => SamplingFilter::Nearest,
            SampleFilter::Triangle => SamplingFilter::Triangle,
            SampleFilter::CatmullRom => SamplingFilter::CatmullRom,
            SampleFilter::Gaussian => SamplingFilter::Gaussian,
            SampleFilter::Lanczos3 => SamplingFilter::Lanczos3,
        }
    }
}

/// 给 Spec 添加了几个方法
impl Spec {
    pub fn new_resize_seam_crave(width: u32, height: u32) -> Self {
        Self { data: Some(spec::Data::Resize(Resize {
            width,
            height,
            rtype: resize::ResizeType::SeamCarve as i32,
            filter: resize::SampleFilter::Undefined as i32,
        })) }
    }

    pub fn new_resize(width: u32, height: u32, filter: resize::SampleFilter) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                rtype: resize::ResizeType::Normal as i32,
                filter: filter as i32,
            })),
        }
    }

    pub fn new_filter(filter: filter::Filter) -> Self {
        Self { data: Some(spec::Data::Filter(Filter {
            filter: filter as i32,
        })) }
    }

    pub fn new_watermark(x: u32, y: u32) -> Self {
        Self {
            data: Some(spec::Data::Watermark(Watermark { x, y })),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use super::*;

    #[test]
    fn encoded_spec_could_be_decoded() {
        let spec1 = Spec::new_resize(600, 600, resize::SampleFilter::CatmullRom);
        let spec2 = Spec::new_filter(filter::Filter::Marine);
        let image_spec = ImageSpec::new(vec![spec1, spec2]);
        let s: String = image_spec.borrow().into();
        assert_eq!(image_spec, s.as_str().try_into().unwrap());
    }
}
