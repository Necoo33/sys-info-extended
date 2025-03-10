//! #Introduction
//! This crate focuses on geting system information.
//!
//! For now it supports Linux, Mac OS X and Windows.
//! And now it can get information of kernel/cpu/memory/disk/load/hostname and so on.
//!

extern crate libc;
extern crate palin;

use std::ffi;
use std::fmt;
use std::fmt::Display;
use std::io::{self, Read};
use std::fs::File;
#[cfg(any(target_os = "windows", target_vendor = "apple", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
use std::os::raw::c_char;
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "haiku")))]
use std::os::raw::{c_int, c_double};

#[cfg(any(target_vendor = "apple", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
use libc::sysctl;
#[cfg(any(target_vendor = "apple", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
use std::mem::size_of_val;
#[cfg(any(target_vendor = "apple", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
use std::ptr::null_mut;
#[cfg(not(target_os = "windows"))]
use libc::timeval;
#[cfg(any(target_os = "solaris", target_os = "illumos"))]
use std::time::SystemTime;
#[cfg(target_os = "linux")]
use std::collections::HashMap;

#[cfg(any(target_os = "solaris", target_os = "illumos"))]
mod kstat;

#[cfg(any(target_vendor = "apple", target_os="freebsd", target_os = "openbsd", target_os = "netbsd"))]
static OS_CTL_KERN: libc::c_int = 1;
#[cfg(any(target_vendor = "apple", target_os="freebsd", target_os = "openbsd", target_os = "netbsd"))]
static OS_KERN_BOOTTIME: libc::c_int = 21;

/// System load average value.
#[repr(C)]
#[derive(Debug)]
pub struct LoadAvg {
    /// Average load within one minutes.
    pub one: f64,
    /// Average load within five minutes.
    pub five: f64,
    /// Average load within fifteen minutes.
    pub fifteen: f64,
}

/// System memory information.
#[repr(C)]
#[derive(Debug)]
pub struct MemInfo {
    /// Total physical memory.
    pub total: u64,
    pub free: u64,
    pub avail: u64,

    pub buffers: u64,
    pub cached: u64,

    /// Total swap memory.
    pub swap_total: u64,
    pub swap_free: u64,
}

/// The os release info of Linux.
///
/// See [man os-release](https://www.freedesktop.org/software/systemd/man/os-release.html).
#[derive(Debug)]
#[derive(Default)]
pub struct LinuxOSReleaseInfo {
    /// A lower-case string (no spaces or other characters outside of 0–9, a–z, ".", "_" and "-")
    /// identifying the operating system, excluding any version information and suitable for
    /// processing by scripts or usage in generated filenames.
    ///
    /// Note that we don't verify that the string is lower-case and can be used in file-names. If
    /// the /etc/os-release file has an invalid value, you will get this value.
    ///
    /// If not set, defaults to "ID=linux". Use `self.id()` to fallback to the default.
    ///
    /// Example: "fedora" or "debian".
    pub id: Option<String>,

    /// A space-separated list of operating system identifiers in the same syntax as the ID=
    /// setting. It should list identifiers of operating systems that are closely related to the
    /// local operating system in regards to packaging and programming interfaces, for example
    /// listing one or more OS identifiers the local OS is a derivative from. An OS should
    /// generally only list other OS identifiers it itself is a derivative of, and not any OSes
    /// that are derived from it, though symmetric relationships are possible. Build scripts and
    /// similar should check this variable if they need to identify the local operating system and
    /// the value of ID= is not recognized. Operating systems should be listed in order of how
    /// closely the local operating system relates to the listed ones, starting with the closest.
    ///
    /// This field is optional.
    ///
    /// Example: for an operating system with `ID=centos`, an assignment of `ID_LIKE="rhel fedora"`
    /// would be appropriate. For an operating system with `ID=ubuntu`, an assignment of
    /// `ID_LIKE=debian` is appropriate.
    pub id_like: Option<String>,

    /// A string identifying the operating system, without a version component, and suitable for
    /// presentation to the user.
    ///
    /// If not set, defaults to "NAME=Linux".Use `self.id()` to fallback to the default.
    ///
    /// Example: "Fedora" or "Debian GNU/Linux".
    pub name: Option<String>,

    /// A pretty operating system name in a format suitable for presentation to the user. May or
    /// may not contain a release code name or OS version of some kind, as suitable.
    ///
    /// If not set, defaults to "Linux". Use `self.id()` to fallback to the default.
    ///
    /// Example: "Fedora 17 (Beefy Miracle)".
    pub pretty_name: Option<String>,

    /// A string identifying the operating system version, excluding any OS name information,
    /// possibly including a release code name, and suitable for presentation to the user.
    ///
    /// This field is optional.
    ///
    /// Example: "17" or "17 (Beefy Miracle)"
    pub version: Option<String>,

    /// A lower-case string (mostly numeric, no spaces or other characters outside of 0–9, a–z,
    /// ".", "_" and "-") identifying the operating system version, excluding any OS name
    /// information or release code name, and suitable for processing by scripts or usage in
    /// generated filenames.
    ///
    /// This field is optional.
    ///
    /// Example: "17" or "11.04".
    pub version_id: Option<String>,

    /// A lower-case string (no spaces or other characters outside of 0–9, a–z, ".", "_" and "-")
    /// identifying the operating system release code name, excluding any OS name information or
    /// release version, and suitable for processing by scripts or usage in generated filenames.
    ///
    /// This field is optional and may not be implemented on all systems.
    ///
    /// Examples: "buster", "xenial".
    pub version_codename: Option<String>,

    /// A suggested presentation color when showing the OS name on the console. This should be
    /// specified as string suitable for inclusion in the ESC [ m ANSI/ECMA-48 escape code for
    /// setting graphical rendition.
    ///
    /// This field is optional.
    ///
    /// Example: "0;31" for red, "1;34" for light blue, or "0;38;2;60;110;180" for Fedora blue.
    pub ansi_color: Option<String>,

    /// A string, specifying the name of an icon as defined by freedesktop.org Icon Theme
    /// Specification. This can be used by graphical applications to display an operating
    /// system's or distributor's logo.
    ///
    /// This field is optional and may not necessarily be implemented on all systems.
    ///
    /// Examples: "LOGO=fedora-logo", "LOGO=distributor-logo-opensuse".
    pub logo: Option<String>,

    /// A CPE name for the operating system, in URI binding syntax, following the Common Platform
    /// Enumeration Specification as proposed by the NIST.
    ///
    /// This field is optional.
    ///
    /// Example: "cpe:/o:fedoraproject:fedora:17".
    pub cpe_name: Option<String>,

    /// A string uniquely identifying the system image used as the origin for a distribution (it is
    /// not updated with system updates). The field can be identical between different VERSION_IDs
    /// as BUILD_ID is an only a unique identifier to a specific version. Distributions that
    /// release each update as a new version would only need to use VERSION_ID as each build is
    /// already distinct based on the VERSION_ID.
    ///
    /// This field is optional.
    ///
    /// Example: "2013-03-20.3" or "BUILD_ID=201303203".
    pub build_id: Option<String>,

    /// A string identifying a specific variant or edition of the operating system suitable for
    /// presentation to the user. This field may be used to inform the user that the configuration
    /// of this system is subject to a specific divergent set of rules or default configuration
    /// settings.
    ///
    /// This field is optional and may not be implemented on all systems.
    ///
    /// Examples: "Server Edition", "Smart Refrigerator Edition".
    ///
    /// Note: this field is for display purposes only. The VARIANT_ID field should be used for
    /// making programmatic decisions.
    pub variant: Option<String>,

    /// A lower-case string (no spaces or other characters outside of 0–9, a–z, ".", "_" and "-"),
    /// identifying a specific variant or edition of the operating system. This may be interpreted
    /// by other packages in order to determine a divergent default configuration.
    ///
    /// This field is optional and may not be implemented on all systems.
    ///
    /// Examples: "server", "embedded".
    pub variant_id: Option<String>,

    /// HOME_URL= should refer to the homepage of the operating system, or alternatively some homepage of
    /// the specific version of the operating system.
    ///
    /// These URLs are intended to be exposed in "About this system" UIs behind links with captions
    /// such as "About this Operating System", "Obtain Support", "Report a Bug", or "Privacy
    /// Policy". The values should be in RFC3986 format, and should be "http:" or "https:" URLs,
    /// and possibly "mailto:" or "tel:". Only one URL shall be listed in each setting. If multiple
    /// resources need to be referenced, it is recommended to provide an online landing page
    /// linking all available resources.
    ///
    /// Example: "https://fedoraproject.org/".
    pub home_url: Option<String>,

    /// DOCUMENTATION_URL= should refer to the main documentation page for this operating system.
    ///
    /// See also `home_url`.
    pub documentation_url: Option<String>,

    /// SUPPORT_URL= should refer to the main support page for the operating system, if there is
    /// any. This is primarily intended for operating systems which vendors provide support for.
    ///
    /// See also `home_url`.
    pub support_url: Option<String>,

    /// BUG_REPORT_URL= should refer to the main bug reporting page for the operating system, if
    /// there is any. This is primarily intended for operating systems that rely on community QA.
    ///
    /// Example: "https://bugzilla.redhat.com/".
    ///
    /// See also `home_url`.
    pub bug_report_url: Option<String>,

