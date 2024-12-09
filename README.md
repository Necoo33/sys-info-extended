# sys-info-extended

This crate is a fork of [sys-info](https://crates.io/crates/sys-info) crate, and i'll continue to develop. Contributions are welcome especially for mac os.

Get system information in Rust.

For now it supports Linux, Mac OS X, illumos, Solaris, FreeBSD, OpenBSD, NetBSD and Windows.
And now it can get information of kernel/cpu/memory/disk/load/hostname/graphics and so on.

I especially focused on very practical informations about system(computer type, user name, public ipv4 address etc.) and especially Windows api's. Because of that, in my opinion it's the best crate for getting system info especially for windows. I aim the include outputs of all windows classes in future releases. So if you're a game developer or windows programmer, this will be one of the go-to crates for you.

If you like this liblary, give a star on it's [github repo](https://github.com/Necoo33/sys-info-extended)

## Usage

Add this to `Cargo.toml`:

```toml
[dependencies]
sys-info-extended = "0.8.0"
```

and add this to crate root:

```rust

use sys_info_extended::{os_type, os_release, get_graphics_info, get_system_env_var, get_public_ipv4_address, append_env, set_env};

```

use some functions:

```rust

let our_os_type = os_type().unwrap();
let os_release = os_release().unwrap();
let graphics = get_graphics_info();
let path_env = get_system_env_var("PATH").unwrap();
let ip_address = get_public_ipv4_address();

let env_option = EnvOptions {
    name: "A Env name that not exist",
    value: "A value",
    level: EnvLevel::User
}

set_env(env_option);

```

### Already Planned Features For Next Releases

* Optimizations and idiomaticizations on later implemented functions.
* adding `get_download_speed()` function which measures your network's download speed.
* Camera Infos
* USB Infos
* Mouse Infos
* All other windows system classes
