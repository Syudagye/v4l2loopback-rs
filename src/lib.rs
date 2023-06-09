//! Safe binding to interact with [v4l2loopback].
//!
//! [v4l2loopback] allows your to manage virtual video devices on linux.
//! Application are able to pass video content to those devices, and this video feed will be
//! available as a camera.
//!
//! <div class="example-wrap" style="display:inline-block">
//! <pre class="compile_fail" style="white-space:normal;font:inherit;">
//!
//! **⚠️ Warning**:
//!
//! > This crate is based on an unreleased version of [v4l2loopback].
//! > To use this crate you will need to build and install [v4l2loopback] from source.
//! > (If you are on archlinux, you can install [v4l2loopback-dkms-git] from the aur)
//!
//! </pre>
//! </div>
//!
//! # Usage
//!
//! ```
//! use std::path::Path;
//! use v4l2loopback_rs::{add_device, delete_device, query_device, DeviceConfig};
//!
//! // Device configuration
//! // Here you declare informations about the camera device that will be created.
//! // It should be matching the content that you will want to pass through
//! let device_config = DeviceConfig {
//!     label: "Test Device".to_string(),
//!     min_width: 100,
//!     max_width: 4000,
//!     min_height: 100,
//!     max_height: 4000,
//!     max_buffers: 9,
//!     max_openers: 3,
//!     announce_all_caps: 1,
//! };
//! // Create a device
//! let device_num =
//!     add_device(None, device_config.clone()).expect("Error when creating the device");
//!
//! // Querying informations about a device
//! // This returns the matchin device's configuration
//! let cfg =
//!     query_device(device_num).expect("Error when querying the device");
//!
//! // When you are done with you processing, don't forget to delete the device, otherwise the
//! // device `/dev/videoN` will not be removed.
//! delete_device(device_num).expect("Error when removing device");
//! assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
//! ```
//!
//! [v4l2loopback]: https://github.com/umlaeute/v4l2loopback
//! [v4l2loopback-dkms-git]: https://aur.archlinux.org/packages/v4l2loopback-dkms-git

use std::{
    ffi::{CStr, CString, NulError},
    fs::OpenOptions,
    io::ErrorKind,
    os::fd::{IntoRawFd, RawFd},
    slice::from_raw_parts,
    str::Utf8Error,
};

use nix::{errno::Errno, ioctl_read_bad, ioctl_readwrite_bad, ioctl_write_int_bad};
use thiserror::Error;

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

/// Wrapper type describing a v4l2loopback device.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DeviceConfig {
    /// A nice name for you device.
    /// If empty, v4l2loopback will choose a generic name
    pub label: String,

    /// Allowed minimum frame witdh.
    /// Setting this value below 48 as no effets as it is the minimal accepted value.
    pub min_width: u32,
    /// Allowed maximum frame witdh.
    /// Setting this value above 8192 as no effets as it is the maximal accepted value.
    pub max_width: u32,
    /// Allowed minimum frame height.
    /// Setting this value below 32 as no effets as it is the minimal accepted value.
    pub min_height: u32,
    /// Allowed maximum frame height.
    /// Setting this value above 8192 as no effets as it is the maximal accepted value.
    pub max_height: u32,

    /// Number of buffers to allocate for the queue.
    /// If <=0, then a default value is picked by v4l2loopback.
    pub max_buffers: i32,

    /// How many consumers are allowed to open this device concurrently.
    /// If <=0, then a default value is picked by v4l2loopback.
    pub max_openers: i32,

    pub announce_all_caps: i32,
}

impl TryInto<ffi::v4l2_loopback_config> for DeviceConfig {
    type Error = NulError;

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
    type Error = Utf8Error;

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

/// Error generated when accessing the control device fails
///
/// The control device usually is `/dev/v4l2loopback`.
#[derive(Debug, Error)]
pub enum ControlDeviceError {
    /// You don't have permissions to open the control device.
    /// Your may require root permissions.
    #[error("You don't have the right permissions")]
    PermissionDenied,

    /// The control device couldn't be found.
    /// Verify if the kernel module is properly loaded.
    #[error("Can't find control device /dev/v4l2loopback, check if the kernel module is properly loaded")]
    NotFound,