    /// PRIVACY_POLICY_URL= should refer to the main privacy policy page for the operating system,
    /// if there is any. These settings are optional, and providing only some of these settings is
    /// common.
    ///
    /// See also `home_url`.
    pub privacy_policy_url: Option<String>,
}

macro_rules! os_release_defaults {
    (
        $(
            $(#[$meta:meta])*
            $vis:vis fn $field:ident => $default:literal
        )*
    ) => {
        $(
            $(#[$meta])*
            $vis fn $field(&self) -> &str {
                match self.$field.as_ref() {
                    Some(value) => value,
                    None => $default,
                }
            }
        )*
    }
}

impl LinuxOSReleaseInfo {
    os_release_defaults!(
        /// Returns the value of `self.id` or, if `None`, "linux" (the default value).
        pub fn id => "linux"
        /// Returns the value of `self.name` or, if `None`, "Linux" (the default value).
        pub fn name => "Linux"
        /// Returns the value of `self.pretty_name` or, if `None`, "Linux" (the default value).
        pub fn pretty_name => "Linux"
    );
}

/// Disk information.
#[repr(C)]
#[derive(Debug)]
pub struct DiskInfo {
    pub total: u64,
    pub free: u64,
}

/// Error types
#[derive(Debug)]
pub enum Error {
    UnsupportedSystem,
    ExecFailed(io::Error),
    IO(io::Error),
    SystemTime(std::time::SystemTimeError),
    General(String),
    Other(String),
    Unknown,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            UnsupportedSystem => write!(fmt, "System is not supported"),
            ExecFailed(ref e) => write!(fmt, "Execution failed: {}", e),
            IO(ref e) => write!(fmt, "IO error: {}", e),
            SystemTime(ref e) => write!(fmt, "System time error: {}", e),
            General(ref e) => write!(fmt, "Error: {}", e),
            Unknown => write!(fmt, "An unknown error occurred"),
            Other(ref e) => write!(fmt, "Error: {}", e)
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            UnsupportedSystem => "unsupported system",
            ExecFailed(_) => "execution failed",
            IO(_) => "io error",
            SystemTime(_) => "system time",
            General(_) => "general error",
            Other(_) => "other error",
            Unknown => "unknown error",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        use self::Error::*;
        match *self {
            UnsupportedSystem => None,
            ExecFailed(ref e) => Some(e),
            IO(ref e) => Some(e),
            SystemTime(ref e) => Some(e),
            Other(_) => None,
            General(_) => None,
            Unknown => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Error {
        Error::SystemTime(e)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(e: Box<dyn std::error::Error>) -> Error {
        Error::General(e.to_string())
    }
}

extern "C" {
    #[cfg(any(target_vendor = "apple", target_os = "windows"))]
    fn get_os_type() -> *const i8;
    #[cfg(any(target_vendor = "apple", target_os = "windows", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    fn get_os_release() -> *const i8;

    #[cfg(all(not(any(target_os = "solaris", target_os = "illumos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")), any(unix, windows)))]
    fn get_cpu_num() -> u32;
    #[cfg(any(all(target_vendor = "apple", not(any(target_arch = "aarch64", target_arch = "arm"))), target_os = "windows", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "haiku"))]
    fn get_cpu_speed() -> u64;

    #[cfg(target_os = "windows")]
    fn get_loadavg() -> LoadAvg;
    #[cfg(any(target_vendor = "apple", target_os = "windows", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "haiku"))]
    fn get_proc_total() -> u64;

    #[cfg(any(target_vendor = "apple", target_os = "windows", target_os = "haiku"))]
    fn get_mem_info() -> MemInfo;
    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    fn get_mem_info_bsd(mi: &mut MemInfo) ->i32;

    #[cfg(any(target_os = "linux", target_vendor = "apple", target_os = "windows", target_os = "haiku"))]
    fn get_disk_info() -> DiskInfo;
    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    fn get_disk_info_bsd(di: &mut DiskInfo) -> i32;
}


/// Get operation system type.
///
/// Such as "Linux", "Darwin", "Windows".
pub fn os_type() -> Result<String, Error> {
    #[cfg(target_os = "linux")]
    {
        let mut s = String::new();
        File::open("/proc/sys/kernel/ostype")?.read_to_string(&mut s)?;
        s.pop(); // pop '\n'
        Ok(s)
    }
    #[cfg(any(target_vendor = "apple", target_os = "windows"))]
    {
        let typ = unsafe { ffi::CStr::from_ptr(get_os_type() as *const c_char).to_bytes() };
        Ok(String::from_utf8_lossy(typ).into_owned())
    }
    #[cfg(target_os = "solaris")]
    {
        Ok("solaris".to_string())
    }
    #[cfg(target_os = "illumos")]
    {
        Ok("illumos".to_string())
    }
    #[cfg(target_os = "freebsd")]
    {
        Ok("freebsd".to_string())
    }
    #[cfg(target_os = "openbsd")]
    {
        Ok("openbsd".to_string())
    }
    #[cfg(target_os = "netbsd")]
    {
        Ok("netbsd".to_string())
    }
    #[cfg(target_os = "haiku")]
    {
        Ok("haiku".to_string())
    }
    #[cfg(not(any(target_os = "linux", target_vendor = "apple", target_os = "windows", target_os = "solaris", target_os = "illumos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "haiku")))]
    {
        Err(Error::UnsupportedSystem)
    }
}

/// Get operation system release version.
///
/// Such as "3.19.0-gentoo"
pub fn os_release() -> Result<String, Error> {
    #[cfg(target_os = "linux")]
    {
        let mut s = String::new();
        File::open("/proc/sys/kernel/osrelease")?.read_to_string(&mut s)?;
        s.pop(); // pop '\n'
        Ok(s)
    }
    #[cfg(any(target_vendor = "apple", target_os = "windows", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
        unsafe {
	    let rp = get_os_release() as *const c_char;
	    if rp == std::ptr::null() {
		Err(Error::Unknown)
	    } else {
		let typ = ffi::CStr::from_ptr(rp).to_bytes();
		Ok(String::from_utf8_lossy(typ).into_owned())
	    }
	}
    }
    #[cfg(any(target_os = "solaris", target_os = "illumos", target_os = "haiku"))]
    {
        let release: Option<String> = unsafe {
            let mut name: libc::utsname = std::mem::zeroed();
            if libc::uname(&mut name) < 0 {
                None
            } else {
                let cstr = std::ffi::CStr::from_ptr(name.release.as_mut_ptr());
                Some(cstr.to_string_lossy().to_string())
            }
        };
        match release {
            None => Err(Error::Unknown),
            Some(release) => Ok(release),
        }
    }
    #[cfg(not(any(target_os = "linux", target_vendor = "apple", target_os = "windows", target_os = "solaris", target_os = "illumos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "haiku")))]
    {
        Err(Error::UnsupportedSystem)
    }
}

/// Get the os release note of Linux
///
/// Information in /etc/os-release, such as name and version of distribution.
///
/// See `LinuxOSReleaseInfo` for more documentation.
pub fn linux_os_release() -> Result<LinuxOSReleaseInfo, Error> {
    if !cfg!(target_os = "linux") {
        return Err(Error::UnsupportedSystem);
    }

    let mut s = String::new();
    File::open("/etc/os-release")?.read_to_string(&mut s)?;

    let mut info: LinuxOSReleaseInfo = Default::default();
    for l in s.split('\n') {
        match parse_line_for_linux_os_release(l.trim().to_string()) {
            Some((key, value)) =>
                match (key.as_ref(), value) {
                    ("ID", val) => info.id = Some(val),
                    ("ID_LIKE", val) => info.id_like = Some(val),
                    ("NAME", val) => info.name = Some(val),
                    ("PRETTY_NAME", val) => info.pretty_name = Some(val),

                    ("VERSION", val) => info.version = Some(val),
                    ("VERSION_ID", val) => info.version_id = Some(val),
                    ("VERSION_CODENAME", val) => info.version_codename = Some(val),

                    ("ANSI_COLOR", val) => info.ansi_color = Some(val),
                    ("LOGO", val) => info.logo = Some(val),

                    ("CPE_NAME", val) => info.cpe_name = Some(val),
                    ("BUILD_ID", val) => info.build_id = Some(val),
                    ("VARIANT", val) => info.variant = Some(val),
                    ("VARIANT_ID", val) => info.variant_id = Some(val),

                    ("HOME_URL", val) => info.home_url = Some(val),
                    ("BUG_REPORT_URL", val) => info.bug_report_url = Some(val),
                    ("SUPPORT_URL", val) => info.support_url = Some(val),
                    ("DOCUMENTATION_URL", val) => info.documentation_url = Some(val),
                    ("PRIVACY_POLICY_URL", val) => info.privacy_policy_url = Some(val),
                    _ => {}
                }
            None => {}
        }
    }

    Ok(info)
}

fn parse_line_for_linux_os_release(l: String) -> Option<(String, String)> {
    let words: Vec<&str> = l.splitn(2, '=').collect();
    if words.len() < 2 {
        return None
    }
    let mut trim_value = String::from(words[1]);

    if trim_value.starts_with('"') {
        trim_value.remove(0);
    }
    if trim_value.ends_with('"') {
        let len = trim_value.len();
        trim_value.remove(len - 1);
    }

    return Some((String::from(words[0]), trim_value))
}

/// Get cpu num quantity.
///
/// Notice, it returns the logical cpu quantity.
pub fn cpu_num() -> Result<u32, Error> {
    #[cfg(any(target_os = "solaris", target_os = "illumos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
        let ret = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
        if ret < 1 || ret as i64 > std::u32::MAX as i64 {
            Err(Error::IO(io::Error::last_os_error()))
        } else {
            Ok(ret as u32)
        }
    }
    #[cfg(all(not(any(target_os = "solaris", target_os = "illumos", target_os="freebsd", target_os = "openbsd", target_os = "netbsd")), any(unix, windows)))]
    {
        unsafe { Ok(get_cpu_num()) }
    }
    #[cfg(not(any(target_os = "solaris", target_os = "illumos", unix, windows)))]
    {
        Err(Error::UnsupportedSystem)
    }
}

/// Get cpu speed.
///
/// Such as 2500, that is 2500 MHz.
pub fn cpu_speed() -> Result<u64, Error> {
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    {
       Ok(kstat::cpu_mhz()?)
    }
    #[cfg(target_os = "linux")]
    {
        // /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_cur_freq
        let mut s = String::new();
        File::open("/proc/cpuinfo")?.read_to_string(&mut s)?;

        let find_cpu_mhz = s.split('\n').find(|line|
            line.starts_with("cpu MHz\t") ||
                line.starts_with("BogoMIPS") ||
                line.starts_with("clock\t") ||
                line.starts_with("bogomips per cpu")
        );

        find_cpu_mhz.and_then(|line| line.split(':').last())
            .and_then(|val| val.replace("MHz", "").trim().parse::<f64>().ok())
            .map(|speed| speed as u64)
            .ok_or(Error::Unknown)
    }
    #[cfg(any(all(target_vendor = "apple", not(any(target_arch = "aarch64", target_arch = "arm"))), target_os = "windows", target_os = "haiku"))]
    {
        unsafe { Ok(get_cpu_speed()) }
    }
    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
	let res: u64 = unsafe { get_cpu_speed() };
	match res {
	    0 => Err(Error::IO(io::Error::last_os_error())),
	    _ => Ok(res),
	}
    }
    #[cfg(not(any(target_os = "solaris", target_os = "illumos", target_os = "linux", all(target_vendor = "apple", not(any(target_arch = "aarch64", target_arch = "arm"))), target_os = "windows", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "haiku")))]
    {
        Err(Error::UnsupportedSystem)
    }
}

/// Get system load average value.
///
/// Notice, on windows, one/five/fifteen of the LoadAvg returned are the current load.
pub fn loadavg() -> Result<LoadAvg, Error> {
    #[cfg(target_os = "linux")]
    {
        let mut s = String::new();
        File::open("/proc/loadavg")?.read_to_string(&mut s)?;
        let loads = s.trim().split(' ')
            .take(3)
            .map(|val| val.parse::<f64>().unwrap())
            .collect::<Vec<f64>>();
        Ok(LoadAvg {
            one: loads[0],
            five: loads[1],
            fifteen: loads[2],
        })
    }
    #[cfg(any(target_os = "solaris", target_os = "illumos", target_vendor = "apple", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
        let mut l: [c_double; 3] = [0f64; 3];
        if unsafe { libc::getloadavg(l.as_mut_ptr(), l.len() as c_int) } < 3 {
            Err(Error::Unknown)
        } else {
            Ok(LoadAvg {
                one: l[0],
                five: l[1],
                fifteen: l[2],
            })
        }
    }
    #[cfg(any(target_os = "windows"))]
    {
        Ok(unsafe { get_loadavg() })
    }
    #[cfg(not(any(target_os = "linux", target_os = "solaris", target_os = "illumos", target_vendor = "apple", target_os = "windows", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
    {
        Err(Error::UnsupportedSystem)
    }
}

