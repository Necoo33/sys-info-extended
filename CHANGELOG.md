# ChangeLog

## sys-info-extended

### v1.1.0

- Bug in imports in test case fixed.
- cc, libc and palin versions upgraded the latest ones.

### v1.0.2

- Some other modifications made and some tests for c functions made.

### v1.0.1

- README.md file modified.
- Some functions that not returns result type also modified that way.

### v1.0.0

- `index()` functions in `get_disk_info()` function are changed with `strchr()` functions.
- liblary's way of handling operating system configurations of functions changed. All functions which implemented later than fork now returns a result type, and we check if they are used on correct operating system in the functions itself and basically throw an Err variant with explaining that if they are used in wrong os.

### v0.9.2

- Some target os configurations changed.

### v0.9.1

- Added some documentation for the functions that implemented later than `sys-info` fork.
- `is_program_installed()` functions algorithm improved.

### v0.9.0

- Added `UserConfigurations` struct for linux, that includes home directory and shell preference of the user and `get_home_dir_and_shell()` function for getting that struct.
- cc and libc versions upgraded.

### v0.8.1

- Fixed a documentation typo that can mislead people.

### v0.8.0

- Added `append_env()` and `set_env()` functions for windows. It needs an options argument which has type `EnvOptions` for defining env's name, value and which level of env it is. If you want to set or append an env to System, you have to run the application that your code included as administrator.
- A bug on `get_language_options()` function debugged, it made public.

### 0.7.0

- `get_language_options()` function and `LanguageOptions` struct added. It'll take also improvements on next releases.
- palin, cc and libc versions upgraded.


### 0.6.0

- `is_program_installed()` function updated, it's algorithm improved.
-- cc and libc versions upgraded.

### 0.5.0

- added `get_system_env_var()` and `get_user_env_var()` functions. First works on both Windows and Linux, second works on only Windows. They took only one argument and that is the var name. They return to the environment variable's value. But they don't checks the whether characters are in utf8 format or not: because Windows api's won't return answers with Utf8 format, so if env variable's value includes non ascii characters and when you want to get them as Utf8 Rust String, that makes code panic. Because of that functions don't control whether they are utf8 or not and if that variable's value includes non ascii character that characters basically broke. Because of that, use that function with caution. 

### 0.4.0

- added `RamInfo` struct and `get_ram_infos()` function for only windows. It shows the megahertz infos of your individual rams that connected on motherboard and determines which ddr type it is. It has some drawbacks, it accepts 400-800 megahertz's as ddr2 and accept 801-1860 megahertz's as ddr3. Because there is no other way to determine which type if that ram has mhz between 800-1066 mhz, we cannot implemented it.

### 0.3.0

- added `is_program_installed_search_hard()` function for only windows. Because windows has many api's for listing installed programs and some of them are extremely slow, i tried to made it as efficient as possible. Because of that, that function takes second argument is a struct that named `HardSearchOptions`. You can customize your searching via that struct. You have to select your searching is case sensitive or not and how hard it'll be. The 6 is hardest, more than 6 has the same effect. equal or less than 3 is same with easy search, if you're sure your program is reachable on terminal i strongly recommend to set hardness as 3 or use `is_program_installed()` function instead. If it doesn't, then i recommend that try lower to harder for the sake of performance.
- added `is_program_installed()` function for all operating systems.
- palin version upgraded to v0.3.0
- cc version upgraded to v1.0.83
- libc version upgraded to v0.2.153

### 0.2.2

- palin version upgraded to v0.2.0
- a bug fixed.

### 0.2.1

- a bug fixed on `get_public_ipv4_address()` when using linux.

### 0.2.0

- added `get_public_ipv4_address()` function for windows and linux. It requires either dig, wget or curl has to be installed on the running system for linux. It requires internet connection for running properly.

### 0.1.2

- added `get_current_user()` function for windows and linux. It checks the current user in the running moment of your code.

### 0.1.1

- added `check_computer_type()` function for windows and linux. It checks if your computer is desktop, laptop or another type of computer. It's only Desktop or Notebook answers for linux. Basically if your computer has batteries, that function return "Notebook" value as &str.
- Some documentation fixes

### 0.1.0

- added `get_graphics_info()` function for windows, which includes every property of windows's VideoController class.

## sys-info

### 0.9.1

- Fix iOS Support and CPU speed doesn't work on ARM64 Macs either.
- Rust Nightly fix
- Add a cast to allow building on ILP32 systems
- Prevent free swap from getting larger than total swap
- Fix compiler errors/warnings for NetBSD/arm

### 0.9.0

- Typo fixes & test fixup
- macOS: On failure copy the unknown value and null-terminate it correctly
- Fix windows-gnu build
- Support for NetBSD platform

### 0.8.0

- Fix build for target android
- add OpenBSD support
- Make get_cpu_speed arch-independent on windows
- improve get_mem_info on macos
- Make Disk Info Thread-Safe on Linux
- Loop to find max CPU speed in Windows get_cpu_speed

### 0.7.0

- FreeBSD port.

### 0.6.1

- Restore `Send` trait to `Error` for wrapping with `error-chain`
- Use cfg attribute instead of cfg! macro, which fixes Windows build errors in v0.6.0

### 0.6.0

- Support illumos and Solaris systems

### 0.5.10

- Cast gethostname() arguments to a proper type.

### 0.5.9

- Optimize getHostname for hostname command might not be installed.

### 0.5.8

- Support get os-release information for Linux #38
