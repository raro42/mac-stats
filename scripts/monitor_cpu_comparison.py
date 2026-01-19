#!/usr/bin/env python3
"""
CPU Usage Comparison Monitor
Starts both apps, waits for stabilization, then monitors CPU usage

Usage:
    python3 monitor_cpu_comparison.py [duration] [interval] [warmup] [--auto-kill] [--keep-running]
    
    duration: How long to monitor after warmup (seconds, default: 60)
    interval: Sampling interval (seconds, default: 1)
    warmup: Warmup/stabilization period (seconds, default: 30)
    --auto-kill: Automatically kill existing processes (non-interactive)
    --keep-running: Keep apps running after monitoring (default: ask)
"""

import subprocess
import time
import sys
import os
import signal
from datetime import datetime
from collections import defaultdict

def find_processes_by_pattern(pattern):
    """Find all processes matching the pattern"""
    try:
        result = subprocess.run(
            ['pgrep', '-fl', pattern],
            capture_output=True,
            text=True,
            check=False
        )
        processes = []
        for line in result.stdout.strip().split('\n'):
            if line:
                parts = line.split(None, 1)
                if len(parts) >= 2:
                    pid = parts[0]
                    cmd = parts[1]
                    processes.append({'pid': pid, 'cmd': cmd})
        return processes
    except Exception as e:
        return []

def kill_existing_processes(pattern, app_name):
    """Kill existing processes matching pattern"""
    procs = find_processes_by_pattern(pattern)
    if not procs:
        return []
    
    killed_pids = []
    failed_pids = []
    for proc in procs:
        pid = proc['pid']
        try:
            os.kill(int(pid), signal.SIGTERM)
            killed_pids.append(pid)
            print(f"  ✓ Killed existing {app_name} process (PID {pid})")
        except ProcessLookupError:
            # Process already dead
            pass
        except PermissionError:
            failed_pids.append(pid)
            print(f"  ⚠ Permission denied killing PID {pid} (may need sudo)")
        except Exception as e:
            failed_pids.append(pid)
            print(f"  ⚠ Could not kill PID {pid}: {e}")
    
    # Wait a moment for processes to terminate
    if killed_pids:
        time.sleep(2)
    
    # If some processes couldn't be killed, try SIGKILL as last resort
    if failed_pids:
        print(f"  Attempting force kill for {len(failed_pids)} process(es)...")
        for pid in failed_pids[:]:  # Copy list to iterate safely
            try:
                os.kill(int(pid), signal.SIGKILL)
                killed_pids.append(pid)
                failed_pids.remove(pid)
                print(f"  ✓ Force killed PID {pid}")
            except:
                pass
        
        if failed_pids:
            print(f"  ⚠ Warning: {len(failed_pids)} process(es) could not be killed")
            print(f"     You may need to kill them manually or use sudo")
    
    return killed_pids

def start_mac_stats(build_type="release"):
    """Start mac_stats app and return main PID"""
    print("Starting mac_stats...")
    
    # Determine path
    if build_type == "debug":
        cmd = ["cargo", "run", "--manifest-path", "src-tauri/Cargo.toml", "--", "--frequency"]
        cwd = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    else:
        # Release build
        script_dir = os.path.dirname(os.path.abspath(__file__))
        project_root = os.path.dirname(script_dir)
        exe_path = os.path.join(project_root, "src-tauri", "target", "release", "mac_stats")
        if not os.path.exists(exe_path):
            print(f"  ⚠ Release build not found at {exe_path}")
            print("  Falling back to debug build...")
            return start_mac_stats("debug")
        cmd = [exe_path, "--frequency"]
        cwd = project_root
    
    try:
        # Start process in background
        process = subprocess.Popen(
            cmd,
            cwd=cwd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True
        )
        pid = process.pid
        print(f"  ✓ Started mac_stats (PID {pid})")
        return pid
    except Exception as e:
        print(f"  ✗ Failed to start mac_stats: {e}")
        return None