/// Get current processes quantity.
pub fn proc_total() -> Result<u64, Error> {
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    {
        Ok(kstat::nproc()?)
    }
    #[cfg(target_os = "linux")]
    {
        let mut s = String::new();
        File::open("/proc/loadavg")?.read_to_string(&mut s)?;
        s.split(' ')
            .nth(3)
            .and_then(|val| val.split('/').last())
            .and_then(|val| val.parse::<u64>().ok())
            .ok_or(Error::Unknown)
    }
    #[cfg(any(target_vendor = "apple", target_os = "windows", target_os = "haiku"))]
    {
        Ok(unsafe { get_proc_total() })
    }
    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
	let res: u64 = unsafe { get_proc_total() };
	match res {
	    0 => Err(Error::IO(io::Error::last_os_error())),
	    _ => Ok(res),
	}
    }
    #[cfg(not(any(target_os = "linux", target_os = "solaris", target_os = "illumos", target_vendor = "apple", target_os = "windows", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "haiku")))]
    {
        Err(Error::UnsupportedSystem)
    }
}

#[cfg(any(target_os = "solaris", target_os = "illumos"))]
fn pagesize() -> Result<u32, Error> {
    let ret = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if ret < 1 || ret > std::u32::MAX as i64 {
        Err(Error::Unknown)
    } else {
        Ok(ret as u32)
    }
}

/// Get memory information.
///
/// On Mac OS X and Windows, the buffers and cached variables of the MemInfo returned are zero.
pub fn mem_info() -> Result<MemInfo, Error> {
    #[cfg(target_os = "linux")]
    {
        let mut s = String::new();
        File::open("/proc/meminfo")?.read_to_string(&mut s)?;
        let mut meminfo_hashmap = HashMap::new();
        for line in s.lines() {
            let mut split_line = line.split_whitespace();
            let label = split_line.next();
            let value = split_line.next();
            if value.is_some() && label.is_some() {
                let label = label.unwrap().split(':').nth(0).ok_or(Error::Unknown)?;
                let value = value.unwrap().parse::<u64>().ok().ok_or(Error::Unknown)?;
                meminfo_hashmap.insert(label, value);
            }
        }
        let total = *meminfo_hashmap.get("MemTotal").ok_or(Error::Unknown)?;
        let free = *meminfo_hashmap.get("MemFree").ok_or(Error::Unknown)?;
        let buffers = *meminfo_hashmap.get("Buffers").ok_or(Error::Unknown)?;
        let cached = *meminfo_hashmap.get("Cached").ok_or(Error::Unknown)?;
        let avail = meminfo_hashmap.get("MemAvailable").map(|v| v.clone()).or_else(|| {
            let sreclaimable = *meminfo_hashmap.get("SReclaimable")?;
            let shmem = *meminfo_hashmap.get("Shmem")?;
            Some(free + buffers + cached + sreclaimable - shmem)
        }).ok_or(Error::Unknown)?;
        let swap_total = *meminfo_hashmap.get("SwapTotal").ok_or(Error::Unknown)?;
        let swap_free = *meminfo_hashmap.get("SwapFree").ok_or(Error::Unknown)?;
        Ok(MemInfo {
            total,
            free,
            avail,
            buffers,
            cached,
            swap_total,
            swap_free,
        })
    }
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    {
        let pagesize = pagesize()? as u64;
        let pages = kstat::pages()?;
        return Ok(MemInfo {
            total: pages.physmem * pagesize / 1024,
            avail: 0,
            free: pages.freemem * pagesize / 1024,
            cached: 0,
            buffers: 0,
            swap_total: 0,
            swap_free: 0,
        });
    }
    #[cfg(any(target_vendor = "apple", target_os = "windows", target_os = "haiku"))]
    {
        Ok(unsafe { get_mem_info() })
    }
    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
	let mut mi:MemInfo = MemInfo{total: 0, free: 0, avail: 0, buffers: 0,
				     cached: 0, swap_total: 0, swap_free: 0};
	let res: i32 = unsafe { get_mem_info_bsd(&mut mi) };
	match res {
	    -1 => Err(Error::IO(io::Error::last_os_error())),
	    0 => Ok(mi),
	    _ => Err(Error::Unknown),
	}
    }
    #[cfg(not(any(target_os = "linux", target_os = "solaris", target_os = "illumos", target_vendor = "apple", target_os = "windows", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "haiku")))]
    {
        Err(Error::UnsupportedSystem)
    }
}

/// Get disk information.
///
/// Notice, it just calculate current disk on Windows.
pub fn disk_info() -> Result<DiskInfo, Error> {
    #[cfg(any(target_os = "linux", target_vendor = "apple", target_os = "windows", target_os = "haiku"))]
    {
        Ok(unsafe { get_disk_info() })
    }
    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
	let mut di:DiskInfo = DiskInfo{total: 0, free: 0};
	let res: i32 = unsafe { get_disk_info_bsd(&mut di) };
	match res {
	    -1 => Err(Error::IO(io::Error::last_os_error())),
	    0 => Ok(di),
	    _ => Err(Error::Unknown),
	}
    }
    #[cfg(not(any(target_os = "linux", target_vendor = "apple", target_os = "windows", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd", target_os = "haiku")))]
    {
        Err(Error::UnsupportedSystem)
    }
}

/// Get hostname.
#[cfg(target_family = "unix")]
pub fn hostname() -> Result<String, Error> {
    unsafe {
        let buf_size = libc::sysconf(libc::_SC_HOST_NAME_MAX) as usize;
        let mut buf = Vec::<u8>::with_capacity(buf_size + 1);
        if libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf_size) < 0 {
            return Err(Error::IO(io::Error::last_os_error()));
        }
        let hostname_len = libc::strnlen(buf.as_ptr() as *const libc::c_char, buf_size);
        buf.set_len(hostname_len);
        Ok(ffi::CString::new(buf).unwrap().into_string().unwrap())
    }
}

