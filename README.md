# sys-info-extended

This crate is a fork of [sys-info](https://crates.io/crates/sys-info) crate, and i'll continue to develop. Contributions are welcome especially for mac os.

Get system information in Rust.

For now it supports Linux, Mac OS X, illumos, Solaris, FreeBSD, OpenBSD, NetBSD and Windows.
And now it can get information of kernel/cpu/memory/disk/load/hostname/graphics and so on.

If you like this liblary, give a star on it's [github repo](https://github.com/Necoo33/sys-info-extended)

## Usage

Add this to `Cargo.toml`:

```toml
[dependencies]
sys-info-extended = "0.1.2"
```

and add this to crate root:

```rust

use sys_info_extended::{os_type, os_release, get_graphics_info};

```

use some functions:

```rust

let our_os_type = os_type().unwrap();
let os_release = os_release().unwrap();
let graphics = get_graphics_info();

```

### Already Planned Features For Next Releases

* Camera Infos
* USB Infos
* Mouse Infos
* All other windows system classes