def start_stats_app():
    """Start Stats.app and return main PID"""
    print("Starting Stats.app...")
    
    try:
        # Use open command to launch Stats.app
        result = subprocess.run(
            ["open", "-a", "Stats"],
            check=True,
            capture_output=True
        )
        
        # Wait a moment for app to start
        time.sleep(2)
        
        # Find the main Stats process
        # Stats.app main process is typically the one with the longest command line
        procs = find_processes_by_pattern("Stats")
        stats_procs = [p for p in procs if "Stats.app/Contents/MacOS/Stats" in p['cmd']]
        
        if stats_procs:
            pid = stats_procs[0]['pid']
            print(f"  ✓ Started Stats.app (PID {pid})")
            return int(pid)
        else:
            print("  ⚠ Stats.app started but main PID not found")
            # Return first Stats process as fallback
            if procs:
                return int(procs[0]['pid'])
            return None
    except Exception as e:
        print(f"  ✗ Failed to start Stats.app: {e}")
        return None

def get_process_children(parent_pid, seen=None):
    """Get all child processes recursively"""
    if seen is None:
        seen = set()
    
    children = []
    try:
        result = subprocess.run(
            ['pgrep', '-P', str(parent_pid)],
            capture_output=True,
            text=True,
            check=False
        )
        for pid_str in result.stdout.strip().split('\n'):
            if pid_str:
                pid = int(pid_str.strip())
                if pid not in seen:
                    seen.add(pid)
                    children.append(pid)
                    # Recursively get grandchildren
                    grandchildren = get_process_children(pid, seen)
                    children.extend(grandchildren)
    except Exception as e:
        pass
    return children

def get_all_related_pids(main_pid):
    """Get main PID and all child PIDs"""
    if main_pid is None:
        return []
    
    all_pids = [main_pid]
    children = get_process_children(main_pid)
    all_pids.extend(children)
    return all_pids

def parse_cpu_time(cpu_time_str):
    """Parse CPU time string to seconds
    
    macOS formats:
    - MM:SS.mm (e.g., "0:01.29" = 1.29 seconds)
    - HH:MM:SS (e.g., "1:23:45" = 1 hour, 23 minutes, 45 seconds)
    - DD-HH:MM:SS (e.g., "2-12:34:56" = 2 days, 12 hours, 34 minutes, 56 seconds)
    """
    try:
        cpu_time_str = cpu_time_str.strip()
        
        # Handle day format: DD-HH:MM:SS
        if '-' in cpu_time_str and cpu_time_str.count('-') == 1:
            day_part, time_part = cpu_time_str.split('-', 1)
            days = int(day_part)
            time_seconds = parse_cpu_time(time_part)
            return days * 86400 + time_seconds
        
        # Split by colon
        parts = cpu_time_str.split(':')
        
        if len(parts) == 3:
            # Format: HH:MM:SS
            hours = int(parts[0])
            minutes = int(parts[1])
            seconds = float(parts[2])  # May have milliseconds
            return hours * 3600 + minutes * 60 + seconds
        elif len(parts) == 2:
            # Format: MM:SS or MM:SS.mm
            minutes = int(parts[0])
            seconds = float(parts[1])  # Handles both "45" and "45.67"
            return minutes * 60 + seconds
        else:
            # Single number (seconds)
            return float(cpu_time_str)
    except Exception as e:
        # Debug: print error if parsing fails
        # print(f"[DEBUG] Failed to parse CPU time '{cpu_time_str}': {e}")
        pass
    return 0.0

def get_cpu_time(pid):
    """Get cumulative CPU time for a process in seconds"""
    try:
        result = subprocess.run(
            ['ps', '-p', str(pid), '-o', 'time='],
            capture_output=True,
            text=True,
            check=False
        )
        if result.stdout.strip():
            cpu_time_str = result.stdout.strip()
            return parse_cpu_time(cpu_time_str)
    except:
        pass
    return 0.0

def get_cpu_usage(pid):
    """Get CPU usage percentage and other metrics for a process"""
    try:
        result = subprocess.run(
            ['ps', '-p', str(pid), '-o', '%cpu=,rss=,%mem=,time='],
            capture_output=True,
            text=True,
            check=False
        )
        if result.stdout.strip():
            parts = result.stdout.strip().split()
            if len(parts) >= 4:
                cpu = float(parts[0])
                rss_kb = int(parts[1])
                mem = float(parts[2])
                cpu_time_str = parts[3]
                cpu_time_seconds = parse_cpu_time(cpu_time_str)
                return {
                    'cpu': cpu,
                    'rss_mb': rss_kb / 1024.0,
                    'mem': mem,
                    'cpu_time_seconds': cpu_time_seconds
                }
    except Exception as e:
        pass
    return None

