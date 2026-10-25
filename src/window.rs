use gpui::Window;

pub struct BackdropStatus {
    pub translucent: bool,
}
impl gpui::Global for BackdropStatus {}

pub fn panel_color(cx: &gpui::App) -> gpui::Rgba {
    if cx
        .try_global::<BackdropStatus>()
        .is_some_and(|state| state.translucent)
    {
        gpui::rgba(0x303840e8)
    } else {
        gpui::rgba(0x303840ff)
    }
}

pub fn configure_background(window: &Window, cx: &mut gpui::App) {
    use crate::services::config::{Backdrop, LoadedClientConfig};
    use gpui::WindowBackgroundAppearance;
    let mode = cx.global::<LoadedClientConfig>().config.appearance.backdrop;
    let translucent = match mode {
        Backdrop::Opaque => false,
        Backdrop::Transparent => {
            window.set_background_appearance(WindowBackgroundAppearance::Transparent);
            true
        }
        // Windows 10 ACCENT_ENABLE_ACRYLICBLURBEHIND can stall native move/size.
        // Keep it explicit rather than enabling it on every launch.
        Backdrop::Auto => false,
        Backdrop::Acrylic => {
            window.set_background_appearance(WindowBackgroundAppearance::Transparent);
            try_acrylic(window)
        }
    };
    if !translucent {
        window.set_background_appearance(WindowBackgroundAppearance::Opaque);
    }
    log::info!(
        "backdrop: requested={mode:?}, translucent={translucent}; opaque fallback={}",
        !translucent
    );
    cx.set_global(BackdropStatus { translucent });
}

#[cfg(not(target_os = "windows"))]
fn try_acrylic(_: &Window) -> bool {
    false
}

#[cfg(target_os = "windows")]
fn try_acrylic(window: &Window) -> bool {
    use windows_sys::Win32::{
        Graphics::Dwm::DwmIsCompositionEnabled,
        System::{
            LibraryLoader::{GetModuleHandleW, GetProcAddress},
            Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW},
        },
        UI::WindowsAndMessaging::{GetSystemMetrics, SM_REMOTESESSION},
    };
    #[repr(C)]
    struct Version {
        size: u32,
        major: u32,
        minor: u32,
        build: u32,
        platform: u32,
        service_pack: [u16; 128],
    }
    #[repr(C)]
    struct Accent {
        state: i32,
        flags: u32,
        color: u32,
        animation: u32,
    }
    #[repr(C)]
    struct Attribute {
        attribute: u32,
        data: *mut core::ffi::c_void,
        size: usize,
    }
    let Some(hwnd) = native_handle(window) else {
        return false;
    };
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    // SWCA is dynamically resolved and optional. No Windows 11 Mica dependency.
    unsafe {
        let mut composition = 0;
        if DwmIsCompositionEnabled(&mut composition) < 0
            || composition == 0
            || GetSystemMetrics(SM_REMOTESESSION) != 0
        {
            return false;
        }
        let mut transparency = 1_u32;
        let mut bytes = 4_u32;
        RegGetValueW(
            HKEY_CURRENT_USER,
            wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize").as_ptr(),
            wide("EnableTransparency").as_ptr(),
            RRF_RT_REG_DWORD,
            core::ptr::null_mut(),
            &mut transparency as *mut _ as *mut _,
            &mut bytes,
        );
        if transparency == 0 {
            return false;
        }
        let module = GetModuleHandleW(wide("ntdll.dll").as_ptr());
        let Some(raw_version) = GetProcAddress(module, c"RtlGetVersion".as_ptr() as *const u8)
        else {
            return false;
        };
        let version_fn: unsafe extern "system" fn(*mut Version) -> i32 =
            core::mem::transmute(raw_version);
        let mut version = Version {
            size: core::mem::size_of::<Version>() as u32,
            major: 0,
            minor: 0,
            build: 0,
            platform: 0,
            service_pack: [0; 128],
        };
        if version_fn(&mut version) < 0 || version.major < 10 || version.build < 17763 {
            return false;
        }
        let module = GetModuleHandleW(wide("user32.dll").as_ptr());
        let Some(raw) = GetProcAddress(
            module,
            c"SetWindowCompositionAttribute".as_ptr() as *const u8,
        ) else {
            return false;
        };
        let apply: unsafe extern "system" fn(
            windows_sys::Win32::Foundation::HWND,
            *mut Attribute,
        ) -> i32 = core::mem::transmute(raw);
        let mut accent = Accent {
            state: 4,
            flags: 0,
            color: 0xd92b2520,
            animation: 0,
        };
        let mut attribute = Attribute {
            attribute: 19,
            data: &mut accent as *mut _ as *mut _,
            size: core::mem::size_of::<Accent>(),
        };
        apply(hwnd, &mut attribute) != 0
    }
}