#[cfg(target_family = "windows")]
pub fn hostname() -> Result<String, Error> {
    use std::process::Command;
    Command::new("hostname")
        .output()
        .map_err(Error::ExecFailed)
        .map(|output| String::from_utf8(output.stdout).unwrap().trim().to_string())
}

/// Get system boottime
#[cfg(not(windows))]
pub fn boottime() -> Result<timeval, Error> {
    let mut bt = timeval {
        tv_sec: 0,
        tv_usec: 0
    };

    #[cfg(any(target_os = "linux", target_os="android"))]
    {
        let mut s = String::new();
        File::open("/proc/uptime")?.read_to_string(&mut s)?;
        let secs = s.trim().split(' ')
            .take(2)
            .map(|val| val.parse::<f64>().unwrap())
            .collect::<Vec<f64>>();
        bt.tv_sec = secs[0] as libc::time_t;
        bt.tv_usec = secs[1] as libc::suseconds_t;
	    return Ok(bt);
    }
    #[cfg(any(target_vendor = "apple", target_os="freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
        let mut mib = [OS_CTL_KERN, OS_KERN_BOOTTIME];
        let mut size: libc::size_t = size_of_val(&bt) as libc::size_t;
        unsafe {
            if sysctl(&mut mib[0], 2,
                   &mut bt as *mut timeval as *mut libc::c_void,
                   &mut size, null_mut(), 0) == -1 {
                return Err(Error::IO(io::Error::last_os_error()));
            } else {
                return Ok(bt);
            }
        }
    }
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    {
        let start = kstat::boot_time()?;
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let now = now.as_secs();
        if now < start {
            return Err(Error::General("time went backwards".into()));
        }
        bt.tv_sec = (now - start) as i64;
	    return Ok(bt);
    }

    #[cfg(target_os = "haiku")]
    {
        unsafe {
            let mut sysinfo: libc::system_info = std::mem::zeroed();
            if libc::get_system_info(&mut sysinfo) == libc::B_OK {
                let mut now: libc::time_t = 0;
                libc::time(&mut now);
                bt.tv_sec = now - (sysinfo.boot_time / 1000000);
                bt.tv_usec = (sysinfo.boot_time % 1000000) as libc::suseconds_t;
                return Ok(bt);
            }
        }
        return Err(Error::IO(io::Error::last_os_error()));
    }

    #[warn(unreachable_code)]
    Err(Error::UnsupportedSystem)
}


/// a type that includes all (probably) informations that windows provide about your graphics card.
#[derive(Debug)]
pub struct WindowsGraphicsCard {
    pub name: Vec<String>,
    pub description: Vec<String>,
    pub caption: Vec<String>,
    pub status: Vec<String>,
    pub status_info: Vec<String>,
    pub availability: Vec<String>,
    pub driver_version: Vec<String>,
    pub adapter_ram: Vec<String>,
    pub adapter_dac_type: Vec<String>,
    pub current_refresh_rate: Vec<String>,
    pub max_refresh_rate: Vec<String>,
    pub min_refresh_rate: Vec<String>,
    pub current_bits_per_pixel: Vec<String>,
    pub current_horizontal_resolution: Vec<String>,
    pub current_vertical_resolution: Vec<String>,
    pub current_number_of_colors: Vec<String>,
    pub current_number_of_columns: Vec<String>,
    pub current_number_of_rows: Vec<String>,
    pub current_scan_mode: Vec<String>,
    pub device_id: Vec<String>,
    pub dither_type: Vec<String>,
    pub driver_date: Vec<String>,
    pub icm_intent: Vec<String>,
    pub icm_method: Vec<String>,
    pub inf_file_name: Vec<String>,
    pub inf_section: Vec<String>,
    pub install_date: Vec<String>,
    pub installed_display_drivers: Vec<String>,
    pub max_memory_supported: Vec<String>,
    pub max_number_controlled: Vec<String>,
    pub monochrome: Vec<String>,
    pub number_of_color_planes: Vec<String>,
    pub number_of_video_pages: Vec<String>,
    pub pnp_device_id: Vec<String>,
    pub power_management_capabilities: Vec<String>,
    pub power_management_supported: Vec<String>,
    pub protocol_supported: Vec<String>,
    pub reserved_system_palette_entries: Vec<String>,
    pub specification_version: Vec<String>,
    pub system_creation_classname: Vec<String>,
    pub system_name: Vec<String>,
    pub system_palette_entries: Vec<String>,
    pub time_of_last_reset: Vec<String>,
    pub video_architecture: Vec<String>,
    pub video_memory_type: Vec<String>,
    pub video_mode: Vec<String>,
    pub video_mode_description: Vec<String>,
    pub video_processor: Vec<String>,
    pub accelerator_capabilities: Vec<String>,
    pub capability_descriptions: Vec<String>,
    pub color_table_entries: Vec<String>,
    pub config_manager_error_code: Vec<String>,
    pub config_manager_user_config: Vec<String>,
    pub creation_classname: Vec<String>
}

/// get the graphic card infos, for windows.
pub fn get_graphics_info() -> std::result::Result<WindowsGraphicsCard, std::io::Error> {
    if !cfg!(target_os = "windows") {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "The 'get_graphics_info()' function is only available on windows."));
    }
    use std::process::{Command, Output};
    use std::str;

    let name = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("name").output().unwrap();
    let driver_version = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("driverversion").output().unwrap();
    let adapter_ram = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("adapterram").output().unwrap();
    let video_mode_description = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("VideoModeDescription").output().unwrap();
    let current_refresh_rate = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CurrentRefreshRate").output().unwrap();
    let current_bits_per_pixel = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CurrentBitsPerPixel").output().unwrap();
    let current_number_of_colors = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CurrentNumberOfColors").output().unwrap();
    let current_number_of_columns = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CurrentNumberOfColumns").output().unwrap();
    let current_number_of_rows = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CurrentNumberOfRows").output().unwrap();
    let current_horizontal_resolution = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CurrentHorizontalResolution").output().unwrap();
    let current_vertical_resolution = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CurrentVerticalResolution").output().unwrap();
    let current_scan_mode = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CurrentScanMode").output().unwrap();
    let device_id = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("DeviceID").output().unwrap();
    let dither_type = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("DitherType").output().unwrap();
    let driver_date = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("DriverDate").output().unwrap();
    let icm_intent = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("ICMIntent").output().unwrap();
    let icm_method = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("ICMMethod").output().unwrap();
    let inf_file_name = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("InfFilename").output().unwrap();
    let inf_section = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("InfSection").output().unwrap();
    let install_date = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("InstallDate").output().unwrap();
    let installed_display_drivers = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("InstalledDisplayDrivers").output().unwrap();
    let max_memory_supported = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("MaxMemorySupported").output().unwrap();
    let max_number_controlled = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("MaxNumberControlled").output().unwrap();
    let max_refresh_rate = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("MaxRefreshRate").output().unwrap();
    let min_refresh_rate = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("MinRefreshRate").output().unwrap();
    let monochrome = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("Monochrome").output().unwrap();
    let number_of_color_planes = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("NumberOfColorPlanes").output().unwrap();
    let number_of_video_pages = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("NumberOfVideoPages").output().unwrap();
    let pnp_device_id = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("PNPDeviceID").output().unwrap();
    let power_management_capabilities = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("PowerManagementCapabilities").output().unwrap();
    let power_management_supported = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("PowerManagementSupported").output().unwrap();
    let description = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("Description").output().unwrap();
    let protocol_supported = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("ProtocolSupported").output().unwrap();
    let reserved_system_palette_entries = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("ReservedSystemPaletteEntries").output().unwrap();
    let specification_version = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("SpecificationVersion").output().unwrap();
    let status = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("Status").output().unwrap();
    let status_info = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("StatusInfo").output().unwrap();
    let system_creation_classname = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("SystemCreationClassName").output().unwrap();
    let system_name = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("SystemName").output().unwrap();
    let system_palette_entries = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("SystemPaletteEntries").output().unwrap();
    let time_of_last_reset = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("TimeOfLastReset").output().unwrap();
    let video_architecture = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("VideoArchitecture").output().unwrap();
    let video_memory_type = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("VideoMemoryType").output().unwrap();
    let video_mode = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("VideoMode").output().unwrap();
    let video_processor = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("VideoProcessor").output().unwrap();
    let accelerator_capabilities = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("AcceleratorCapabilities").output().unwrap();
    let adapter_dac_type = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("AdapterDACType").output().unwrap();
    let availability = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("Availability").output().unwrap();
    let capability_descriptions = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CapabilityDescriptions").output().unwrap();
    let caption = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("Caption").output().unwrap();
    let color_table_entries = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("ColorTableEntries").output().unwrap();
    let config_manager_error_code = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("ConfigManagerErrorCode").output().unwrap();
    let config_manager_user_config = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("ConfigManagerUserConfig").output().unwrap();
    let creation_classname = Command::new("wmic").arg("path").arg("win32_VideoController").arg("get").arg("CreationClassName").output().unwrap();

    fn windows_outputs_cleanup(output: Output) -> Vec<String> {
        let parse_output = str::from_utf8(&output.stdout).unwrap();

        let mut new_string_array = vec![];

        for (index, line) in parse_output.lines().enumerate() {
            if index != 0 && line != "" {
                let line = line.replace("\r", "").trim().to_string();

                if line != "".to_string() {
                    new_string_array.push(line.to_string())
                }
            }
        }

        return new_string_array;
    }

    Ok(WindowsGraphicsCard {
        accelerator_capabilities: windows_outputs_cleanup(accelerator_capabilities),
        name: windows_outputs_cleanup(name), 
        description: windows_outputs_cleanup(description),
        caption: windows_outputs_cleanup(caption),
        driver_version: windows_outputs_cleanup(driver_version), 
        availability: windows_outputs_cleanup(availability),
        adapter_ram: windows_outputs_cleanup(adapter_ram),
        adapter_dac_type: windows_outputs_cleanup(adapter_dac_type),  
        current_refresh_rate: windows_outputs_cleanup(current_refresh_rate),
        max_refresh_rate: windows_outputs_cleanup(max_refresh_rate),
        min_refresh_rate: windows_outputs_cleanup(min_refresh_rate),
        current_bits_per_pixel: windows_outputs_cleanup(current_bits_per_pixel), 
        current_number_of_colors: windows_outputs_cleanup(current_number_of_colors), 
        current_number_of_columns: windows_outputs_cleanup(current_number_of_columns), 
        current_number_of_rows: windows_outputs_cleanup(current_number_of_rows), 
        current_horizontal_resolution: windows_outputs_cleanup(current_horizontal_resolution),
        current_vertical_resolution: windows_outputs_cleanup(current_vertical_resolution),
        current_scan_mode: windows_outputs_cleanup(current_scan_mode),
        device_id: windows_outputs_cleanup(device_id),
        dither_type: windows_outputs_cleanup(dither_type),
        driver_date: windows_outputs_cleanup(driver_date),
        icm_intent: windows_outputs_cleanup(icm_intent),
        icm_method: windows_outputs_cleanup(icm_method),
        inf_file_name: windows_outputs_cleanup(inf_file_name),
        inf_section: windows_outputs_cleanup(inf_section),
        install_date: windows_outputs_cleanup(install_date),
        installed_display_drivers: windows_outputs_cleanup(installed_display_drivers),
        max_memory_supported: windows_outputs_cleanup(max_memory_supported),
        max_number_controlled: windows_outputs_cleanup(max_number_controlled),
        monochrome: windows_outputs_cleanup(monochrome),
        number_of_color_planes: windows_outputs_cleanup(number_of_color_planes),
        number_of_video_pages: windows_outputs_cleanup(number_of_video_pages),
        pnp_device_id: windows_outputs_cleanup(pnp_device_id),
        power_management_capabilities: windows_outputs_cleanup(power_management_capabilities),
        power_management_supported: windows_outputs_cleanup(power_management_supported),
        protocol_supported: windows_outputs_cleanup(protocol_supported),
        reserved_system_palette_entries: windows_outputs_cleanup(reserved_system_palette_entries),
        specification_version: windows_outputs_cleanup(specification_version),
        status: windows_outputs_cleanup(status),
        status_info: windows_outputs_cleanup(status_info),
        system_creation_classname: windows_outputs_cleanup(system_creation_classname),
        system_name: windows_outputs_cleanup(system_name),
        system_palette_entries: windows_outputs_cleanup(system_palette_entries),
        time_of_last_reset: windows_outputs_cleanup(time_of_last_reset),
        video_architecture: windows_outputs_cleanup(video_architecture),
        video_memory_type: windows_outputs_cleanup(video_memory_type),
        video_mode: windows_outputs_cleanup(video_mode),
        video_mode_description: windows_outputs_cleanup(video_mode_description),
        video_processor: windows_outputs_cleanup(video_processor),
        capability_descriptions: windows_outputs_cleanup(capability_descriptions),
        color_table_entries: windows_outputs_cleanup(color_table_entries),
        config_manager_error_code: windows_outputs_cleanup(config_manager_error_code),
        config_manager_user_config: windows_outputs_cleanup(config_manager_user_config),
        creation_classname: windows_outputs_cleanup(creation_classname)
    })
}