def capture_initial_cpu_times(pids, debug=False):
    """Capture initial CPU times for all PIDs"""
    initial_times = {}
    for pid in pids:
        # Check if process exists
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            if debug:
                print(f"    [DEBUG] PID {pid} does not exist, skipping")
            continue
        
        cpu_time = get_cpu_time(pid)
        initial_times[pid] = cpu_time
        if debug:
            print(f"    [DEBUG] Captured initial CPU time for PID {pid}: {cpu_time:.2f}s")
    return initial_times

def get_total_cpu_metrics(pids, initial_times, debug=False):
    """Get total CPU metrics for a set of PIDs"""
    total_cpu = 0.0
    total_rss = 0.0
    total_mem = 0.0
    total_cpu_time = 0.0
    total_cpu_time_delta = 0.0
    valid_pids = []
    
    for pid in pids:
        # Check if process still exists
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            if debug:
                print(f"    [DEBUG] PID {pid} no longer exists, skipping")
            continue
        except PermissionError:
            # Process exists but we can't signal it (might be owned by root)
            pass
        
        metrics = get_cpu_usage(pid)
        if metrics:
            total_cpu += metrics['cpu']
            total_rss += metrics['rss_mb']
            total_mem += metrics['mem']
            total_cpu_time += metrics['cpu_time_seconds']
            
            # Calculate delta from initial time
            if pid in initial_times:
                delta = metrics['cpu_time_seconds'] - initial_times[pid]
                total_cpu_time_delta += delta
                if debug:
                    print(f"    [DEBUG] PID {pid}: initial={initial_times[pid]:.2f}s, current={metrics['cpu_time_seconds']:.2f}s, delta={delta:.2f}s")
            else:
                if debug:
                    print(f"    [DEBUG] PID {pid}: no initial time recorded")
            
            valid_pids.append(pid)
        else:
            if debug:
                print(f"    [DEBUG] PID {pid}: could not get metrics")
    
    return {
        'cpu': total_cpu,
        'rss_mb': total_rss,
        'mem': total_mem,
        'cpu_time_seconds': total_cpu_time,
        'cpu_time_delta': total_cpu_time_delta,
        'process_count': len(valid_pids),
        'pids': valid_pids
    }

