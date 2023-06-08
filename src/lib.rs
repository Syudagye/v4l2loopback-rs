use std::{
    fs::OpenOptions,
    os::fd::{IntoRawFd, RawFd},
};

use nix::{ioctl_read_bad, ioctl_readwrite_bad, ioctl_write_int_bad};

mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/v4l2loopback.rs"));
}

pub use ffi::V4L2LOOPBACK_VERSION_MAJOR;
pub use ffi::V4L2LOOPBACK_VERSION_MINOR;
pub use ffi::V4L2LOOPBACK_VERSION_BUGFIX;

impl Default for ffi::v4l2_loopback_config {
    fn default() -> Self {
        Self {
            output_nr: -1,
            unused: 0,
            card_label: [0; 32],
            min_width: 0,
            max_width: 0,
            min_height: 0,
            max_height: 0,
            max_buffers: 0,
            max_openers: 0,
            debug: 0,
            announce_all_caps: -1,
        }
    }
}

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

pub fn open_control_device() -> Result<RawFd, Box<dyn std::error::Error>> {
    Ok(OpenOptions::new()
        .read(true)
        .open(CONTROL_DEVICE)?
        .into_raw_fd())
}

pub fn create_device(device_num: Option<u32>) -> Result<i32, Box<dyn std::error::Error>> {
    let output_nr: i32 = device_num
        .map(i32::try_from)
        .map(Result::ok)
        .flatten()
        .unwrap_or(-1);
    let mut cfg = ffi::v4l2_loopback_config {
        output_nr,
        ..Default::default()
    };

    let fd = open_control_device()?;

    let dev = unsafe { v4l2loopback_ctl_add(fd, &mut cfg as *mut ffi::v4l2_loopback_config) }?;

    if cfg.output_nr.is_negative() {
        // TODO: Error handling
    }
    Ok(dev)
}

pub fn delete_device(device_num: u32) -> Result<(), Box<dyn std::error::Error>> {
    let fd = open_control_device()?;
    unsafe { v4l2loopback_ctl_remove(fd, device_num.try_into()?) }?;
    Ok(())
}

pub fn query_device(
    device_num: u32,
) -> Result<ffi::v4l2_loopback_config, Box<dyn std::error::Error>> {
    let mut cfg = ffi::v4l2_loopback_config::default();
    cfg.output_nr = device_num.try_into()?;

    let fd = open_control_device()?;

    unsafe { v4l2loopback_ctl_query(fd, &mut cfg as *mut ffi::v4l2_loopback_config) }?;

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{create_device, delete_device, query_device};

    #[test]
    fn device_no_num() {
        // Device creation
        let device_num = create_device(None).expect("Error when creating the device");
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
        println!("{}", next_num);

        // Device creation
        let device_num = create_device(Some(next_num)).expect("Error when creating the device");
        assert!(Path::new(&format!("/dev/video{}", device_num)).exists());

        // Device removal
        delete_device(device_num.try_into().unwrap()).expect("Error when removing device");
        assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
    }

    #[test]
    fn device_query_infos() {
        // Device creation
        let device_num = create_device(None).expect("Error when creating the device");
        assert!(Path::new(&format!("/dev/video{}", device_num)).exists());

        // Check informations
        let cfg =
            query_device(device_num.try_into().unwrap()).expect("Error when querying the device");
        assert_eq!(cfg.output_nr, device_num);

        // Device removal
        delete_device(device_num.try_into().unwrap()).expect("Error when removing device");
        assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
    }
}