/// get the computer type. Only "Notebook" and "Desktop" allowed for linux, checks if a battery is exist that implemented on your computer and if it exists, return "Notebook" value, otherwise "Desktop" value. But the way it work on windows is different, it can return various values since windows has able to give more specific infos about computer types. 
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn check_computer_type<'a>() -> std::result::Result<&'a str, Error> {
    use std::process::{Command, Output};
    use std::str;
    let mut result = "Unknown";

    #[cfg(target_os = "windows")]
    {
        fn computer_type_cleanup_for_windows<'a>(our_output: Output) -> &'a str {
            let get_output = str::from_utf8(&our_output.stdout).unwrap()
                                            .replace("\r", "")
                                            .replace("\n", "")
                                            .replace("chassistypes", "")
                                            .replace("-", "")
                                            .replace(" ", "");
    
            return match get_output.as_str() {
                "{1}" => "Other", "{2}" => "Unknown", "{3}" => "Desktop", "{4}" => "Low Profile Desktop", "{5}" => "Pizza Box",
                "{6}" => "Mini Tower", "{7}" => "Tower", "{8}" => "Portable", "{9}" => "Laptop", "{10}" => "Notebook", "{11}" => "Handheld", 
                "{12}" => "Docking Station", "{13}" => "All-in-One", "{14}" => "Sub-Notebook", "{15}" => "Space Saving", "{16}" => "Lunch Box", 
                "{17}" => "Main System Chassis", "{18}" => "Expansion Chassis", "{19}" => "Sub-Chassis", "{20}" => "Bus Expansion Chassis", 
                "{21}" => "Peripheral Chassis", "{22}" => "Storage Chassis", "{23}" => "Rack Mount Chassis", "{24}" => "Sealed-Case PC",
                &_ => "Unknown" 
            }
        }
    
        let chassis_type_number = Command::new("powershell")
                                            .arg("Get-WmiObject")
                                            .arg("win32_systemenclosure | select chassistypes")
                                            .output();
    
        match chassis_type_number {
            Ok(chassis_num) => result = computer_type_cleanup_for_windows(chassis_num),
            Err(err) => return Err(Error::ExecFailed(err))
        };

        
    };

    #[cfg(target_os = "linux")]
    {
        let check_bat0 = Command::new("sh")
                            .arg("-c")
                            .arg("test -d /sys/class/power_supply/BAT0")
                            .output();


        let check_bat0 = match check_bat0 {
            Ok(bat) => bat.status.success(),
            Err(error) => return Err(Error::ExecFailed(error))
        };

        let check_bat1 = Command::new("sh")
                            .arg("-c")
                            .arg("test -d /sys/class/power_supply/BAT1")
                            .output();

        let check_bat1 = match check_bat1 {
            Ok(bat) => bat.status.success(),
            Err(error) => return Err(Error::ExecFailed(error))
        };

        let check_bat2 = Command::new("sh")
                            .arg("-c")
                            .arg("test -d /sys/class/power_supply/BAT2")
                            .output();

        let check_bat2 = match check_bat2 {
            Ok(bat) => bat.status.success(),
            Err(error) => return Err(Error::ExecFailed(error))
        };

        let check_bat3 = Command::new("sh")
                            .arg("-c")
                            .arg("test -d /sys/class/power_supply/BAT3")
                            .output();

        let check_bat3 = match check_bat3 {
            Ok(bat) => bat.status.success(),
            Err(error) => return Err(Error::ExecFailed(error))
        };

        if check_bat0 || check_bat1 || check_bat2 || check_bat3 {
            result = "Notebook";
        } else {
            result = "Desktop";
        }
    }

    return Ok(result);
}


/// get the current user as string. Both works on windows and linux.
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn get_current_user() -> String {
    use std::process::Command;
    use std::str::from_utf8;
    let result;

    #[cfg(target_os = "windows")]
    {
        let current_user_command = Command::new("cmd")
                                        .arg("/C")
                                        .arg("echo")
                                        .arg("%username%")
                                        .output()
                                        .expect("cannot figured out user");

        result = from_utf8(&current_user_command.stdout).unwrap().trim().to_string()
    }

    #[cfg(target_os = "linux")]
    {
        let current_user_command = Command::new("whoami")
                                                .output()
                                                .expect("cannot figured out user");

        result = from_utf8(&current_user_command.stdout).unwrap().trim().to_string()
    }

    return result
}

