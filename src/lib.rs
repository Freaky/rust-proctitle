#[cfg(any(
    target_os = "freebsd",
    target_os = "hardenedbsd",
    target_os = "dragonflybsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod imp {
    use std::ffi::CString;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    pub fn set_title<T: AsRef<OsStr>>(title: T) {
        if let Ok(title) = CString::new(title.as_ref().to_owned().as_bytes()) {
            unsafe {
                setproctitle(b"-%s\0".as_ptr(), title.as_ptr());
            }
        }
    }

    #[link(name = "c")]
    extern "C" {
        fn setproctitle(fmt: *const u8, ...);
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use libc;
    use std::ffi::CString;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    pub fn set_title<T: AsRef<OsStr>>(title: T) {
        if let Ok(title) = CString::new(title.as_ref().to_owned().as_bytes()) {
            unsafe { libc::prctl(libc::PR_SET_NAME, title.as_ptr(), 0, 0, 0) };
        }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::Mutex;

    use lazy_static::lazy_static;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::synchapi::CreateEventW;
    use winapi::um::wincon::SetConsoleTitleW;
    use winapi::um::winnt::HANDLE;

    struct Handle(HANDLE);
    unsafe impl Send for Handle {}

    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    lazy_static! {
        static ref EVENT_HANDLE: Mutex<Option<Handle>> = Mutex::new(None);
    }

    // Windows doesn't appear to have a userspace mechanism to name the current
    // process.
    //
    // Try to set a console title, and in case we're not attached to one,
    // follow PostgreSQL's lead and create a named event handle that can be
    // found in Process Explorer, Process Hacker, etc.
    pub fn set_title<T: AsRef<OsStr>>(title: T) {
        let mut t: Vec<u16> = title.as_ref().encode_wide().take(1024).collect();
        t.push(0);

        unsafe { SetConsoleTitleW(t.as_ptr()) };

        let mut handle = EVENT_HANDLE.lock().expect("event handle lock");

        handle.replace(Handle(unsafe {
            CreateEventW(std::ptr::null_mut(), 1, 0, t.as_ptr())
        }));
    }
}

#[cfg(not(any(
    target_os = "freebsd",
    target_os = "hardenedbsd",
    target_os = "dragonflybsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "linux",
    target_os = "windows"
)))]
mod imp {
    use std::ffi::OsStr;

    pub fn set_title<T: AsRef<OsStr>>(_title: T) {}
}

pub use imp::*;