#[cfg(target_os = "windows")]
fn native_handle(window: &Window) -> Option<windows_sys::Win32::Foundation::HWND> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let window_handle = HasWindowHandle::window_handle(window).ok()?;
    let RawWindowHandle::Win32(window_handle) = window_handle.as_raw() else {
        return None;
    };

    Some(window_handle.hwnd.get() as windows_sys::Win32::Foundation::HWND)
}

#[cfg(target_os = "windows")]
pub fn remove_frame(window: &Window) {
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_USE_IMMERSIVE_DARK_MODE,
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
    };

    let Some(hwnd) = native_handle(window) else {
        return;
    };

    unsafe {
        use windows_sys::Win32::UI::{
            Shell::SetWindowSubclass,
            WindowsAndMessaging::{
                SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
                SetWindowPos,
            },
        };
        if SetWindowSubclass(hwnd, Some(frameless_subclass), 0x414d53, 0) != 0 {
            SetWindowPos(
                hwnd,
                core::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        // Optional dark non-client border on supporting Windows 10 builds.
        // Preserve all native sizing/caption styles and never make the window topmost.
        let dark = 1_i32;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            &dark as *const _ as *const _,
            core::mem::size_of_val(&dark) as u32,
        );
        // Keep GPUI's native resizable/maximizable styles so Windows Snap remains available,
        // and suppress only the visible one-pixel DWM border.
        let border_color = DWMWA_COLOR_NONE;
        let border_result = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR as u32,
            (&border_color as *const _) as *const core::ffi::c_void,
            core::mem::size_of_val(&border_color) as u32,
        );

        if border_result < 0 {
            log::debug!("Native DWM border preference unavailable: {border_result:#x}");
        }

        // The player also clips its GPUI content to rounded corners on Windows 10.
        // This optional DWM preference is supported on newer Windows versions.
        let corner_preference = DWMWCP_ROUND;
        let corner_result = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            (&corner_preference as *const _) as *const core::ffi::c_void,
            core::mem::size_of_val(&corner_preference) as u32,
        );
        if corner_result < 0 {
            log::debug!("Native DWM rounded corners unavailable: {corner_result:#x}");
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn remove_frame(_: &Window) {}

// Remove the residual native resize-frame pixels without removing sizing styles.
// GPUI still handles hit testing; maximized NCCALCSIZE stays with GPUI so that
// monitor work-area and taskbar calculations are preserved.
#[cfg(target_os = "windows")]
unsafe extern "system" fn frameless_subclass(
    hwnd: windows_sys::Win32::Foundation::HWND,
    message: u32,
    wparam: usize,
    lparam: isize,
    id: usize,
    _: usize,
) -> isize {
    use windows_sys::Win32::UI::{
        Shell::{DefSubclassProc, RemoveWindowSubclass},
        WindowsAndMessaging::{IsZoomed, WM_NCCALCSIZE, WM_NCDESTROY},
    };
    unsafe {
        if message == WM_NCCALCSIZE && wparam != 0 && IsZoomed(hwnd) == 0 {
            return 0;
        }
        if message == WM_NCDESTROY {
            RemoveWindowSubclass(hwnd, Some(frameless_subclass), id);
        }
        DefSubclassProc(hwnd, message, wparam, lparam)
    }
}