/// Get the public ipv4 address as string.
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn get_public_ipv4_address() -> std::result::Result<String, Error> {
    let mut ip_address = String::new();

    #[cfg(target_os = "linux")]
    {
        extern crate palin;
        use palin::{check_if_curl_exist, check_if_dig_exist, check_if_wget_exist};

        if check_if_dig_exist() {
            let dig_command = std::process::Command::new("dig").arg("+short").arg("myip.opendns.com").arg("@resolver1.opendns.com").output();

            match dig_command {
                Ok(answer) => {
                    let parse_answer = std::str::from_utf8(&answer.stdout).unwrap();

                    ip_address = parse_answer.trim().to_string();
                },
                Err(error) => return Err(Error::Other(error.to_string()))
            }
        } else if check_if_wget_exist() {
            let wget_command = std::process::Command::new("wget").arg("-qO-").arg("ifconfig.me/ip").output();

            match wget_command {
                Ok(answer) => {
                    let parse_answer = std::str::from_utf8(&answer.stdout).unwrap();

                    ip_address = parse_answer.trim().to_string();
                },
                Err(error) => return Err(Error::Other(error.to_string()))
            }
        } else if check_if_curl_exist() {
            let curl_command = std::process::Command::new("curl").arg("ifconfig.me/ip").output();

            match curl_command {
                Ok(answer) => {
                    let parse_answer = std::str::from_utf8(&answer.stdout).unwrap();

                    ip_address = parse_answer.trim().to_string();
                },
                Err(error) => return Err(Error::Other(error.to_string()))
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let get_ip_address = std::process::Command::new("powershell").arg("-Command").arg("(Invoke-WebRequest -Uri \"http://ifconfig.me/ip\").Content").output();

        match get_ip_address {
            Ok(answer) => {
                let parse_answer = std::str::from_utf8(&answer.stdout).unwrap();

                ip_address = parse_answer.trim().to_string()
            },
            Err(error) => return Err(Error::Other(error.to_string()))
        }
    }

    return Ok(ip_address)
}

/// that function searchs a program on the terminal if it's exist and / or returns a positive answer to various version arguments. Works on both Windows And Linux.
pub fn is_program_installed(program: &str) -> bool {
    let check1 = std::process::Command::new(program).output();

    match check1 {
        Ok(_) => return true,
        Err(_) => ()
    };

    let check2 = std::process::Command::new(program).arg("version").output();

    match check2 {
        Ok(_) => return true,
        Err(_) => ()
    };

    let check3 = std::process::Command::new(program).arg("--version").output();

    match check3 {
        Ok(_) => return true,
        Err(_) => ()
    }

    let check4 = std::process::Command::new(program).arg("-version").output();

    match check4 {
        Ok(_) => return true,
        Err(_) => ()
    }

    let check5 = std::process::Command::new(program).arg("-v").output();

    match check5 {
        Ok(_) => return true,
        Err(_) => ()
    }

    return false
}


/// type that includes hard search options for `is_program_installed_search_hard()` function.
#[derive(Debug)]
pub struct HardSearchOptions{
    pub case_sensitive: bool,
    pub search_hardness: u8
}

/// Since windows has a bunch of api's that enlists downloaded programs and not all of them reachable via a terminal, that function searchs a program with given name and options on both terminal and various program listing api's of windows. Warning: It runs too slow. Use it with caution.
pub fn is_program_installed_search_hard(program: &str, options: HardSearchOptions) -> std::result::Result<bool, std::io::Error> {
    if !cfg!(target_os = "windows") {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "The 'is_program_installed_search_hard()' function is only available on windows."));
    }

    if options.search_hardness == 0 {
        return Ok(false);
    }

    let check1 = std::process::Command::new(program).output();

    match check1 {
        Ok(_) => return Ok(true),
        Err(_) => ()
    };

    if options.search_hardness == 1 {
        return Ok(false);
    }

    let check2 = std::process::Command::new(program).arg("version").output();

    match check2 {
        Ok(_) => return Ok(true),
        Err(_) => ()
    };

    if options.search_hardness == 2 {
        return Ok(false);
    }

    let check3 = std::process::Command::new(program).arg("--version").output();

    match check3 {
        Ok(_) => return Ok(true),
        Err(_) => ()
    }

    if options.search_hardness == 3 {
        return Ok(false);
    }

    let check4 = std::process::Command::new("powershell").arg("Get-Package").output();

    match check4 {
        Ok(answer) => {
            let parse_answer = String::from_utf8_lossy(&answer.stdout);

            for line in parse_answer.lines() {
                if options.case_sensitive {
                    if line.trim().starts_with(program) {
                        return Ok(true);
                    } 
                } else {
                    let left_side = line.trim().to_lowercase();
                    let left_side = String::from_utf8_lossy(left_side.as_bytes());
                    let left_side = left_side.as_ref();
                    let right_side = program.to_lowercase();
                    let right_side = String::from_utf8_lossy(right_side.as_bytes());
                    let right_side = right_side.as_ref();
                    
                    if left_side.contains(right_side) {
                        return Ok(true)
                    }
                }
            }
        },
        Err(_) => ()
    }

    if options.search_hardness == 4 {
        return Ok(false);
    }

    let check5 = std::process::Command::new("powershell").arg("Get-AppxPackage").output();

    match check5 {
        Ok(answer) => {
            let parse_answer = String::from_utf8_lossy(&answer.stdout);

            for line in parse_answer.lines() {
                match options.case_sensitive {
                    true => {
                        if line.starts_with("Name") {
                            let split_the_line: &str = line.split(":").collect::<Vec<&str>>()[1].trim();

                            if program == split_the_line {
                                return Ok(true);
                            }
                        }

                        if line.starts_with("PackageFullName") {
                            let split_the_line: &str = line.split(":").collect::<Vec<&str>>()[1].trim();

                            if program == split_the_line {
                                return Ok(true);
                            }
                        }

                        if line.starts_with("PackageFamilyName") {
                            let split_the_line: &str = line.split(":").collect::<Vec<&str>>()[1].trim();

                            if program == split_the_line {
                                return Ok(true);
                            }
                        }
                    },
                    false => {
                        if line.starts_with("Name") {
                            let split_the_line: &str = line.split(":").collect::<Vec<&str>>()[1].trim();

                            let left_side = split_the_line.trim().to_lowercase();
                            let left_side = String::from_utf8_lossy(left_side.as_bytes());
                            let left_side = left_side.as_ref();
                            let right_side = program.to_lowercase();
                            let right_side = String::from_utf8_lossy(right_side.as_bytes());
                            let right_side = right_side.as_ref();
                            
                            if left_side == right_side {
                                return Ok(true)
                            }
                        }

                        if line.starts_with("PackageFullName") {
                            let split_the_line: &str = line.split(":").collect::<Vec<&str>>()[1].trim();

                            let left_side = split_the_line.trim().to_lowercase();
                            let left_side = String::from_utf8_lossy(left_side.as_bytes());
                            let left_side = left_side.as_ref();
                            let right_side = program.to_lowercase();
                            let right_side = String::from_utf8_lossy(right_side.as_bytes());
                            let right_side = right_side.as_ref();
                            
                            if left_side == right_side {
                                return Ok(true)
                            }
                        }

                        if line.starts_with("PackageFamilyName") {
                            let split_the_line: &str = line.split(":").collect::<Vec<&str>>()[1].trim();

                            let left_side = split_the_line.trim().to_lowercase();
                            let left_side = String::from_utf8_lossy(left_side.as_bytes());
                            let left_side = left_side.as_ref();
                            let right_side = program.to_lowercase();
                            let right_side = String::from_utf8_lossy(right_side.as_bytes());
                            let right_side = right_side.as_ref();
                            
                            if left_side == right_side {
                                return Ok(true)
                            }
                        }
                    }
                }
            }
        },
        Err(_) => ()
    }

    if options.search_hardness == 5 {
        return Ok(false);
    }

    let check6 = std::process::Command::new("powershell").arg("wmic").arg("product").arg("get").arg("name").output();

    match check6 {
        Ok(answer) => {
            let parse_answer = String::from_utf8_lossy(&answer.stdout);

            for line in parse_answer.lines() {
                if options.case_sensitive {
                    if line.trim() == program {
                        return Ok(true);
                    } 
                } else {
                    if line.trim().to_lowercase().as_bytes() == program.to_lowercase().as_bytes() {
                        return Ok(true)
                    }
                }
            }
        },
        Err(error) => {
            eprintln!("for that reason we cannot take wmic output: {}", error)
        }
    }

    return Ok(false)
}

/// type that includes mghz and ddr type values.
#[derive(Debug)]
pub struct RamInFo {
    pub mhz: i32,
    pub ddr_type: String
}

/// returns the `RamInfo` struct per each ram that attached your computer, that includes the mhz value and ddr type, for only windows. 
pub fn get_ram_infos() -> std::result::Result<Vec<RamInFo>, std::io::Error> {
    if !cfg!(target_os = "windows") {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "The 'get_ram_infos()' function is only available on windows."));
    }

    let ram_info_command = std::process::Command::new("wmic").arg("memorychip").arg("get").arg("speed").output();

    match ram_info_command {
        Ok(answer) => {
            let parse_the_answer = std::str::from_utf8(&answer.stdout).unwrap();
            let mut rams = vec![];

            for (index, line) in parse_the_answer.lines().into_iter().enumerate() {
                let mut mhz: i32 = 0;
                let mut ddr_type: String = "".to_string();

                if index == 0 || line.trim() == "" {
                    continue;
                }

                mhz = line.trim().parse::<i32>().unwrap();

                if mhz >= 200 && mhz <= 400 {
                    ddr_type = "ddr1".to_string();
                }

                if mhz >= 400 && mhz <= 800 {
                    ddr_type = "ddr2".to_string();
                }

                if mhz > 800 && mhz <= 1860 {
                    ddr_type = "ddr3".to_string();
                }

                if mhz >= 2133 && mhz <= 3200 {
                    ddr_type = "ddr4".to_string();
                }

                if mhz > 3200 && mhz <= 8400 {
                    ddr_type = "ddr5".to_string();
                }

                let ram_info = RamInFo {
                    ddr_type, mhz
                };

                rams.push(ram_info);
            }

            return Ok(rams);
        },
        Err(error) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))
    }
}