    /// An error resulting from trying to access the control device.
    #[error("Error when opening the control device: {0}")]
    Other(Box<dyn std::error::Error>),
}

const CONTROL_DEVICE: &'static str = "/dev/v4l2loopback";

fn open_control_device() -> Result<RawFd, ControlDeviceError> {
    match OpenOptions::new().read(true).open(CONTROL_DEVICE) {
        Ok(f) => Ok(f.into_raw_fd()),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Err(ControlDeviceError::NotFound),
            ErrorKind::PermissionDenied => Err(ControlDeviceError::PermissionDenied),
            _ => Err(ControlDeviceError::Other(Box::new(e))),
        },
    }
}

/// Error which can occure when calling a function from this crate
#[derive(Debug, Error)]
pub enum Error {
    /// An error occured when accessing the control device.
    /// See [`ControlDeviceError`] for more details
    #[error("Couldn't open control device: {0}")]
    ControlDevice(#[from] ControlDeviceError),

    /// An error resulting from an ioctl function call
    #[error("Error returned from ioctl: {0}")]
    Ioctl(#[from] Errno),

    /// Unable to create a device
    #[error("Failed to create device")]
    DeviceCreationFailed,

    /// Couldn't find the specified device
    #[error("Device /dev/video{0} not found")]
    DeviceNotFound(u32),

    /// Unable to properly convert the label name.
    ///
    /// The label need to comply to the C string format, which means it must not contain null
    /// bytes in it. It also need to contain at most 32 characters.
    /// This error can also be returned if the internal label of a device is wrongly formatted, in
    /// this case, this issue should be forwarded to the v4l2loopback project.
    #[error("Failed to convert label name. The label string must not contain null bytes, and it's legth must not exceed 32.")]
    LabelConversionError(Box<dyn std::error::Error>),

    /// Any other error
    #[error(transparent)]
    Other(Box<dyn std::error::Error>),
}

/// Create a new v4l2loopback device.
///
/// If you pass [`None`] to `num`, the device will be created using the next available device
/// number.
///
/// This function returns a result containing the device number is it is [`Ok`], and one of the
/// following error if it is [`Err`].
///
/// # Errors
///
/// This function will return the following errors:
/// - [`LabelConversionError`] if the label given in `config` contains null bytes.
/// - [`ControlDevice`] if it is unable to open the control device
/// - [`Ioctl`] if the underlying ioctl call fails
/// - [`DeviceCreationFailed`] if v4l2loopback was unable to create a device. This generally
/// happens when you specify an explicit number in `num`.
///
/// [`LabelConversionError`]: Error::LabelConversionError
/// [`ControlDevice`]: Error::ControlDevice
/// [`Ioctl`]: Error::Ioctl
/// [`DeviceCreationFailed`]: Error::DeviceCreationFailed
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use v4l2loopback_rs::{add_device, delete_device, DeviceConfig};
///
/// // We create the device without specifying a number
/// let device_num = add_device(None, DeviceConfig::default()).expect("Error when creating the device");
/// assert!(Path::new(&format!("/dev/video{}", device_num)).exists());
///
/// // Don't forget to delete it when you don't need it anymore
/// delete_device(device_num.try_into().unwrap()).expect("Error when removing device");
/// assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
/// ```
pub fn add_device(num: Option<u32>, config: DeviceConfig) -> Result<u32, Error> {
    let mut cfg: ffi::v4l2_loopback_config = match config.try_into() {
        Ok(cfg) => cfg,
        Err(e) => return Err(Error::LabelConversionError(Box::new(e))),
    };
    cfg.output_nr = num
        .map(i32::try_from)
        .map(Result::ok)
        .flatten()
        .unwrap_or(-1);

    let fd = open_control_device()?;

    let dev = unsafe { v4l2loopback_ctl_add(fd, &mut cfg as *mut ffi::v4l2_loopback_config) }?;

    if dev.is_negative() {
        return Err(Error::DeviceCreationFailed);
    }

    Ok(dev as u32)
}

/// Delete a v4l2loopback device.
///
/// Given the device number, this function will attempt to delete thev4l2loopback device.
///
/// # Errors
///
/// This function will return the following errors:
/// - [`ControlDevice`] if it is unable to open the control device
/// - [`Ioctl`] if the underlying ioctl call fails
/// - [`DeviceNotFound`] if the specified device is not recognized by v4l2loopback.
/// - [`Other`] for other errors
///
/// [`ControlDevice`]: Error::ControlDevice
/// [`Ioctl`]: Error::Ioctl
/// [`DeviceNotFound`]: Error::DeviceNotFound
/// [`Other`]: Error::Other
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use v4l2loopback_rs::{add_device, delete_device, DeviceConfig};
///
/// // You created a device earlier...
/// let device_num = add_device(None, DeviceConfig::default()).expect("Error when creating the device");
/// assert!(Path::new(&format!("/dev/video{}", device_num)).exists());
///
/// // ... Some fancy processing is being done ...
///
/// // ...And now you want to delete it.
/// delete_device(device_num.try_into().unwrap()).expect("Error when removing device");
/// assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
/// ```
pub fn delete_device(device_num: u32) -> Result<(), Error> {
    let fd = open_control_device()?;

    let converted_num = match device_num.try_into() {
        Ok(n) => n,
        Err(e) => return Err(Error::Other(Box::new(e))),
    };

    let res = unsafe { v4l2loopback_ctl_remove(fd, converted_num) }?;

    if res.is_negative() {
        return Err(Error::DeviceNotFound(device_num));
    }

    Ok(())
}

/// Queries the configuration for a specified device.
///
/// Given the device number, this function will fetch the corresponding device configuration
///
/// This function returns a result containing a [`DeviceConfig`] is it is [`Ok`], and one of the
/// following error if it is [`Err`].
///
/// # Errors
///
/// This function will return the following errors:
/// - [`ControlDevice`] if it is unable to open the control device
/// - [`Ioctl`] if the underlying ioctl call fails
/// - [`DeviceNotFound`] if the specified device is not recognized by v4l2loopback.
/// - [`LabelConversionError`] if the label returned by v4l2loopback contains null bytes.
/// - [`Other`] for other errors
///
/// [`ControlDevice`]: Error::ControlDevice
/// [`Ioctl`]: Error::Ioctl
/// [`DeviceNotFound`]: Error::DeviceNotFound
/// [`LabelConversionError`]: Error::LabelConversionError
/// [`Other`]: Error::Other
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use v4l2loopback_rs::{add_device, delete_device, query_device, DeviceConfig};
///
/// // We specify our desired config
/// let device_config = DeviceConfig {
///     label: "Test Device".to_string(),
///     min_width: 100,
///     max_width: 4000,
///     min_height: 100,
///     max_height: 4000,
///     max_buffers: 9,
///     max_openers: 3,
///     announce_all_caps: 1,
/// };
/// // Device creation
/// let device_num =
///     add_device(None, device_config.clone()).expect("Error when creating the device");
/// assert!(Path::new(&format!("/dev/video{}", device_num)).exists());
///
/// // Querying the informations
/// let cfg =
///     query_device(device_num).expect("Error when querying the device");
/// assert_eq!(cfg, device_config);
///
/// // Don't forget to remove the device !
/// delete_device(device_num).expect("Error when removing device");
/// assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
/// ```
pub fn query_device(device_num: u32) -> Result<DeviceConfig, Error> {
    let mut cfg = ffi::v4l2_loopback_config::default();
    cfg.output_nr = match device_num.try_into() {
        Ok(n) => n,
        Err(e) => return Err(Error::Other(Box::new(e))),
    };

    let fd = open_control_device()?;

    let res = unsafe { v4l2loopback_ctl_query(fd, &mut cfg as *mut ffi::v4l2_loopback_config) }?;

    if res.is_negative() {
        return Err(Error::DeviceNotFound(device_num));
    }

    let device_config = match DeviceConfig::try_from(cfg) {
        Ok(cfg) => cfg,
        Err(e) => return Err(Error::LabelConversionError(Box::new(e))),
    };

    Ok(device_config)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{add_device, delete_device};

    #[test]
    fn device_with_num() {
        // Getting the next unused device num
        let mut next_num = 0;
        while Path::new(&format!("/dev/video{}", next_num)).exists() {
            next_num += 1;
        }

        // Device creation
        let device_num =
            add_device(Some(next_num), Default::default()).expect("Error when creating the device");
        assert!(Path::new(&format!("/dev/video{}", device_num)).exists());

        // Device removal
        delete_device(device_num).expect("Error when removing device");
        assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
    }

    #[test]
    fn device_with_used_num() {
        let create_device_0 = !Path::new("/dev/video0").exists();
        if create_device_0 {
            add_device(Some(0), Default::default()).expect("Error when creating the device");
        }
        assert!(Path::new("/dev/video0").exists());

        // Let's try to create an already existing device
        let res = add_device(Some(0), Default::default());
        assert!(res.is_err());

        if create_device_0 {
            delete_device(0).expect("Error when removing device");
            assert!(!Path::new("/dev/video0").exists());
        }
    }
}
