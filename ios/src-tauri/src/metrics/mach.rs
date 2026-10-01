//! Lecturas de Mach vía `libc`: CPU total, memoria del sistema y memoria de la app.
//!
//! Son APIs públicas de Darwin que funcionan igual en iOS y macOS (en macOS solo
//! se usan para `cargo test`).

use std::mem::{offset_of, size_of, MaybeUninit};

/// Ticks acumulados de todos los núcleos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CpuTicks {
    pub busy: u64,
    pub total: u64,
}

impl CpuTicks {
    /// Porcentaje de uso entre `prev` y `self`; `None` si los contadores no avanzaron
    /// o retrocedieron (desbordamiento o reinicio).
    pub fn usage_since(&self, prev: &CpuTicks) -> Option<f32> {
        let total = self.total.checked_sub(prev.total)?;
        let busy = self.busy.checked_sub(prev.busy)?;
        if total == 0 || busy > total {
            return None;
        }
        Some((busy as f64 / total as f64 * 100.0) as f32)
    }
}

#[allow(deprecated)] // libc recomienda `mach2`, pero no compensa otra dependencia por esto.
fn task_self() -> libc::mach_port_t {
    unsafe { libc::mach_task_self() }
}

#[allow(deprecated)]
fn host_self() -> libc::mach_port_t {
    unsafe { libc::mach_host_self() }
}

/// Suma de ticks (user, system, nice, idle) de todos los núcleos.
pub fn cpu_ticks() -> Option<CpuTicks> {
    let mut cpu_count: libc::natural_t = 0;
    let mut info: libc::processor_info_array_t = std::ptr::null_mut();
    let mut info_count: libc::mach_msg_type_number_t = 0;
    let kr = unsafe {
        libc::host_processor_info(
            host_self(),
            libc::PROCESSOR_CPU_LOAD_INFO,
            &mut cpu_count,
            &mut info,
            &mut info_count,
        )
    };
    if kr != libc::KERN_SUCCESS || info.is_null() {
        return None;
    }

    let loads = unsafe {
        std::slice::from_raw_parts(
            info as *const libc::processor_cpu_load_info,
            cpu_count as usize,
        )
    };
    let mut ticks = CpuTicks::default();
    for load in loads {
        let user = load.cpu_ticks[libc::CPU_STATE_USER as usize] as u64;
        let system = load.cpu_ticks[libc::CPU_STATE_SYSTEM as usize] as u64;
        let nice = load.cpu_ticks[libc::CPU_STATE_NICE as usize] as u64;
        let idle = load.cpu_ticks[libc::CPU_STATE_IDLE as usize] as u64;
        ticks.busy += user + system + nice;
        ticks.total += user + system + nice + idle;
    }

    // El kernel reserva el array en nuestro espacio de direcciones: hay que liberarlo.
    unsafe {
        libc::vm_deallocate(
            task_self(),
            info as libc::vm_address_t,
            (info_count as usize * size_of::<libc::integer_t>()) as libc::vm_size_t,
        );
    }
    Some(ticks)
}

fn page_size() -> u64 {
    let size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if size > 0 {
        size as u64
    } else {
        16_384
    }
}

/// Memoria usada por el sistema, con el mismo criterio que Monitor de Actividad:
/// memoria de apps (internal - purgeable) + cableada + comprimida.
pub fn system_memory_used() -> Option<u64> {
    let mut stats = MaybeUninit::<libc::vm_statistics64>::zeroed();
    let mut count = libc::HOST_VM_INFO64_COUNT;
    let kr = unsafe {
        libc::host_statistics64(
            host_self(),
            libc::HOST_VM_INFO64,
            stats.as_mut_ptr() as libc::host_info64_t,
            &mut count,
        )
    };
    if kr != libc::KERN_SUCCESS {
        return None;
    }
    let s = unsafe { stats.assume_init() };
    let app = (s.internal_page_count as u64).saturating_sub(s.purgeable_count as u64);
    let pages = app + s.wire_count as u64 + s.compressor_page_count as u64;
    Some(pages * page_size())
}

