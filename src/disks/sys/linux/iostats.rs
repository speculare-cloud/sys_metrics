use crate::disks::IoStats;

use std::io::Error;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

/// Get basic [IoStats] info for each disks/partitions.
///
/// It only contains the `device_name` and the number of bytes `read`/`wrtn`.
///
/// On linux it will get them from `/proc/diskstats`.
///
/// On macOS it will use unsafes call to multiple OSX specific functions.
///
/// [IoStats]: ../disks/struct.IoStats.html
pub fn get_iostats() -> Result<Vec<IoStats>, Error> {
    let mut viostats: Vec<IoStats> = Vec::new();
    let file = File::open("/proc/diskstats")?;
    let mut file = BufReader::with_capacity(2048, file);

    let mut line = String::with_capacity(512);
    while file.read_line(&mut line)? != 0 {
        // See (https://www.kernel.org/doc/Documentation/ABI/testing/procfs-diskstats)
        let mut fields = line.split_whitespace();
        let name = fields.nth(2).unwrap();
        let byte_r = fields.nth(2).unwrap();
        let byte_w = fields.nth(3).unwrap();
        // If we have less than 14 fields, we're missing some data
        if fields.count() < 4 {
            line.clear();
            continue;
        }
        viostats.push(IoStats {
            device_name: name.to_owned(),
            bytes_read: byte_r.parse::<i64>().unwrap() * 512,
            bytes_wrtn: byte_w.parse::<i64>().unwrap() * 512,
        });
        line.clear();
    }

    Ok(viostats)
}

/// Get basic [IoStats] info for each physical disks.
///
/// On linux it will get them from `/proc/diskstats` and filter the result based on the access to their `/sys/block/{}`.
///
/// On macOS it will use unsafes call and detect if the disk is marked as Removable, if it's not... it's a physical device
///
/// [IoStats]: ../struct.IoStats.html
pub fn get_iostats_physical() -> Result<Vec<IoStats>, Error> {
    let mut viostats: Vec<IoStats> = Vec::new();
    let file = File::open("/proc/diskstats")?;
    let mut file = BufReader::with_capacity(2048, file);

    let mut line = String::with_capacity(512);
    while file.read_line(&mut line)? != 0 {
        let mut fields = line.split_whitespace();
        let name = fields.nth(2).unwrap();
        let byte_r = fields.nth(2).unwrap();
        let byte_w = fields.nth(3).unwrap();
        if fields.count() < 4 {
            line.clear();
            continue;
        }
        // Based on the sysstat code:
        // https://github.com/sysstat/sysstat/blob/1c711c1fd03ac638cfc1b25cdf700625c173fd2c/common.c#L200
        // Some devices may have a slash in their name (eg. cciss/c0d0...) so replace them with `!`
        let path = std::ffi::CString::new(format!("/sys/block/{}/device", name.replace("/", "!")))?;
        if unsafe { libc::access(path.as_ptr(), libc::F_OK) } != 0 {
            line.clear();
            continue;
        }
        viostats.push(IoStats {
            device_name: name.to_owned(),
            bytes_read: byte_r.parse::<i64>().unwrap() * 512,
            bytes_wrtn: byte_w.parse::<i64>().unwrap() * 512,
        });
        line.clear();
    }

    Ok(viostats)
}
