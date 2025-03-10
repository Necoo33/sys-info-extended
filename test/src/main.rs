
extern crate sys_info_extended;

use sys_info_extended::*;

fn main() {

    println!("os: {} {}", os_type().unwrap(), os_release().unwrap());
    println!("cpu: {} cores, {} MHz", cpu_num().unwrap(), cpu_speed().unwrap());
    println!("proc total: {}", proc_total().unwrap());
    let load = loadavg().unwrap();
    println!("load: {} {} {}", load.one, load.five, load.fifteen);
    let mem = mem_info().unwrap();
    println!("mem: total {} KB, free {} KB, avail {} KB, buffers {} KB, cached {} KB",
             mem.total, mem.free, mem.avail, mem.buffers, mem.cached);
    println!("swap: total {} KB, free {} KB", mem.swap_total, mem.swap_free);
    #[cfg(not(target_os = "solaris"))] {
        let disk = disk_info().unwrap();
        println!("disk: total {} KB, free {} KB", disk.total, disk.free);
    }
    println!("hostname: {}", hostname().unwrap());
    #[cfg(not(target_os = "windows"))] {
        let t = boottime().unwrap();
        println!("boottime {} sec, {} usec", t.tv_sec, t.tv_usec);
    }
    #[cfg(target_os = "linux")]
    println!("/etc/os-release: {:?}", linux_os_release().unwrap());
}

