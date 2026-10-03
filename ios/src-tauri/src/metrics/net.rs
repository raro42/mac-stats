//! Tráfico de red (Wi‑Fi `en*` y datos móviles `pdp_ip*`) a partir de `getifaddrs`.
//!
//! Los contadores de `struct if_data` son de 32 bits y se desbordan a los 4 GB,
//! así que las diferencias se calculan por interfaz con aritmética modular.

use std::collections::HashMap;
use std::ffi::CStr;
use std::time::Instant;

/// Prefijo de `struct if_data` (`<net/if_var.h>`) hasta los contadores de bytes.
/// `libc` solo expone `if_data64`, que no es lo que devuelve `getifaddrs`.
#[repr(C)]
#[allow(dead_code)] // los campos fijan el desplazamiento; solo se leen los bytes
struct IfDataPrefix {
    ifi_type: u8,
    ifi_typelen: u8,
    ifi_physical: u8,
    ifi_addrlen: u8,
    ifi_hdrlen: u8,
    ifi_recvquota: u8,
    ifi_xmitquota: u8,
    ifi_unused1: u8,
    ifi_mtu: u32,
    ifi_metric: u32,
    ifi_baudrate: u32,
    ifi_ipackets: u32,
    ifi_ierrors: u32,
    ifi_opackets: u32,
    ifi_oerrors: u32,
    ifi_collisions: u32,
    ifi_ibytes: u32,
    ifi_obytes: u32,
}

fn is_tracked(name: &str) -> bool {
    name.starts_with("en") || name.starts_with("pdp_ip")
}

/// Bytes recibidos y enviados por interfaz.
fn read_counters() -> HashMap<String, (u32, u32)> {
    let mut out = HashMap::new();
    let mut ifap: *mut libc::ifaddrs = std::ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut ifap) } != 0 {
        return out;
    }
    let mut cur = ifap;
    while !cur.is_null() {
        let ifa = unsafe { &*cur };
        cur = ifa.ifa_next;
        if ifa.ifa_addr.is_null() || ifa.ifa_data.is_null() || ifa.ifa_name.is_null() {
            continue;
        }
        if i32::from(unsafe { (*ifa.ifa_addr).sa_family }) != libc::AF_LINK {
            continue;
        }
        let name = unsafe { CStr::from_ptr(ifa.ifa_name) }.to_string_lossy();
        if !is_tracked(&name) {
            continue;
        }
        let data = unsafe { &*(ifa.ifa_data as *const IfDataPrefix) };
        out.insert(name.into_owned(), (data.ifi_ibytes, data.ifi_obytes));
    }
    unsafe { libc::freeifaddrs(ifap) };
    out
}

/// Suma de las diferencias por interfaz; las interfaces nuevas no aportan hasta
/// la siguiente lectura.
fn delta_bytes(
    prev: &HashMap<String, (u32, u32)>,
    now: &HashMap<String, (u32, u32)>,
) -> (u64, u64) {
    let mut rx = 0u64;
    let mut tx = 0u64;
    for (name, (now_rx, now_tx)) in now {
        if let Some((prev_rx, prev_tx)) = prev.get(name) {
            rx += now_rx.wrapping_sub(*prev_rx) as u64;
            tx += now_tx.wrapping_sub(*prev_tx) as u64;
        }
    }
    (rx, tx)
}

/// Velocidad de bajada y subida en bytes por segundo.
#[derive(Default)]
pub struct NetMeter {
    last: Option<(Instant, HashMap<String, (u32, u32)>)>,
}

impl NetMeter {
    pub fn sample(&mut self) -> Option<(f64, f64)> {
        let now = Instant::now();
        let counters = read_counters();
        let rates = self.last.as_ref().and_then(|(at, prev)| {
            let secs = now.duration_since(*at).as_secs_f64();
            if secs <= 0.0 {
                return None;
            }
            let (rx, tx) = delta_bytes(prev, &counters);
            Some((rx as f64 / secs, tx as f64 / secs))
        });
        self.last = Some((now, counters));
        rates
    }

    /// Olvida la línea base (p. ej. al volver de segundo plano).
    pub fn reset(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: &[(&str, u32, u32)]) -> HashMap<String, (u32, u32)> {
        entries
            .iter()
            .map(|(n, rx, tx)| (n.to_string(), (*rx, *tx)))
            .collect()
    }

    #[test]
    fn sums_deltas_across_interfaces() {
        let prev = map(&[("en0", 1_000, 500), ("pdp_ip0", 10, 20)]);
        let now = map(&[("en0", 1_600, 700), ("pdp_ip0", 15, 25)]);
        assert_eq!(delta_bytes(&prev, &now), (605, 205));
    }

    #[test]
    fn handles_32_bit_wraparound() {
        let prev = map(&[("en0", u32::MAX - 99, 0)]);
        let now = map(&[("en0", 100, 0)]);
        assert_eq!(delta_bytes(&prev, &now), (200, 0));
    }

    #[test]
    fn ignores_interfaces_without_baseline() {
        let prev = map(&[("en0", 100, 100)]);
        let now = map(&[("en0", 150, 120), ("en1", 9_999, 9_999)]);
        assert_eq!(delta_bytes(&prev, &now), (50, 20));
    }

    #[test]
    fn tracks_only_wifi_and_cellular() {
        assert!(is_tracked("en0"));
        assert!(is_tracked("pdp_ip0"));
        assert!(!is_tracked("lo0"));
        assert!(!is_tracked("utun3"));
    }

    #[test]
    fn struct_offsets_match_if_data() {
        assert_eq!(std::mem::offset_of!(IfDataPrefix, ifi_ibytes), 40);
        assert_eq!(std::mem::offset_of!(IfDataPrefix, ifi_obytes), 44);
    }
}
