# sys-info-extended

This crate is fork of [text](https://crates.io/crates/sys-info) crate, and i'll continue to develop that crate. Contributions are welcome.

Get system information in Rust.

For now it supports Linux, Mac OS X, illumos, Solaris, FreeBSD, OpenBSD, NetBSD and Windows.
And now it can get information of kernel/cpu/memory/disk/load/hostname/graphics and so on.

### Usage
Add this to `Cargo.toml`:

```
[dependencies]
sys-info = "0.9"
```

and add this to crate root:

```rust

use sys_info::{os_type, os_release, get_graphics_info};

```

use some functions:

```rust

let our_os_type = os_type().unwrap();
let os_release = os_release().unwrap();
let graphics = get_graphics_info();

```

### Already Planned Features For Next Releases:

* Camera Infos
* USB Infos
* Mouse Infos
* All other windows system classes
