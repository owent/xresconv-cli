//! Own the backend and its descendants until all redirected pipes have closed.
use std::io;
use std::process::{Child, Command};

pub(crate) struct ChildTree {
    pub child: Child,
    killed: bool,
    #[cfg(windows)]
    job: windows_sys::Win32::Foundation::HANDLE,
}

impl ChildTree {
    pub fn spawn(command: &mut Command) -> io::Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            use windows_sys::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;
            command.creation_flags(CREATE_NEW_PROCESS_GROUP);
        }
        let child = command.spawn()?;
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::{Foundation::CloseHandle, System::JobObjects::*};
            // SAFETY: all handles are owned here; the information structure is initialized,
            // has the exact API size, and outlives SetInformationJobObject.
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                let mut tree = Self {
                    child,
                    job,
                    killed: false,
                };
                if job.is_null() {
                    return Err(io::Error::last_os_error());
                }
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                if SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const _,
                    std::mem::size_of_val(&info) as u32,
                ) == 0
                    || AssignProcessToJobObject(job, tree.child.as_raw_handle()) == 0
                {
                    let err = io::Error::last_os_error();
                    CloseHandle(job);
                    tree.job = std::ptr::null_mut();
                    return Err(err);
                }
                Ok(tree)
            }
        }
        #[cfg(not(windows))]
        {
            Ok(Self {
                child,
                killed: false,
            })
        }
    }

    pub fn kill(&mut self) {
        if self.killed {
            return;
        }
        self.killed = true;
        #[cfg(unix)]
        // SAFETY: this child's process group was created by spawn; a negative pid
        // targets that group, never the caller's or other workers' groups.
        unsafe {
            libc::kill(-(self.child.id() as i32), libc::SIGKILL);
        }
        #[cfg(windows)]
        if !self.job.is_null() {
            // SAFETY: the job remains owned and open until Drop.
            unsafe {
                windows_sys::Win32::System::JobObjects::TerminateJobObject(self.job, 1);
            }
        }
        let _ = self.child.kill();
    }
}

impl Drop for ChildTree {
    fn drop(&mut self) {
        self.kill();
        let _ = self.child.wait();
        #[cfg(windows)]
        if !self.job.is_null() {
            // SAFETY: release the job exactly once after stopping the owned tree.
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self.job);
            }
        }
    }
}
