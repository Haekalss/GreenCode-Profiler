//! Process-Level Energy Engine in Rust
//! Mengukur konsumsi energi perangkat lunak murni pada tingkat proses (PID)
//! menggunakan Windows NT Kernel API (GetProcessTimes) dan /proc/stat di Linux.
//! Bebas dari distorsi waktu tunggu jaringan (I/O Wait Confounding).

use std::collections::HashMap;

#[cfg(windows)]
mod platform {
    type HANDLE = *mut std::ffi::c_void;
    type BOOL = i32;
    type DWORD = u32;

    #[repr(C)]
    #[derive(Default, Copy, Clone)]
    #[allow(non_snake_case)]
    pub struct FILETIME {
        pub dwLowDateTime: DWORD,
        pub dwHighDateTime: DWORD,
    }

    extern "system" {
        fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
        fn CloseHandle(hObject: HANDLE) -> BOOL;
        fn GetProcessTimes(
            hProcess: HANDLE,
            lpCreationTime: *mut FILETIME,
            lpExitTime: *mut FILETIME,
            lpKernelTime: *mut FILETIME,
            lpUserTime: *mut FILETIME,
        ) -> BOOL;
    }

    const PROCESS_QUERY_LIMITED_INFORMATION: DWORD = 0x1000;
    const PROCESS_QUERY_INFORMATION: DWORD = 0x0400;

    fn filetime_to_sec(ft: &FILETIME) -> f64 {
        let raw = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);
        raw as f64 / 10_000_000.0 // 100-nanosecond intervals to seconds
    }

    pub fn get_cpu_times(pid: u32) -> Option<(f64, f64)> {
        unsafe {
            let mut handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
            }
            if handle.is_null() {
                return None;
            }
            let mut creation = FILETIME::default();
            let mut exit = FILETIME::default();
            let mut kernel = FILETIME::default();
            let mut user = FILETIME::default();

            let ok = GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user);
            CloseHandle(handle);

            if ok != 0 {
                Some((filetime_to_sec(&user), filetime_to_sec(&kernel)))
            } else {
                None
            }
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use std::fs;

    pub fn get_cpu_times(pid: u32) -> Option<(f64, f64)> {
        let stat_path = format!("/proc/{}/stat", pid);
        if let Ok(content) = fs::read_to_string(stat_path) {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 15 {
                let utime: f64 = parts[13].parse().unwrap_or(0.0);
                let stime: f64 = parts[14].parse().unwrap_or(0.0);
                let ticks_per_sec = 100.0; // Standard sysconf(_SC_CLK_TCK)
                return Some((utime / ticks_per_sec, stime / ticks_per_sec));
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub user_sec: f64,
    pub system_sec: f64,
}

pub struct ProcessEnergyEngine {
    pub pids: Vec<u32>,
    pub cpu_tdp_watts: f64,
}

impl ProcessEnergyEngine {
    pub fn new(pids: Vec<u32>, cpu_tdp_watts: f64) -> Self {
        Self {
            pids,
            cpu_tdp_watts,
        }
    }

    /// Mengambil snapshot CPU user dan kernel dari semua target PID
    pub fn capture_snapshot(&self) -> HashMap<u32, ProcessSnapshot> {
        let mut map = HashMap::new();
        for &pid in &self.pids {
            if let Some((u, s)) = platform::get_cpu_times(pid) {
                map.insert(
                    pid,
                    ProcessSnapshot {
                        pid,
                        user_sec: u,
                        system_sec: s,
                    },
                );
            }
        }
        map
    }

    /// Menghitung selisih waktu aktif CPU murni dan konsumsi energi dalam Joule
    pub fn calculate_energy(
        &self,
        start: &HashMap<u32, ProcessSnapshot>,
        end: &HashMap<u32, ProcessSnapshot>,
        wall_clock_sec: f64,
    ) -> (f64, f64, f64) {
        let mut delta_user = 0.0;
        let mut delta_sys = 0.0;

        for (pid, end_snap) in end {
            if let Some(start_snap) = start.get(pid) {
                let u = (end_snap.user_sec - start_snap.user_sec).max(0.0);
                let s = (end_snap.system_sec - start_snap.system_sec).max(0.0);
                delta_user += u;
                delta_sys += s;
            }
        }

        let total_active_cpu = delta_user + delta_sys;
        let effective_wall = wall_clock_sec.max(0.0001);

        // Model Energi Green Profiler: Proporsional terhadap utilitas CPU aktif murni
        // Menghilangkan distorsi I/O wait latensi jaringan
        let cpu_utilization = (total_active_cpu / effective_wall).min(1.0);
        let energy_joules = self.cpu_tdp_watts * cpu_utilization * total_active_cpu;

        (total_active_cpu, energy_joules, cpu_utilization)
    }
}
