use smithay::{
    backend::{
        allocator::gbm::GbmDevice,
        drm::{DrmDevice, DrmDeviceFd, DrmError, DrmNode},
        egl::{display::EGLDisplay, EGLDevice, Error as EglErr},
        session::{libseat::Error as LibseatErr, Session},
    },
    reexports::nix::fcntl::OFlag,
    utils::DeviceFd,
};

use std::{os::fd::FromRawFd, path::Path};

#[allow(dead_code)]
struct Device {
    drm: DrmDevice,
    gbm: GbmDevice<DrmDeviceFd>,
    device_node: DrmNode,
    render_node: DrmNode,
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
enum DeviceError {
    #[error("Failed to open device with session: {0}")]
    OpenDevFile(LibseatErr),
    #[error("Unable to add drm device: {0}")]
    NewDevice(DrmError),
    #[error("Unable to add gbm device: {0}")]
    GbmDevice(std::io::Error),
    #[error("Failure to create egl display: {0}")]
    Egl(EglErr),
}

#[allow(unused)]
impl Device {
    pub fn new<S: Session>(node: DrmNode, path: &Path, session: &mut S) -> Result<Self, DeviceError>
    where
        S::Error: Into<LibseatErr>,
    {
        let fd = session
            .open(
                path,
                OFlag::O_RDWR | OFlag::O_CLOEXEC | OFlag::O_NOCTTY | OFlag::O_NONBLOCK,
            )
            .map_err(|err| DeviceError::OpenDevFile(err.into()))?;

        let fd = unsafe { DeviceFd::from_raw_fd(fd) };
        let drm_fd = DrmDeviceFd::new(fd);

        let (drm, notifier) =
            DrmDevice::new(drm_fd, true).map_err(|err| DeviceError::NewDevice(err))?;
        let gbm =
            GbmDevice::new(drm.device_fd().clone()).map_err(|err| DeviceError::GbmDevice(err))?;

        let render_node =
            EGLDevice::device_for_display(&EGLDisplay::new(gbm.clone()).map_err(DeviceError::Egl)?)
                .map_err(DeviceError::Egl)
                .ok()
                .and_then(|egl| egl.try_get_render_node().map_err(DeviceError::Egl).ok()?)
                .unwrap_or(node);

        Ok(Device {
            drm,
            gbm,
            device_node: node,
            render_node,
        })
    }
}