/// gets value of the env that has given name from the system. only for windows. 
pub fn get_system_env_var(var_name: &str) -> std::result::Result<String, std::io::Error> {
    #[cfg(target_os = "windows")]
    {
        let sanitize_var_name = var_name.to_ascii_uppercase();
        let format_the_command = format!("[System.Environment]::GetEnvironmentVariable('{}', 'Machine')", sanitize_var_name);

        let output = std::process::Command::new("powershell")
                        .arg("-Command")
                        .arg(format_the_command)
                        .output();

        return match output {
            Ok(var_list) => Ok(String::from_utf8_lossy(&var_list.stdout).to_string()),
            Err(error) => Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
        }
    }

    #[cfg(target_os = "linux")]
    {
        let sanitize_var_name = var_name.to_ascii_uppercase();

        let output = std::process::Command::new("printenv")
                        .arg(sanitize_var_name)
                        .output();

        return match output {
            Ok(var_list) => Ok(String::from_utf8_lossy(&var_list.stdout).to_string()),
            Err(error) => Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
        }      
    }
}

/// gets value of the env that has given name from the user. only for windows.
pub fn get_user_env_var(var_name: &str) -> std::result::Result<String, std::io::Error> {
    if !cfg!(target_os = "windows") {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "The 'get_user_env_var()' function is only available on windows."));
    }

    let sanitize_var_name = var_name.to_ascii_uppercase();
    let format_the_command = format!("[System.Environment]::GetEnvironmentVariable('{}', 'User')", sanitize_var_name);

    let output = std::process::Command::new("powershell")
                    .arg("-Command")
                    .arg(format_the_command)
                    .output();

    return match output {
        Ok(var_list) => Ok(String::from_utf8_lossy(&var_list.stdout).to_string()),
        Err(error) => Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
pub struct LanguageOptions {
   pub country: String,
   pub name: String,
   pub shortening: String,
   pub lcid: String
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct LanguageOptions {
   pub shortening: String,
   pub character_encoding: String,
   pub country: String,
}

/// returns the language options for the computer, both works on windows and linux.
pub fn get_language_options() -> std::result::Result<LanguageOptions, std::io::Error> {
   #[cfg(target_os = "windows")]
   {
       let get_lang_options_command = std::process::Command::new("powershell.exe").arg("Get-WinSystemLocale").output();

       match get_lang_options_command {
           Ok(output) => {
               let parse_output = String::from_utf8_lossy(&output.stdout);
               
               for line in parse_output.lines() {
                   if line.starts_with(" ") || line.starts_with("LCID") || line.starts_with("-") || line == "" {
                       continue;
                   }

                   let split_the_line: Vec<&str> = line.split(" ").collect::<Vec<&str>>();

                   let mut lcid = "".to_string();
                   let mut shortening = "".to_string();
                   let mut name = "".to_string();
                   let mut country = "".to_string();

                   let mut i: u8 = 0;
                   for infos in split_the_line {
                       if i == 0 {
                           i = i + 1;

                           lcid = infos.to_string()
                       }

                       if infos == "" {
                           continue;
                       }

                       if infos.contains("-") {
                           shortening = infos.split("-").collect::<Vec<&str>>()[0].to_string()
                       } else {
                           if infos != "" {
                               if infos == "T�rk�e" {
                                   country = "Türkiye".to_string();
                                   name = "Türkçe".to_string();
                               } else {
                                   if infos.starts_with("(") {
                                       country = infos.replace("(", "").replace("(", "")
                                   } else {
                                       name = infos.to_string()
                                   }
                               }
                           }
                       }

                   }

                   return Ok(LanguageOptions{
                       lcid,
                       country,
                       name,
                       shortening
                   })
               }

               return Ok(LanguageOptions {
                   lcid: "".to_string(),
                   country: "".to_string(),
                   name: "".to_string(),
                   shortening: "".to_string()
               })
           },
           Err(error) => {
               Err(std::io::Error::new(std::io::ErrorKind::Other, error))
           }
       }
   }

   #[cfg(target_os = "linux")]
   {
       let get_lang_options_command = std::process::Command::new("locale").output();

       match get_lang_options_command {
           Ok(output) => {
               let parse_answer = String::from_utf8_lossy(&output.stdout);

               let mut shortening = "".to_string();
               let mut character_encoding = "".to_string();
               let mut country = "".to_string();
           
               for line in parse_answer.lines() {
                   if line.starts_with("LANG") {
                       let split_the_line = line.split("=").collect::<Vec<&str>>()[1];

                       let split_the_line_second_time = split_the_line.split(".").collect::<Vec<&str>>();

                       character_encoding = split_the_line_second_time[1].to_string();

                       let stltt = split_the_line_second_time[0].split("_").collect::<Vec<&str>>();

                       shortening = stltt[0].to_string();
                       country = stltt[1].to_string();

                       break;
                   } else {
                       continue;
                   }
               }

               return Ok(LanguageOptions {
                   shortening,
                   country,
                   character_encoding
               })
           },
           Err(error) => Err(std::io::Error::new(std::io::ErrorKind::Other, error))
       }

   }
}

/// Env level implementation for windows.
pub enum EnvLevel {
    User, Machine
}

impl Display for EnvLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &EnvLevel::Machine => write!(f, "System"),
            &EnvLevel::User => write!(f, "User"),
        }
    }
}

/// configurations for working with env's on windows.
pub struct EnvOptions {
    pub level: EnvLevel,
    pub name: String,
    pub value: String
}

/// append a value currently existing env, only windows.
pub fn append_env(options: EnvOptions) -> std::result::Result<(), std::io::Error> {
    if !cfg!(target_os = "windows") {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "The 'append_env()' function is only available on windows."));
    }

    let format_the_command: String;

    match options.level {
        EnvLevel::User => {
            let variable = get_user_env_var(&options.name);

            match variable {
                Ok(value) => {
                    let appended_var = format!("{};{}", value, options.value);

                    format_the_command = format!("[System.Environment]::SetEnvironmentVariable('{}', '{}', 'User')", options.name, appended_var);
        
                },
                Err(error) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("That error occured when we try to find {} variable in user's variable: {}", options.name, error)))
                }
            }
        },
        EnvLevel::Machine => {
            let variable = get_system_env_var(&options.name);

            match variable {
                Ok(value) => {
                    let appended_var = format!("{};{}", value, options.value);

                    format_the_command = format!("[System.Environment]::SetEnvironmentVariable('{}', '{}', 'Machine')", options.name, appended_var)
        
                },
                Err(error) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("That error occured when we try to find {} variable in system's variable: {}", options.name, error)))
                }
            }
        }
    }

    let execute_appending = std::process::Command::new("powershell.exe")
                                                                            .arg("-Command")
                                                                            .arg(format_the_command)
                                                                            .output();

    match execute_appending {
        Ok(_) => {
            println!("{}'s {} env successfully updated.", options.level, options.name);
            Ok(())
        },
        Err(error) => {
            Err(std::io::Error::new(std::io::ErrorKind::Other, format!("That Error Occured When we updating the {}'s {} Env: {}", options.level, options.name, error)))
        }
    }
}


/// set an env variable if it's not exist before, only windows.
pub fn set_env(options: EnvOptions) -> std::result::Result<(), std::io::Error> {
    if !cfg!(target_os = "windows") {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "The 'set_env()' function is only available on windows."));
    }

    let format_the_command: String;

    match options.level {
        EnvLevel::User => format_the_command = format!("[System.Environment]::SetEnvironmentVariable('{}', '{}', 'User')", options.name, options.value),
        EnvLevel::Machine => format_the_command = format!("[System.Environment]::SetEnvironmentVariable('{}', '{}', 'Machine')", options.name, options.value),
    }

    let execute_appending = std::process::Command::new("powershell.exe")
                                                                        .arg("-Command")
                                                                        .arg(format_the_command)
                                                                        .output();

    match execute_appending {
        Ok(_) => {
            println!("{}'s {} env successfully updated.", options.level, options.name);
            Ok(())
        },
        Err(error) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("That Error Occured When we updating the {}'s {} Env: {}", options.level, options.name, error)))
    }
}

/// Type that includes home directory and shell preference of user.
#[derive(Debug, Clone)]
pub struct UserConfigurations {
    pub home_dir: String,
    pub shell: String
}

