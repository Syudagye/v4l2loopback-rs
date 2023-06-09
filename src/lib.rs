use std::{
    ffi::{CStr, CString},
    fs::OpenOptions,
    os::fd::{IntoRawFd, RawFd},
    slice::from_raw_parts,
};

use nix::{ioctl_read_bad, ioctl_readwrite_bad, ioctl_write_int_bad};

mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/v4l2loopback.rs"));

    impl Default for v4l2_loopback_config {
        fn default() -> Self {
            Self {
                output_nr: -1,
                card_label: [0; 32],
                min_width: 0,
                max_width: 0,
                min_height: 0,
                max_height: 0,
                max_buffers: 0,
                max_openers: 0,
                announce_all_caps: 0,

                unused: 0,
                debug: 0,
            }
        }
    }
}

pub use ffi::V4L2LOOPBACK_VERSION_BUGFIX;
pub use ffi::V4L2LOOPBACK_VERSION_MAJOR;
pub use ffi::V4L2LOOPBACK_VERSION_MINOR;

ioctl_readwrite_bad!(
    v4l2loopback_ctl_add,
    ffi::V4L2LOOPBACK_CTL_ADD,
    ffi::v4l2_loopback_config
);
ioctl_write_int_bad!(v4l2loopback_ctl_remove, ffi::V4L2LOOPBACK_CTL_REMOVE);
ioctl_read_bad!(
    v4l2loopback_ctl_query,
    ffi::V4L2LOOPBACK_CTL_QUERY,
    ffi::v4l2_loopback_config
);

const CONTROL_DEVICE: &'static str = "/dev/v4l2loopback";

fn open_control_device() -> Result<RawFd, Box<dyn std::error::Error>> {
    Ok(OpenOptions::new()
        .read(true)
        .open(CONTROL_DEVICE)?
        .into_raw_fd())
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DeviceConfig {
    pub label: String,
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
    pub max_buffers: i32,
    pub max_openers: i32,
    pub announce_all_caps: i32,
}

impl TryInto<ffi::v4l2_loopback_config> for DeviceConfig {
    type Error = Box<dyn std::error::Error>;

    fn try_into(self) -> Result<ffi::v4l2_loopback_config, Self::Error> {
        let mut cfg = ffi::v4l2_loopback_config::default();

        let mut slice: [i8; 32] = [0; 32];
        unsafe { from_raw_parts(CString::new(self.label)?.as_ptr(), 32) }
            .into_iter()
            .enumerate()
            .for_each(|(i, v)| slice[i] = *v);
        cfg.card_label = slice;

        cfg.min_width = self.min_width;
        cfg.max_width = self.max_width;
        cfg.min_height = self.min_height;
        cfg.max_height = self.max_height;
        cfg.max_buffers = self.max_buffers;
        cfg.max_openers = self.max_openers;
        cfg.announce_all_caps = self.announce_all_caps;

        Ok(cfg)
    }
}

impl TryFrom<ffi::v4l2_loopback_config> for DeviceConfig {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: ffi::v4l2_loopback_config) -> Result<Self, Self::Error> {
        let ffi::v4l2_loopback_config {
            output_nr: _,
            unused: _,
            card_label,
            min_width,
            max_width,
            min_height,
            max_height,
            max_buffers,
            max_openers,
            debug: _,
            announce_all_caps,
        } = value;

        let label = unsafe { CStr::from_ptr(card_label.as_ptr()) }
            .to_str()?
            .to_string();

        Ok(Self {
            label,
            min_width,
            max_width,
            min_height,
            max_height,
            max_buffers,
            max_openers,
            announce_all_caps,
        })
    }
}

pub fn create_device(
    num: Option<u32>,
    config: DeviceConfig,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut cfg: ffi::v4l2_loopback_config = config.try_into()?;
    cfg.output_nr = num
        .map(i32::try_from)
        .map(Result::ok)
        .flatten()
        .unwrap_or(-1);

    let fd = open_control_device()?;

    let dev = unsafe { v4l2loopback_ctl_add(fd, &mut cfg as *mut ffi::v4l2_loopback_config) }?;

    if cfg.output_nr.is_negative() {
        // TODO: Error handling
    }
    Ok(dev as u32)
}

pub fn delete_device(device_num: u32) -> Result<(), Box<dyn std::error::Error>> {
    let fd = open_control_device()?;
    println!("Removing {}", device_num);
    unsafe { v4l2loopback_ctl_remove(fd, device_num.try_into()?) }?;
    Ok(())
}

pub fn query_device(device_num: u32) -> Result<DeviceConfig, Box<dyn std::error::Error>> {
    let mut cfg = ffi::v4l2_loopback_config::default();
    cfg.output_nr = device_num.try_into()?;

    let fd = open_control_device()?;

    unsafe { v4l2loopback_ctl_query(fd, &mut cfg as *mut ffi::v4l2_loopback_config) }?;

    let device_config = DeviceConfig::try_from(cfg)?;

    Ok(device_config)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{create_device, delete_device, query_device, DeviceConfig};

    #[test]
    fn device_no_num() {
        // Device creation
        let device_num =
            create_device(None, Default::default()).expect("Error when creating the device");
        assert!(Path::new(&format!("/dev/video{}", device_num)).exists());

        // Device removal
        delete_device(device_num.try_into().unwrap()).expect("Error when removing device");
        assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
    }

    #[test]
    fn device_with_num() {
        // Getting the next unused device num
        let mut next_num = 0;
        while Path::new(&format!("/dev/video{}", next_num)).exists() {
            next_num += 1;
        }

        // Device creation
        let device_num = create_device(Some(next_num), Default::default())
            .expect("Error when creating the device");
        assert!(Path::new(&format!("/dev/video{}", device_num)).exists());

        // Device removal
        delete_device(device_num.try_into().unwrap()).expect("Error when removing device");
        assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
    }

    #[test]
    fn device_query_infos() {
        // Device creation
        // If values a too low, they will be clamped
        // So we use value just above/below the limits
        let device_config = DeviceConfig {
            label: "Test thing".to_string(),
            min_width: 50,
            max_width: 8100,
            min_height: 40,
            max_height: 8100,
            max_buffers: 9,
            max_openers: 9,
            announce_all_caps: 1,
        };
        let device_num =
            create_device(None, device_config.clone()).expect("Error when creating the device");
        assert!(Path::new(&format!("/dev/video{}", device_num)).exists());

        // Check informations
        let cfg =
            query_device(device_num.try_into().unwrap()).expect("Error when querying the device");
        assert_eq!(cfg, device_config);

        // Device removal
        delete_device(device_num.try_into().unwrap()).expect("Error when removing device");
        assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
    }
}
