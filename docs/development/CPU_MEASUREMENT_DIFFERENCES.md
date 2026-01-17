# CPU Measurement Differences: Stats vs Activity Monitor

## The Discrepancy

**Stats app shows:** 6.4% CPU usage  
**Activity Monitor shows:** >8% CPU usage

## Why They're Different

### 1. **Measurement Method**

**Stats app (using sysinfo):**
- Uses `sysinfo::System::global_cpu_usage()`
- Calculates **system-wide average** across all cores
- Returns a value **0-100%** (total system CPU usage)
- Averages over a **time window** (typically 1-2 seconds)
- **Smooths out spikes** - shows average usage

**Activity Monitor:**
- Shows **per-process CPU usage**
- Can show **>100%** (if process uses multiple cores)
- Updates **very frequently** (multiple times per second)
- Shows **instantaneous values** - captures spikes
- Includes **all work** (user + system/kernel)

### 2. **What's Included**

**Stats app measurement:**
- Measures **system-wide CPU usage** (all processes combined)
- Uses Mach APIs (`host_statistics64`, `host_processor_info`)
- Averages over time window
- May **exclude some kernel/system work** depending on implementation

**Activity Monitor:**
- Shows **per-process CPU usage**
- Includes **all CPU work** for that process:
  - User-space work
  - System/kernel calls
  - Interrupt handling
  - Background threads
- Updates in **real-time** (catches spikes)

### 3. **Time Window / Averaging**

**Stats app:**
- Averages over **1-2 seconds** (refresh interval)
- Smooths out brief spikes
- Shows **steady-state average**

**Activity Monitor:**
- Updates **multiple times per second**
- Shows **instantaneous values**
- Captures **brief spikes** that average out

### 4. **Core Counting**

**Stats app:**
- Shows **system-wide average** (0-100%)
- If 2 cores at 50% each → shows 50% total
- Normalized to total CPU capacity

**Activity Monitor:**
- Shows **per-core usage**
- If process uses 2 cores at 50% each → shows 100% CPU
- Can exceed 100% (e.g., 200% = using 2 cores fully)

## Why Activity Monitor Shows Higher

When Activity Monitor shows **>8%** for Stats app:

1. **Includes all overhead:**
   - UI rendering (gauge updates, animations)
   - Process collection (`refresh_processes()`)
   - WebKit IPC communication
   - Core Animation layer updates
   - System calls (sysctl, Mach APIs)

2. **Captures spikes:**
   - Brief CPU spikes during process refresh
   - WebKit rendering spikes
   - System call overhead

3. **Per-process measurement:**
   - Shows what **Stats app itself** is using
   - Not system-wide average
   - Includes all threads/cores used by Stats

## Why Stats Shows Lower (6.4%)

When Stats shows **6.4%**:

1. **System-wide average:**
   - Measures **total system CPU usage**
   - Not just Stats app's own usage
   - Averages across all processes

2. **Time-averaged:**
   - Smooths out spikes
   - Shows average over 1-2 seconds
   - Brief spikes get averaged down

3. **May exclude some overhead:**
   - UI rendering overhead might not be fully captured
   - WebKit rendering happens in separate process
   - Some system work might be excluded

## The Math

**Example scenario:**
- Stats app uses 8% CPU (what Activity Monitor shows)
- System has 16 cores
- Stats app measurement: 8% / 16 cores = 0.5% per core average
- But Stats shows system-wide average, not per-process

**Actually:**
- Activity Monitor: Shows Stats app using 8% of **one core** (or 8% of total if normalized)
- Stats app: Shows **system-wide** CPU usage (all processes combined) = 6.4%

These are **different measurements**:
- Activity Monitor = "How much CPU is Stats app using?"
- Stats app = "How much CPU is the entire system using?"

## Conclusion

The discrepancy is **expected and normal**:

1. **Different metrics:**
   - Activity Monitor: Per-process CPU usage
   - Stats app: System-wide CPU usage

2. **Different time windows:**
   - Activity Monitor: Instantaneous (catches spikes)
   - Stats app: Averaged (smooths spikes)

3. **Different refresh rates:**
   - Activity Monitor: Very frequent updates
   - Stats app: 1-2 second intervals

4. **Different scope:**
   - Activity Monitor: Just Stats app's own CPU usage
   - Stats app: Total system CPU usage (all processes)

**Both are correct** - they're just measuring different things!