/// Returns The `UserConfigurations` struct that includes home dir and shell preference of the user. Only works on linux.
pub fn get_home_dir_and_shell(username: &str) -> Result<UserConfigurations, std::io::Error> {
    if !cfg!(target_os = "linux") {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "'get_home_dir_and_shell()' function is only available on linux."));
    }

    let path = std::path::Path::new("/etc/passwd");

    let file = File::open(&path);

    match file {
        Ok(file) => {
            use std::io::BufRead;

            let mut i: usize = 0;

            for line in std::io::BufReader::new(file).lines() {
                if i == 0 {
                    i = 1;
                }

                match line {
                    Ok(line) => {
                        match line.starts_with(username) {
                            true => {
                                let mut split_the_lines = line.split(":");

                                match split_the_lines.nth(5) {
                                    Some(home_dir) => {
                                        match split_the_lines.nth(0) {
                                            Some(shell) => {
                                                return Ok(UserConfigurations {
                                                    home_dir: home_dir.to_string(), 
                                                    shell: shell.to_string()
                                                })
                                            },
                                            None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Error: The /etc/passwd configurations for your user is configured unusually, we cannot find the seventh element on the row of given user."))
                                        }
                                    },
                                    None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Error: The /etc/passwd configurations for your user is configured unusually, we cannot find the sixth element on the row of given user."))
                                }
                            }
                            false => continue,
                        }
                    },
                    Err(error) => {
                        return match error.kind() {
                            std::io::ErrorKind::AddrInUse => Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, error)),
                            std::io::ErrorKind::AddrNotAvailable => Err(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, error)),
                            std::io::ErrorKind::AlreadyExists => Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, error)),
                            std::io::ErrorKind::BrokenPipe => Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, error)),
                            std::io::ErrorKind::ConnectionAborted => Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, error)),
                            std::io::ErrorKind::ConnectionRefused => Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, error)),
                            std::io::ErrorKind::ConnectionReset => Err(std::io::Error::new(std::io::ErrorKind::ConnectionReset, error)),
                            std::io::ErrorKind::Interrupted => Err(std::io::Error::new(std::io::ErrorKind::Interrupted, error)),
                            std::io::ErrorKind::InvalidData => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, error)),
                            std::io::ErrorKind::InvalidInput => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error)),
                            std::io::ErrorKind::NotConnected => Err(std::io::Error::new(std::io::ErrorKind::NotConnected, error)),
                            std::io::ErrorKind::NotFound => Err(std::io::Error::new(std::io::ErrorKind::NotFound, error)),
                            std::io::ErrorKind::OutOfMemory => Err(std::io::Error::new(std::io::ErrorKind::OutOfMemory, error)),
                            std::io::ErrorKind::PermissionDenied => Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, error)),
                            std::io::ErrorKind::TimedOut => Err(std::io::Error::new(std::io::ErrorKind::TimedOut, error)),
                            std::io::ErrorKind::UnexpectedEof => Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, error)),
                            std::io::ErrorKind::Unsupported => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, error)),
                            std::io::ErrorKind::WouldBlock => Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, error)),
                            std::io::ErrorKind::WriteZero => Err(std::io::Error::new(std::io::ErrorKind::WriteZero, error)),
                            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, error)),
                        }
                    }
                }
            }

            match i {
                0 => return Err(std::io::Error::new(std::io::ErrorKind::Other, "An unexpected behavior occured: We could open Your /etc/passwd file but it is empty, which is almost impossible.")),
                _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "It's here because of the rust synthax, impossible to came here."))
            }
        },
        Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error))
    };
}  

/// it returns the system's timezone info. In windows, It returning values are incompatible with tz database timezones such as "Turkey Standard Time" instead of "Europe/Istanbul". 
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn get_timezone() -> Result<String, std::io::Error> {
    if !cfg!(target_os = "windows") && !cfg!(target_os = "linux") {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "'get_timezone()' function is only available on linux and windows."));
    }

    let mut timezone = String::new();

    #[cfg(target_os = "windows")]
    {
    let get_timezone = std::process::Command::new("powershell.exe").arg("Get-TimeZone").output();

    match get_timezone {
        Ok(tz) => {
            let output = String::from_utf8_lossy(&tz.stdout);

            for line in output.lines() {
                if line.starts_with("Id") {
                    timezone = line.split(" : ").nth(1).unwrap().to_string();
                }
            }
        },
        Err(error) => return Err(error)
    }
    }

    #[cfg(target_os = "linux")]
    {
        let path = std::path::Path::new("/etc/timezone");

        let file = File::open(&path);

        match file {
            Ok(file) => {
                use std::io::BufRead;

                for line in std::io::BufReader::new(file).lines() {
                    match line {
                        Ok(l) => {
                            if !l.starts_with(" ") {
                                timezone = l
                            }
                        },
                        Err(error) => {
                            return match error.kind() {
                                std::io::ErrorKind::AddrInUse => Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, error)),
                                std::io::ErrorKind::AddrNotAvailable => Err(std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, error)),
                                std::io::ErrorKind::AlreadyExists => Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, error)),
                                std::io::ErrorKind::BrokenPipe => Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, error)),
                                std::io::ErrorKind::ConnectionAborted => Err(std::io::Error::new(std::io::ErrorKind::ConnectionAborted, error)),
                                std::io::ErrorKind::ConnectionRefused => Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, error)),
                                std::io::ErrorKind::ConnectionReset => Err(std::io::Error::new(std::io::ErrorKind::ConnectionReset, error)),
                                std::io::ErrorKind::Interrupted => Err(std::io::Error::new(std::io::ErrorKind::Interrupted, error)),
                                std::io::ErrorKind::InvalidData => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, error)),
                                std::io::ErrorKind::InvalidInput => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error)),
                                std::io::ErrorKind::NotConnected => Err(std::io::Error::new(std::io::ErrorKind::NotConnected, error)),
                                std::io::ErrorKind::NotFound => Err(std::io::Error::new(std::io::ErrorKind::NotFound, error)),
                                std::io::ErrorKind::OutOfMemory => Err(std::io::Error::new(std::io::ErrorKind::OutOfMemory, error)),
                                std::io::ErrorKind::PermissionDenied => Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, error)),
                                std::io::ErrorKind::TimedOut => Err(std::io::Error::new(std::io::ErrorKind::TimedOut, error)),
                                std::io::ErrorKind::UnexpectedEof => Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, error)),
                                std::io::ErrorKind::Unsupported => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, error)),
                                std::io::ErrorKind::WouldBlock => Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, error)),
                                std::io::ErrorKind::WriteZero => Err(std::io::Error::new(std::io::ErrorKind::WriteZero, error)),
                                _ => Err(std::io::Error::new(std::io::ErrorKind::Other, error)),
                            }
                        }
                    }
                }

            },
            Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error))
        }
    }

    Ok(timezone)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_os_type() {
        let typ = os_type().unwrap();
        assert!(typ.len() > 0);
        println!("os_type(): {}", typ);
    }

    #[test]
    pub fn test_os_release() {
        let release = os_release().unwrap();
        assert!(release.len() > 0);
        println!("os_release(): {}", release);
    }

    #[test]
    pub fn test_cpu_num() {
        let num = cpu_num().unwrap();
        assert!(num > 0);
        println!("cpu_num(): {}", num);
    }

    #[test]
    #[cfg(not(all(target_vendor = "apple", target_arch = "aarch64")))]
    pub fn test_cpu_speed() {
        let speed = cpu_speed().unwrap();
        assert!(speed > 0);
        println!("cpu_speed(): {}", speed);
    }

    #[test]
    pub fn test_loadavg() {
        let load = loadavg().unwrap();
        println!("loadavg(): {:?}", load);
    }

    #[test]
    pub fn test_proc_total() {
        let procs = proc_total().unwrap();
        assert!(procs > 0);
        println!("proc_total(): {}", procs);
    }

    #[test]
    pub fn test_mem_info() {
        let mem = mem_info().unwrap();
        assert!(mem.total > 0);
        println!("mem_info(): {:?}", mem);
    }

    #[test]
    #[cfg(not(any(target_os = "solaris", target_os = "illumos")))]
    pub fn test_disk_info() {
        let info = disk_info().unwrap();
        println!("disk_info(): {:?}", info);
    }

    #[test]
    pub fn test_hostname() {
        let host = hostname().unwrap();
        assert!(host.len() > 0);
        println!("hostname(): {}", host);
    }

    #[cfg(target_os = "windows")]
    #[test]
    pub fn test_get_graphics_info(){
        let graphics = get_graphics_info();
        println!("Graphics info: {:?}", graphics);
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    #[test]
    pub fn test_check_computer_type() {
        let pc_type = check_computer_type();
        println!("computer type: {}", pc_type.unwrap());
    }

    #[test]
    #[cfg(not(windows))]
    pub fn test_boottime() {
        let bt = boottime().unwrap();
        println!("boottime(): {} {}", bt.tv_sec, bt.tv_usec);
        assert!(bt.tv_sec > 0 || bt.tv_usec > 0);
    }

    #[test]
    #[cfg(target_os = "linux")]
    pub fn test_linux_os_release() {
        let os_release = linux_os_release().unwrap();
        println!("linux_os_release(): {:?}", os_release.name)
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[test]
    pub fn test_get_public_ipv4_address(){
        assert_ne!(String::new(), get_public_ipv4_address().unwrap())
    }

    #[cfg(target_os = "windows")]
    #[test]
    pub fn test_is_program_installed_search_hard(){
        let hard_search_options = HardSearchOptions {
            case_sensitive: false, // if this is false, you don't need to match lower or upper cases.
            search_hardness: 5 // the biggest level is 6, and it's it's slowest level. If your program is available on terminal, choose 3 instead.
        };

        assert_eq!(true, is_program_installed_search_hard("miCroSoft eDgE", hard_search_options).unwrap());

        let hard_search_options_2 = HardSearchOptions {
            case_sensitive: true,
            search_hardness: 5
        };

        assert_eq!(true, is_program_installed_search_hard("Microsoft Edge", hard_search_options_2).unwrap())
    }

    #[cfg(target_os = "windows")]
    #[test]
    pub fn test_get_ram_infos() {
        let ram_infos = get_ram_infos().unwrap();
        assert!(ram_infos.len() > 0);
        println!("get_ram_infos(): {:#?}", ram_infos);
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[test]
    pub fn test_get_system_env_var(){
        let path_env_var = get_system_env_var("PATH");

        assert_eq!(true, path_env_var.is_ok());
    }

    #[cfg(target_os = "windows")]
    #[test]
    pub fn test_get_user_env_var(){
        let path_env_var = get_user_env_var("PATH");

        assert_eq!(true, path_env_var.is_ok());
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[test]
    pub fn test_get_timezone(){
        let timezone = get_timezone();

        assert_eq!(true, timezone.is_ok())
    }
}