def format_cpu_time(seconds):
    """Format CPU time in seconds to readable format"""
    if seconds < 60:
        return f"{seconds:.1f}s"
    elif seconds < 3600:
        minutes = int(seconds // 60)
        secs = seconds % 60
        return f"{minutes}m {secs:.1f}s"
    else:
        hours = int(seconds // 3600)
        minutes = int((seconds % 3600) // 60)
        secs = seconds % 60
        return f"{hours}h {minutes}m {secs:.1f}s"

def calculate_reduction_percentage(mac_stats_value, stats_value):
    """Calculate percentage reduction/increase"""
    if stats_value == 0:
        if mac_stats_value == 0:
            return 0.0
        return float('inf')
    if mac_stats_value == 0:
        return 100.0
    
    if mac_stats_value < stats_value:
        reduction = ((stats_value - mac_stats_value) / stats_value) * 100
        return reduction
    else:
        increase = ((mac_stats_value - stats_value) / stats_value) * 100
        return -increase

def print_header():
    """Print monitoring header"""
    print("\n" + "="*80)
    print("  CPU Usage Comparison Monitor")
    print("="*80)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print()

def warmup_period(seconds, mac_stats_pids, stats_pids):
    """Wait for apps to stabilize"""
    print(f"\n⏱️  Warmup period: {seconds} seconds")
    print("   Apps are initializing and stabilizing...")
    print()
    
    start_time = time.time()
    while True:
        elapsed = time.time() - start_time
        remaining = seconds - elapsed
        
        if remaining <= 0:
            break
        
        # Check if processes are still running
        if mac_stats_pids:
            main_pid = mac_stats_pids[0]
            try:
                os.kill(main_pid, 0)  # Check if process exists
            except ProcessLookupError:
                print(f"\n  ✗ mac_stats process (PID {main_pid}) died during warmup!")
                return False
        
        if stats_pids:
            main_pid = stats_pids[0]
            try:
                os.kill(main_pid, 0)
            except ProcessLookupError:
                print(f"\n  ✗ Stats process (PID {main_pid}) died during warmup!")
                return False
        
        # Print countdown
        if remaining > 5:
            print(f"   {int(remaining)} seconds remaining...", end='\r')
        else:
            print(f"   {remaining:.1f} seconds remaining...", end='\r')
        
        time.sleep(0.5)
    
    print("\n   ✓ Warmup complete!")
    return True

def print_status_line(timestamp, mac_stats, stats_app, elapsed_seconds=0):
    """Print a single status line"""
    mac_stats_cpu_time_delta = mac_stats.get('cpu_time_delta', 0.0)
    stats_cpu_time_delta = stats_app.get('cpu_time_delta', 0.0)
    
    mac_stats_procs = mac_stats.get('process_count', 0)
    stats_procs = stats_app.get('process_count', 0)
    
    # Calculate percentage reduction based on CPU time
    reduction_pct = calculate_reduction_percentage(mac_stats_cpu_time_delta, stats_cpu_time_delta)
    
    # Format CPU time deltas
    mac_stats_time_str = format_cpu_time(mac_stats_cpu_time_delta) if mac_stats_cpu_time_delta > 0 else "0.0s"
    stats_time_str = format_cpu_time(stats_cpu_time_delta) if stats_cpu_time_delta > 0 else "0.0s"
    
    # Build comparison message
    if mac_stats_cpu_time_delta < stats_cpu_time_delta and stats_cpu_time_delta > 0:
        if reduction_pct == float('inf'):
            comparison = "✓ mac_stats uses LESS (Stats=0s)"
        else:
            comparison = f"✓ -{reduction_pct:.1f}% CPU time"
    elif mac_stats_cpu_time_delta > stats_cpu_time_delta:
        comparison = f"✗ +{abs(reduction_pct):.1f}% CPU time"
    else:
        comparison = "≈ EQUAL"
    
    proc_detail = f"({mac_stats_procs} procs)" if mac_stats_procs > 1 else ""
    stats_detail = f"({stats_procs} procs)" if stats_procs > 1 else ""
    
    print(f"{timestamp} | mac_stats: {mac_stats_time_str:>10} {proc_detail:12} | "
          f"Stats: {stats_time_str:>10} {stats_detail:12} | {comparison}")

def generate_summary_report(data, output_file, mac_stats_pids, stats_pids, warmup_seconds, duration_seconds):
    """Generate a summary report"""
    if not data:
        return
    
    # Get final CPU time deltas
    final_mac_stats = data[-1]['mac_stats']
    final_stats = data[-1]['stats']
    mac_stats_cpu_time_delta = final_mac_stats.get('cpu_time_delta', 0.0)
    stats_cpu_time_delta = final_stats.get('cpu_time_delta', 0.0)
    
    # Calculate average CPU% per second
    if duration_seconds > 0:
        mac_stats_avg_cpu_percent = (mac_stats_cpu_time_delta / duration_seconds) * 100
        stats_avg_cpu_percent = (stats_cpu_time_delta / duration_seconds) * 100
    else:
        mac_stats_avg_cpu_percent = 0.0
        stats_avg_cpu_percent = 0.0
    
    # Calculate percentage reduction
    reduction_pct = calculate_reduction_percentage(mac_stats_cpu_time_delta, stats_cpu_time_delta)
    
    # Build reduction message
    if mac_stats_cpu_time_delta < stats_cpu_time_delta and stats_cpu_time_delta > 0:
        if reduction_pct == float('inf'):
            reduction_msg = "✓ mac_stats uses LESS CPU time (Stats uses 0s)"
        else:
            reduction_msg = f"✓ mac_stats uses {reduction_pct:.1f}% LESS CPU time than Stats"
    elif mac_stats_cpu_time_delta > stats_cpu_time_delta:
        reduction_msg = f"✗ mac_stats uses {abs(reduction_pct):.1f}% MORE CPU time than Stats"
    else:
        reduction_msg = "≈ EQUAL CPU time usage"
    
    report = f"""
{'='*80}
CPU Usage Comparison Report
{'='*80}
Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
Warmup Period: {warmup_seconds} seconds
Monitoring Duration: {duration_seconds:.1f} seconds
Total Samples: {len(data)}

mac_stats App:
  Main PID: {mac_stats_pids[0] if mac_stats_pids else 'N/A'}
  Total Processes Tracked: {len(mac_stats_pids)}
  CPU Time Consumed (during monitoring): {format_cpu_time(mac_stats_cpu_time_delta)}
  Average CPU%: {mac_stats_avg_cpu_percent:.2f}% (calculated from CPU time)

Stats App:
  Main PID: {stats_pids[0] if stats_pids else 'N/A'}
  Total Processes Tracked: {len(stats_pids)}
  CPU Time Consumed (during monitoring): {format_cpu_time(stats_cpu_time_delta)}
  Average CPU%: {stats_avg_cpu_percent:.2f}% (calculated from CPU time)

{'='*80}
RESULTS SUMMARY (Based on CPU Time - Most Reliable Metric)
{'='*80}
  mac_stats: {format_cpu_time(mac_stats_cpu_time_delta)} CPU time ({mac_stats_avg_cpu_percent:.2f}% average)
  Stats:     {format_cpu_time(stats_cpu_time_delta)} CPU time ({stats_avg_cpu_percent:.2f}% average)
  Absolute Difference: {format_cpu_time(abs(mac_stats_cpu_time_delta - stats_cpu_time_delta))}
  
  {reduction_msg}
  
  {'🎉' if mac_stats_cpu_time_delta < stats_cpu_time_delta and reduction_pct > 50 else ''}

Note: CPU time is measured from the end of the {warmup_seconds}-second warmup period,
representing steady-state usage (not startup). All child processes are tracked.

{'='*80}
"""
    
    with open(output_file, 'w') as f:
        f.write(report)
    
    print(report)
    return report

def save_csv(data, output_file):
    """Save data to CSV file"""
    with open(output_file, 'w') as f:
        f.write("timestamp,mac_stats_cpu_time_delta,mac_stats_cpu_percent,mac_stats_procs,"
                "stats_cpu_time_delta,stats_cpu_percent,stats_procs,"
                "reduction_percent\n")
        
        for entry in data:
            timestamp = entry['timestamp']
            mac_stats = entry['mac_stats']
            stats = entry['stats']
            
            mac_stats_cpu_time = mac_stats.get('cpu_time_delta', 0)
            stats_cpu_time = stats.get('cpu_time_delta', 0)
            mac_stats_cpu = mac_stats.get('cpu', 0)
            stats_cpu = stats.get('cpu', 0)
            
            reduction = calculate_reduction_percentage(mac_stats_cpu_time, stats_cpu_time)
            reduction_str = "inf" if reduction == float('inf') else f"{reduction:.2f}"
            
            f.write(f"{timestamp},"
                   f"{mac_stats_cpu_time:.2f},"
                   f"{mac_stats_cpu:.2f},"
                   f"{mac_stats.get('process_count', 0)},"
                   f"{stats_cpu_time:.2f},"
                   f"{stats_cpu:.2f},"
                   f"{stats.get('process_count', 0)},"
                   f"{reduction_str}\n")
    
    print(f"✓ CSV saved: {output_file}")

def main():
    # Parse arguments
    args = sys.argv[1:]
    duration = 60
    interval = 1.0
    warmup = 30
    auto_kill = False
    keep_running = None  # None = ask, True = keep, False = kill
    debug = False
    
    i = 0
    while i < len(args):
        arg = args[i]
        if arg == '--auto-kill':
            auto_kill = True
        elif arg == '--keep-running':
            keep_running = True
        elif arg == '--kill-after':
            keep_running = False
        elif arg == '--debug':
            debug = True
        elif arg.isdigit() or ('.' in arg and arg.replace('.', '').isdigit()):
            if duration == 60 and interval == 1.0 and warmup == 30:
                # First numeric arg is duration
                duration = int(float(arg))
            elif interval == 1.0 and warmup == 30:
                # Second numeric arg is interval
                interval = float(arg)
            else:
                # Third numeric arg is warmup
                warmup = int(float(arg))
        i += 1
    
    print_header()
    
    # Check for existing processes
    print("Checking for existing processes...")
    mac_stats_existing = find_processes_by_pattern("mac_stats")
    stats_existing = find_processes_by_pattern("Stats.app")
    
    if mac_stats_existing or stats_existing:
        print("  Found existing processes:")
        if mac_stats_existing:
            print(f"    - mac_stats: {len(mac_stats_existing)} process(es)")
        if stats_existing:
            print(f"    - Stats: {len(stats_existing)} process(es)")
        
        if auto_kill:
            response = 'y'
            print("  Auto-kill enabled: killing existing processes...")
        else:
            response = input("  Kill existing processes and start fresh? (y/n): ").strip().lower()
        
        if response == 'y':
            # Kill existing processes
            if mac_stats_existing:
                kill_existing_processes("mac_stats", "mac_stats")
            if stats_existing:
                kill_existing_processes("Stats.app", "Stats")
            
            # Start fresh after killing
            print("\n  Starting fresh...")
            mac_stats_main_pid = start_mac_stats()
            stats_main_pid = start_stats_app()
            
            if not mac_stats_main_pid or not stats_main_pid:
                print("✗ Failed to start apps")
                return
            
            # Get all related PIDs
            mac_stats_pids = get_all_related_pids(mac_stats_main_pid)
            stats_pids = get_all_related_pids(stats_main_pid)
            
            print(f"\n  mac_stats: {len(mac_stats_pids)} process(es) (main PID: {mac_stats_main_pid})")
            print(f"  Stats: {len(stats_pids)} process(es) (main PID: {stats_main_pid})")
        else:
            # Use existing processes
            print("  Using existing processes...")
            # Extract PIDs from existing processes
            if mac_stats_existing:
                mac_stats_main_pid = int(mac_stats_existing[0]['pid'])
            else:
                mac_stats_main_pid = start_mac_stats()
            
            if stats_existing:
                # Find the main Stats process (the one with MacOS/Stats in path)
                main_stats_procs = [p for p in stats_existing if "Stats.app/Contents/MacOS/Stats" in p['cmd']]
                if main_stats_procs:
                    stats_main_pid = int(main_stats_procs[0]['pid'])
                else:
                    stats_main_pid = int(stats_existing[0]['pid'])
            else:
                stats_main_pid = start_stats_app()
            
            if not mac_stats_main_pid or not stats_main_pid:
                print("✗ Failed to get process PIDs")
                return
            
            mac_stats_pids = get_all_related_pids(mac_stats_main_pid)
            stats_pids = get_all_related_pids(stats_main_pid)
            
            print(f"\n  mac_stats: {len(mac_stats_pids)} process(es) (main PID: {mac_stats_main_pid})")
            print(f"  Stats: {len(stats_pids)} process(es) (main PID: {stats_main_pid})")
            
            # Skip warmup if using existing processes
            warmup = 0
    else:
        # No existing processes - start fresh
        print("  No existing processes found. Starting fresh...")
        mac_stats_main_pid = start_mac_stats()
        stats_main_pid = start_stats_app()
        
        if not mac_stats_main_pid or not stats_main_pid:
            print("✗ Failed to start apps")
            return
        
        # Get all related PIDs
        mac_stats_pids = get_all_related_pids(mac_stats_main_pid)
        stats_pids = get_all_related_pids(stats_main_pid)
        
        print(f"\n  mac_stats: {len(mac_stats_pids)} process(es) (main PID: {mac_stats_main_pid})")
        print(f"  Stats: {len(stats_pids)} process(es) (main PID: {stats_main_pid})")
    
    # Warmup period
    if warmup > 0:
        if not warmup_period(warmup, mac_stats_pids, stats_pids):
            print("✗ Warmup failed - processes died")
            return
    
    # Refresh PIDs after warmup (in case new child processes were spawned)
    if warmup > 0:
        print("\nRefreshing process list after warmup...")
        mac_stats_pids = get_all_related_pids(mac_stats_main_pid)
        stats_pids = get_all_related_pids(stats_main_pid)
        print(f"  mac_stats: {len(mac_stats_pids)} process(es)")
        print(f"  Stats: {len(stats_pids)} process(es)")
    
    # Capture baseline CPU times
    print("\nCapturing baseline CPU times...")
    if debug:
        print("  [DEBUG] mac_stats PIDs:", mac_stats_pids)
        print("  [DEBUG] Stats PIDs:", stats_pids)
    
    mac_stats_initial_times = capture_initial_cpu_times(mac_stats_pids, debug)
    stats_initial_times = capture_initial_cpu_times(stats_pids, debug)
    
    if debug:
        print(f"  [DEBUG] mac_stats initial times: {mac_stats_initial_times}")
        print(f"  [DEBUG] Stats initial times: {stats_initial_times}")
    
    if not mac_stats_initial_times and not stats_initial_times:
        print("  ✗ No valid processes found to monitor!")
        print("  [DEBUG] Attempting to verify processes exist...")
        for pid in mac_stats_pids + stats_pids:
            try:
                result = subprocess.run(['ps', '-p', str(pid)], capture_output=True, check=False)
                if result.returncode == 0:
                    print(f"    PID {pid} exists")
                else:
                    print(f"    PID {pid} does not exist")
            except:
                pass
        return
    
    print(f"  ✓ Baseline captured ({len(mac_stats_initial_times)} mac_stats, {len(stats_initial_times)} Stats)")
    
    # Start monitoring
    print(f"\n📊 Monitoring for {duration} seconds (interval: {interval}s)")
    print()
    print("Time     | mac_stats CPU Time | Stats CPU Time | Reduction")
    print("-" * 80)
    
    data = []
    start_time = time.time()
    
    try:
        while True:
            elapsed = time.time() - start_time
            if elapsed >= duration:
                break
            
            timestamp = datetime.now().strftime('%H:%M:%S')
            
            if debug and elapsed < 3:  # Only debug first few seconds
                print(f"  [DEBUG] Sample at {timestamp}:")
            
            mac_stats_metrics = get_total_cpu_metrics(mac_stats_pids, mac_stats_initial_times, debug and elapsed < 3)
            stats_metrics = get_total_cpu_metrics(stats_pids, stats_initial_times, debug and elapsed < 3)
            
            entry = {
                'timestamp': timestamp,
                'mac_stats': mac_stats_metrics,
                'stats': stats_metrics
            }
            data.append(entry)
            
            print_status_line(timestamp, mac_stats_metrics, stats_metrics, elapsed)
            
            time.sleep(interval)
    
    except KeyboardInterrupt:
        print("\n\n⚠ Monitoring interrupted by user")
    
    if not data:
        print("✗ No data collected")
        return
    
    # Generate reports
    print("\n" + "="*80)
    print("Monitoring complete. Generating reports...")
    print("="*80)
    
    timestamp_str = datetime.now().strftime("%Y%m%d_%H%M%S")
    report_file = f"cpu-comparison-report-{timestamp_str}.txt"
    csv_file = f"cpu-comparison-data-{timestamp_str}.csv"
    
    actual_duration = time.time() - start_time
    generate_summary_report(data, report_file, mac_stats_pids, stats_pids, warmup, actual_duration)
    save_csv(data, csv_file)
    
    # Offer to take screenshot (skip in non-interactive mode)
    if not auto_kill:
        print("\nWould you like to take a screenshot? (y/n): ", end='')
        try:
            response = input().strip().lower()
            if response == 'y':
                screenshot_dir = "cpu-comparison-screenshots"
                os.makedirs(screenshot_dir, exist_ok=True)
                filename = f"{screenshot_dir}/comparison-{timestamp_str}.png"
                subprocess.run(['screencapture', '-x', '-t', 'png', filename], check=True)
                print(f"✓ Screenshot saved: {filename}")
        except (EOFError, KeyboardInterrupt):
            pass
    
    print(f"\n✓ Report saved: {report_file}")
    print(f"✓ CSV saved: {csv_file}")
    print("\n✅ Monitoring complete!")
    
    # Handle cleanup
    if keep_running is None and not auto_kill:
        # Ask user
        print("\nKeep apps running? (y/n): ", end='')
        try:
            response = input().strip().lower()
            keep_running = (response == 'y')
        except (EOFError, KeyboardInterrupt):
            keep_running = True  # Default to keeping if interrupted
    elif keep_running is None:
        keep_running = True  # Default to keeping in auto mode
    
    if not keep_running:
        print("\nKilling processes...")
        killed_count = 0
        for pid in mac_stats_pids:
            try:
                os.kill(pid, signal.SIGTERM)
                killed_count += 1
            except:
                pass
        for pid in stats_pids:
            try:
                os.kill(pid, signal.SIGTERM)
                killed_count += 1
            except:
                pass
        print(f"✓ Terminated {killed_count} process(es)")
    else:
        print("\n✓ Apps are still running")

if __name__ == '__main__':
    main()
