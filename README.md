# v4l2loopback-rs

Safe binding to interact with [v4l2loopback].

[v4l2loopback] allows your to manage virtual video devices on linux.
Application are able to pass video content to those devices, and this video feed will be
available as a camera.

> ### **⚠️ Warning**:
> This crate is based on an unreleased version of [v4l2loopback].
> To use this crate you will need to build and install [v4l2loopback] from source.
> (If you are on archlinux, you can install [v4l2loopback-dkms-git] from the aur)

## Usage

Keep in mind that you need the permission to open `/dev/v4l2loopback`.
So when when using this crate in your application, be sure that it is running as root
before calling functions in this crate.

To avoid having to run cargo as root during developpement, you can change the permissions 
of the control device, (See [Building and Testing](#building-and-testing))

```rust
use std::path::Path;
use v4l2loopback_rs::{add_device, delete_device, query_device, DeviceConfig};

// Device configuration
// Here you declare informations about the camera device that will be created.
// It should be matching the content that you will want to pass through
let device_config = DeviceConfig {
    label: "Test Device".to_string(),
    min_width: 100,
    max_width: 4000,
    min_height: 100,
    max_height: 4000,
    max_buffers: 9,
    max_openers: 3,
    announce_all_caps: 1,
};
// Create a device
let device_num =
    add_device(None, device_config.clone()).expect("Error when creating the device");

// Querying informations about a device
// This returns the matchin device's configuration
let cfg =
    query_device(device_num).expect("Error when querying the device");

// When you are done with you processing, don't forget to delete the device, otherwise the
// device `/dev/videoN` will not be removed.
delete_device(device_num).expect("Error when removing device");
assert!(!Path::new(&format!("/dev/video{}", device_num)).exists());
```

[v4l2loopback]: https://github.com/umlaeute/v4l2loopback
[v4l2loopback-dkms-git]: https://aur.archlinux.org/packages/v4l2loopback-dkms-git

## Building and Testing

To be able to test this crate, you will need to clone this repository and it's submodules,
since for generating bindings it needs `v4l2loopback/v4l2loopback.h`.

For executing tests, you need to ensure you can open `/dev/v4l2loopback`.
To do so, you can change the permissions of `/dev/v4l2loopback`:
```bash
# Allow anyone to read the control device
sudo chmod o+r /dev/v4l2loopback
```