/// `struct task_vm_info` de `<mach/task_info.h>` hasta la revisión 2.
#[repr(C)]
#[derive(Default)]
struct TaskVmInfo {
    virtual_size: u64,
    region_count: i32,
    page_size: i32,
    resident_size: u64,
    resident_size_peak: u64,
    device: u64,
    device_peak: u64,
    internal: u64,
    internal_peak: u64,
    external: u64,
    external_peak: u64,
    reusable: u64,
    reusable_peak: u64,
    purgeable_volatile_pmap: u64,
    purgeable_volatile_resident: u64,
    purgeable_volatile_virtual: u64,
    compressed: u64,
    compressed_peak: u64,
    compressed_lifetime: u64,
    // Revisión 1
    phys_footprint: u64,
    // Revisión 2
    min_address: u64,
    max_address: u64,
}

const TASK_VM_INFO: libc::task_flavor_t = 22;

/// Memoria que iOS le atribuye a esta app (`phys_footprint`, la que usa para
/// decidir si la cierra). Si el kernel no rellena la revisión 1, usa la residente.
pub fn app_footprint() -> Option<u64> {
    let mut info = TaskVmInfo::default();
    let mut count = (size_of::<TaskVmInfo>() / size_of::<libc::natural_t>())
        as libc::mach_msg_type_number_t;
    let kr = unsafe {
        libc::task_info(
            task_self(),
            TASK_VM_INFO,
            &mut info as *mut TaskVmInfo as libc::task_info_t,
            &mut count,
        )
    };
    if kr != libc::KERN_SUCCESS {
        return None;
    }
    let rev1_count = (offset_of!(TaskVmInfo, phys_footprint) + size_of::<u64>())
        / size_of::<libc::natural_t>();
    if (count as usize) >= rev1_count && info.phys_footprint > 0 {
        Some(info.phys_footprint)
    } else {
        Some(info.resident_size)
    }
}

#[cfg(target_os = "ios")]
extern "C" {
    /// `<os/proc.h>`: memoria que la app aún puede usar antes de que iOS la cierre.
    fn os_proc_available_memory() -> libc::size_t;
}

/// Margen de memoria de la app. `None` fuera de iOS o en el simulador (devuelve 0).
pub fn app_available_memory() -> Option<u64> {
    #[cfg(target_os = "ios")]
    {
        let available = unsafe { os_proc_available_memory() } as u64;
        if available > 0 {
            return Some(available);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_from_tick_deltas() {
        let prev = CpuTicks { busy: 100, total: 1_000 };
        let now = CpuTicks { busy: 150, total: 1_100 };
        assert_eq!(now.usage_since(&prev), Some(50.0));
    }

    #[test]
    fn usage_is_none_without_progress_or_on_rollback() {
        let prev = CpuTicks { busy: 100, total: 1_000 };
        assert_eq!(prev.usage_since(&prev), None);
        let rolled = CpuTicks { busy: 10, total: 50 };
        assert_eq!(rolled.usage_since(&prev), None);
    }

    #[test]
    fn task_vm_info_layout_matches_kernel_revision_2() {
        // 42 natural_t = TASK_VM_INFO_REV2_COUNT en el SDK.
        assert_eq!(size_of::<TaskVmInfo>() / size_of::<libc::natural_t>(), 42);
        assert_eq!(offset_of!(TaskVmInfo, phys_footprint), 144);
    }

    #[test]
    fn live_readings_are_sane() {
        let a = cpu_ticks().expect("host_processor_info");
        assert!(a.total >= a.busy);
        let used = system_memory_used().expect("host_statistics64");
        assert!(used > 0);
        let footprint = app_footprint().expect("task_info");
        assert!(footprint > 0);
    }
}
