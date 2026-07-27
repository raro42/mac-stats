use macsmc::{DataValue, Smc};

const MIN_VALID_TEMPERATURE_C: f32 = 10.0;
const MAX_VALID_TEMPERATURE_C: f32 = 120.0;

// Apple changes SMC sensor keys between SoC generations. Keep these lists model-scoped so a
// similarly named sensor from another generation is never mistaken for a CPU temperature.
const M3_CPU_TEMPERATURE_KEYS: &[&str] = &["Tf04", "Tf09", "Tf0A", "Tf0B", "Tf0D", "Tf0E"];
const M4_CPU_TEMPERATURE_KEYS: &[&str] = &[
    "Te05", "Te0S", "Te09", "Te0H", // efficiency clusters
    "Tp01", "Tp05", "Tp09", "Tp0D", "Tp0V", "Tp0Y", "Tp0b", "Tp0e", // performance clusters
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppleSiliconGeneration {
    M3,
    M4,
    Other,
}

#[derive(Debug)]
pub(crate) struct CpuTemperatureReading {
    pub(crate) value_celsius: f32,
    pub(crate) keys: Vec<String>,
}

#[derive(Default)]
pub(crate) struct SmcTemperatureReader {
    cached_keys: Vec<String>,
}

impl SmcTemperatureReader {
    pub(crate) fn read(&mut self, smc: &mut Smc, chip_info: &str) -> Option<CpuTemperatureReading> {
        match apple_silicon_generation(chip_info) {
            AppleSiliconGeneration::M3 => self.read_raw_keys(smc, M3_CPU_TEMPERATURE_KEYS),
            AppleSiliconGeneration::M4 => self.read_raw_keys(smc, M4_CPU_TEMPERATURE_KEYS),
            AppleSiliconGeneration::Other => read_standard_temperature(smc),
        }
    }

    fn read_raw_keys(
        &mut self,
        smc: &mut Smc,
        generation_keys: &[&str],
    ) -> Option<CpuTemperatureReading> {
        if !self.cached_keys.is_empty() {
            let cached_keys = self.cached_keys.clone();
            let cached_key_refs: Vec<&str> = cached_keys.iter().map(String::as_str).collect();
            if let Some(reading) = scan_temperature_keys(smc, &cached_key_refs) {
                return Some(reading);
            }
            self.cached_keys.clear();
        }

        let reading = scan_temperature_keys(smc, generation_keys)?;
        self.cached_keys.clone_from(&reading.keys);
        Some(reading)
    }
}

fn apple_silicon_generation(chip_info: &str) -> AppleSiliconGeneration {
    if chip_info.contains("Apple M4") {
        AppleSiliconGeneration::M4
    } else if chip_info.contains("Apple M3") {
        AppleSiliconGeneration::M3
    } else {
        AppleSiliconGeneration::Other
    }
}

fn read_standard_temperature(smc: &mut Smc) -> Option<CpuTemperatureReading> {
    let temperatures = smc.cpu_temperature().ok()?;
    let die: f64 = temperatures.die.into();
    let proximity: f64 = temperatures.proximity.into();
    let value_celsius = [die as f32, proximity as f32]
        .into_iter()
        .find(|value| is_valid_temperature(*value))?;

    Some(CpuTemperatureReading {
        value_celsius,
        keys: vec!["macsmc-standard".to_string()],
    })
}

fn scan_temperature_keys(smc: &mut Smc, candidate_keys: &[&str]) -> Option<CpuTemperatureReading> {
    let data = smc.all_data().ok()?;
    let mut readings = Vec::new();

    for item in data.flatten() {
        if !candidate_keys.contains(&item.key.as_str()) {
            continue;
        }
        if let Ok(Some(DataValue::Float(value))) = item.value {
            if is_valid_temperature(value) {
                readings.push((item.key, value));
                if readings.len() == candidate_keys.len() {
                    break;
                }
            }
        }
    }

    average_readings(readings)
}

fn average_readings(readings: Vec<(String, f32)>) -> Option<CpuTemperatureReading> {
    if readings.is_empty() {
        return None;
    }

    let value_celsius =
        readings.iter().map(|(_, value)| value).sum::<f32>() / readings.len() as f32;
    let keys = readings.into_iter().map(|(key, _)| key).collect();
    Some(CpuTemperatureReading {
        value_celsius,
        keys,
    })
}

fn is_valid_temperature(value: f32) -> bool {
    value.is_finite() && (MIN_VALID_TEMPERATURE_C..=MAX_VALID_TEMPERATURE_C).contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_m4_variants() {
        assert_eq!(
            apple_silicon_generation("Apple M4 · 10 cores"),
            AppleSiliconGeneration::M4
        );
        assert_eq!(
            apple_silicon_generation("Apple M4 Pro · 14 cores"),
            AppleSiliconGeneration::M4
        );
        assert_eq!(
            apple_silicon_generation("Apple M4 Max · 16 cores"),
            AppleSiliconGeneration::M4
        );
    }

    #[test]
    fn keeps_m3_and_other_generations_separate() {
        assert_eq!(
            apple_silicon_generation("Apple M3 Max · 16 cores"),
            AppleSiliconGeneration::M3
        );
        assert_eq!(
            apple_silicon_generation("Apple M2 · 8 cores"),
            AppleSiliconGeneration::Other
        );
    }

    #[test]
    fn averages_discovered_cpu_sensors() {
        let reading = average_readings(vec![
            ("Te05".to_string(), 40.0),
            ("Tp01".to_string(), 50.0),
            ("Tp05".to_string(), 60.0),
        ])
        .expect("valid readings");

        assert_eq!(reading.value_celsius, 50.0);
        assert_eq!(reading.keys, ["Te05", "Tp01", "Tp05"]);
    }

    #[test]
    fn rejects_implausible_temperatures() {
        assert!(!is_valid_temperature(0.0));
        assert!(!is_valid_temperature(121.0));
        assert!(!is_valid_temperature(f32::NAN));
        assert!(is_valid_temperature(42.5));
    }
}
