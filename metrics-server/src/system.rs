use anyhow::{Context, Result};
use std::fs;

use crate::metrics::{CpuInfo, DiskInfo, LoadAvg, MemoryInfo};

pub struct CpuSnapshot {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
}

pub fn read_load_avg() -> Result<LoadAvg> {
    let content = fs::read_to_string("/proc/loadavg").context("read /proc/loadavg")?;
    let mut fields = content.split_whitespace();
    let one: f64 = fields.next().context("missing field 0")?.parse()?;
    let five: f64 = fields.next().context("missing field 1")?.parse()?;
    let fifteen: f64 = fields.next().context("missing field 2")?.parse()?;
    Ok(LoadAvg { one, five, fifteen })
}

pub fn read_memory() -> Result<MemoryInfo> {
    let content = fs::read_to_string("/proc/meminfo").context("read /proc/meminfo")?;
    let mut total_kb: u64 = 0;
    let mut available_kb: u64 = 0;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total_kb = rest.split_whitespace().next().context("MemTotal value")?.parse()?;
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            available_kb = rest.split_whitespace().next().context("MemAvailable value")?.parse()?;
        }
    }
    let used_kb = total_kb.saturating_sub(available_kb);
    let used_pct = if total_kb > 0 { used_kb as f64 / total_kb as f64 * 100.0 } else { 0.0 };
    Ok(MemoryInfo { total_kb, available_kb, used_kb, used_pct })
}

pub fn read_cpu_snapshot() -> Result<CpuSnapshot> {
    let content = fs::read_to_string("/proc/stat").context("read /proc/stat")?;
    let line = content.lines().next().context("empty /proc/stat")?;
    // format: "cpu  user nice system idle iowait irq softirq ..."
    let mut fields = line.split_whitespace();
    fields.next(); // skip "cpu"
    let parse = |f: Option<&str>, name: &str| -> Result<u64> {
        f.context(format!("missing {name}"))?.parse().context(format!("parse {name}"))
    };
    Ok(CpuSnapshot {
        user: parse(fields.next(), "user")?,
        nice: parse(fields.next(), "nice")?,
        system: parse(fields.next(), "system")?,
        idle: parse(fields.next(), "idle")?,
        iowait: parse(fields.next(), "iowait")?,
        irq: parse(fields.next(), "irq")?,
        softirq: parse(fields.next(), "softirq")?,
    })
}

pub fn cpu_diff(prev: &CpuSnapshot, curr: &CpuSnapshot) -> CpuInfo {
    let d = |a: u64, b: u64| b.saturating_sub(a);
    let user = d(prev.user, curr.user);
    let nice = d(prev.nice, curr.nice);
    let system = d(prev.system, curr.system);
    let idle = d(prev.idle, curr.idle);
    let iowait = d(prev.iowait, curr.iowait);
    let irq = d(prev.irq, curr.irq);
    let softirq = d(prev.softirq, curr.softirq);
    let total = user + nice + system + idle + iowait + irq + softirq;
    if total == 0 {
        return CpuInfo { user_pct: 0.0, system_pct: 0.0, idle_pct: 0.0, iowait_pct: 0.0 };
    }
    let pct = |v: u64| v as f64 / total as f64 * 100.0;
    CpuInfo {
        user_pct: pct(user + nice),
        system_pct: pct(system + irq + softirq),
        idle_pct: pct(idle),
        iowait_pct: pct(iowait),
    }
}

pub fn read_disk() -> Result<DiskInfo> {
    let stat = nix::sys::statvfs::statvfs("/").context("statvfs /")?;
    let bsize = stat.block_size() as u64;
    let total_kb = stat.blocks() * bsize / 1024;
    let free_kb = stat.blocks_free() * bsize / 1024;
    let used_kb = total_kb.saturating_sub(free_kb);
    let used_pct = if total_kb > 0 { used_kb as f64 / total_kb as f64 * 100.0 } else { 0.0 };
    Ok(DiskInfo { total_kb, used_kb, free_kb, used_pct })
}
