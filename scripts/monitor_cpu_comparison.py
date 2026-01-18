#!/usr/bin/env python3
"""
CPU Usage Comparison Monitor
Monitors CPU usage of mac_stats and Stats apps for comparison

Usage:
    python3 monitor_cpu_comparison.py [duration] [interval]
    
    duration: How long to monitor (seconds, default: 60)
    interval: Sampling interval (seconds, default: 1)
"""

import subprocess
import time
import sys
import os
from datetime import datetime
from collections import defaultdict
import json

def find_processes_by_pattern(pattern):
    """Find all processes matching the pattern (name or command line)"""
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

def get_process_name(pid):
    """Get the process name (as shown in Activity Monitor)"""
    try:
        result = subprocess.run(
            ['ps', '-p', str(pid), '-o', 'comm='],
            capture_output=True,
            text=True,
            check=False
        )
        return result.stdout.strip()
    except:
        return ""

def is_mac_stats_process(cmd, pid=None):
    """Check if a command line belongs to mac-stats app"""
    cmd_lower = cmd.lower()
    
    # Exclude common false positives
    exclude_patterns = [
        'cursor',
        'python3.framework',
        'extension-host',
        'microsoft teams',
        'code helper',
        'vscode',
    ]
    
    for exclude in exclude_patterns:
        if exclude in cmd_lower:
            return False
    
    # Check process name (as shown in Activity Monitor)
    # This catches processes like "mac_stats Networking", "mac_stats Graphics and Media"
    if pid:
        proc_name = get_process_name(pid)
        proc_name_lower = proc_name.lower()
        if 'mac_stats' in proc_name_lower or 'mac-stats' in proc_name_lower:
            return True
        # Also check for Tauri WebView processes
        if 'tauri://localhost' in proc_name or 'tauri' in proc_name_lower:
            return True
    
    # Must match one of these patterns in the actual executable path
    # (not just in environment variables or arguments)
    include_patterns = [
        '/mac_stats',           # Executable name
        '/mac-stats',           # Executable name with hyphen
        'mac_stats',            # Standalone executable
        'mac-stats.app',        # App bundle
        'target/debug/mac_stats',  # Debug build
        'target/release/mac_stats', # Release build
        'com.raro42.mac-stats', # Bundle identifier
        'tauri://localhost',    # Tauri WebView processes
    ]
    
    # Check if any include pattern matches
    for pattern in include_patterns:
        if pattern in cmd:
            return True
    
    return False

def find_processes_by_bundle_id(bundle_id):
    """Find processes by macOS bundle identifier"""
    processes = []
    try:
        # Use ps to find processes with matching bundle identifier
        result = subprocess.run(
            ['ps', 'ax', '-o', 'pid=,command='],
            capture_output=True,
            text=True,
            check=False
        )
        for line in result.stdout.strip().split('\n'):
            if bundle_id in line:
                parts = line.split(None, 1)
                if len(parts) >= 2:
                    pid = parts[0].strip()
                    cmd = parts[1]
                    processes.append({'pid': pid, 'cmd': cmd})
    except Exception as e:
        pass
    return processes

def get_process_environment(pid):
    """Get environment variables for a process"""
    try:
        result = subprocess.run(
            ['ps', '-p', str(pid), '-o', 'command='],
            capture_output=True,
            text=True,
            check=False
        )
        # Try to get environment via /proc (Linux) or ps eww (macOS)
        # On macOS, we can't easily get full env, but we can check command line
        return result.stdout.strip()
    except Exception as e:
        return ""

def get_process_children(parent_pid):
    """Get all child processes of a parent PID"""
    children = []
    try:
        result = subprocess.run(
            ['pgrep', '-P', str(parent_pid)],
            capture_output=True,
            text=True,
            check=False
        )
        for pid in result.stdout.strip().split('\n'):
            if pid:
                pid = pid.strip()
                # Get command for this PID
                cmd_result = subprocess.run(
                    ['ps', '-p', pid, '-o', 'command='],
                    capture_output=True,
                    text=True,
                    check=False
                )
                cmd = cmd_result.stdout.strip()
                if cmd:
                    children.append({'pid': pid, 'cmd': cmd})
                    # Recursively get grandchildren
                    grandchildren = get_process_children(pid)
                    children.extend(grandchildren)
    except Exception as e:
        pass
    return children

def find_all_mac_stats_processes():
    """Find all processes belonging to mac-stats app"""
    all_processes = []
    seen_pids = set()
    
    # Method 1: Find by process name patterns, but filter carefully
    patterns = ['mac_stats', 'mac-stats', 'tauri://localhost']
    for pattern in patterns:
        procs = find_processes_by_pattern(pattern)
        for proc in procs:
            pid = proc['pid']
            cmd = proc['cmd']
            # Only include if it's actually a mac-stats process
            if pid not in seen_pids and is_mac_stats_process(cmd, pid):
                all_processes.append(proc)
                seen_pids.add(pid)
    
    # Method 2: Find by bundle identifier
    bundle_procs = find_processes_by_bundle_id('com.raro42.mac-stats')
    for proc in bundle_procs:
        pid = proc['pid']
        cmd = proc['cmd']
        if pid not in seen_pids and is_mac_stats_process(cmd, pid):
            all_processes.append(proc)
            seen_pids.add(pid)
    
    # Method 3: Find all child processes of main processes
    # First, identify main processes (those that might spawn children)
    main_pids = [proc['pid'] for proc in all_processes]
    for main_pid in main_pids:
        children = get_process_children(main_pid)
        for child in children:
            pid = child['pid']
            cmd = child['cmd']
            if pid not in seen_pids:
                # Include child processes if they're clearly related (Tauri helpers, etc.)
                if is_mac_stats_process(cmd, pid):
                    all_processes.append(child)
                    seen_pids.add(pid)
    
    # Method 4: Scan all processes and check by process name
    # This catches processes like "mac_stats Networking", "mac_stats Graphics and Media"
    try:
        result = subprocess.run(
            ['ps', 'ax', '-o', 'pid=,comm=,command='],
            capture_output=True,
            text=True,
            check=False
        )
        for line in result.stdout.strip().split('\n'):
            if line:
                parts = line.split(None, 2)
                if len(parts) >= 3:
                    pid = parts[0].strip()
                    comm = parts[1]  # Process name (comm)
                    cmd = parts[2]   # Full command line
                    
                    if pid not in seen_pids:
                        # Check by process name first (catches "mac_stats Networking", etc.)
                        comm_lower = comm.lower()
                        if 'mac_stats' in comm_lower or 'mac-stats' in comm_lower:
                            if is_mac_stats_process(cmd, pid):
                                all_processes.append({'pid': pid, 'cmd': cmd})
                                seen_pids.add(pid)
                        # Also check for Tauri processes
                        elif 'tauri' in comm_lower or 'tauri://localhost' in comm:
                            if is_mac_stats_process(cmd, pid):
                                all_processes.append({'pid': pid, 'cmd': cmd})
                                seen_pids.add(pid)
                        # Or check by command line
                        elif is_mac_stats_process(cmd, pid):
                            all_processes.append({'pid': pid, 'cmd': cmd})
                            seen_pids.add(pid)
    except Exception as e:
        pass
    
    return all_processes

def is_stats_app_process(cmd):
    """Check if a command line belongs to Stats app"""
    cmd_lower = cmd.lower()
    
    # Exclude common false positives
    exclude_patterns = [
        'microsoft teams',
        'cursor',
        'code helper',
        'vscode',
        'python3.framework',
    ]
    
    for exclude in exclude_patterns:
        if exclude in cmd_lower:
            return False
    
    # Must match Stats app patterns
    include_patterns = [
        '/stats.app/contents/macos/stats',  # Main executable
        '/stats.app/contents/plugins/',     # Widget extensions
        'stats.app',
    ]
    
    for pattern in include_patterns:
        if pattern in cmd_lower:
            return True
    
    return False

def find_all_stats_processes():
    """Find all processes belonging to Stats app"""
    all_processes = []
    seen_pids = set()
    
    # Find by pattern
    procs = find_processes_by_pattern("Stats")
    for proc in procs:
        pid = proc['pid']
        cmd = proc['cmd']
        if pid not in seen_pids and is_stats_app_process(cmd):
            all_processes.append(proc)
            seen_pids.add(pid)
    
    # Also check for child processes
    main_pids = [proc['pid'] for proc in all_processes]
    for main_pid in main_pids:
        children = get_process_children(main_pid)
        for child in children:
            pid = child['pid']
            cmd = child['cmd']
            if pid not in seen_pids and is_stats_app_process(cmd):
                all_processes.append(child)
                seen_pids.add(pid)
    
    return all_processes

def find_processes(pattern):
    """Legacy function - now uses smarter detection for mac_stats and Stats"""
    if pattern == "mac_stats" or pattern == "mac-stats":
        return find_all_mac_stats_processes()
    elif pattern == "Stats":
        return find_all_stats_processes()
    else:
        return find_processes_by_pattern(pattern)

def parse_cpu_time(cpu_time_str):
    """Parse CPU time string (format: DD-HH:MM:SS or HH:MM:SS) to seconds"""
    try:
        parts = cpu_time_str.strip().split(':')
        if len(parts) == 3:
            # Format: HH:MM:SS
            hours = int(parts[0])
            minutes = int(parts[1])
            seconds = int(parts[2])
            return hours * 3600 + minutes * 60 + seconds
        elif len(parts) == 2:
            # Format: MM:SS
            minutes = int(parts[0])
            seconds = int(parts[1])
            return minutes * 60 + seconds
        elif '-' in cpu_time_str:
            # Format: DD-HH:MM:SS
            day_part, time_part = cpu_time_str.split('-', 1)
            days = int(day_part)
            time_seconds = parse_cpu_time(time_part)
            return days * 86400 + time_seconds
    except:
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
            ['ps', '-p', pid, '-o', '%cpu=,rss=,%mem=,time='],
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

def get_all_process_cpu(pattern, initial_cpu_times=None):
    """Get total CPU usage and CPU time for all processes matching pattern"""
    processes = find_processes(pattern)
    total_cpu = 0.0
    total_rss = 0.0
    total_mem = 0.0
    total_cpu_time = 0.0
    total_cpu_time_delta = 0.0
    pids = []
    
    if initial_cpu_times is None:
        initial_cpu_times = {}
    
    for proc in processes:
        pid = proc['pid']
        metrics = get_cpu_usage(pid)
        if metrics:
            total_cpu += metrics['cpu']
            total_rss += metrics['rss_mb']
            total_mem += metrics['mem']
            total_cpu_time += metrics['cpu_time_seconds']
            
            # Calculate delta from initial time
            if pid in initial_cpu_times:
                delta = metrics['cpu_time_seconds'] - initial_cpu_times[pid]
                total_cpu_time_delta += delta
            
            pids.append(pid)
    
    return {
        'cpu': total_cpu,
        'rss_mb': total_rss,
        'mem': total_mem,
        'cpu_time_seconds': total_cpu_time,
        'cpu_time_delta': total_cpu_time_delta,
        'process_count': len(processes),
        'pids': pids,
        'processes': processes
    }

def print_process_details(processes, app_name):
    """Print details about found processes"""
    if not processes:
        print(f"  ⚠ No {app_name} processes found")
        return
    
    print(f"  Found {len(processes)} {app_name} process(es):")
    for i, proc in enumerate(processes, 1):
        pid = proc['pid']
        cmd = proc['cmd']
        # Truncate long commands
        if len(cmd) > 70:
            cmd = cmd[:67] + "..."
        print(f"    {i}. PID {pid:>6}: {cmd}")

def print_header():
    """Print monitoring header"""
    print("\n" + "="*80)
    print("  CPU Usage Comparison Monitor")
    print("="*80)
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print()

def calculate_reduction_percentage(mac_stats_cpu, stats_cpu):
    """Calculate percentage reduction/increase in CPU usage"""
    if stats_cpu == 0:
        if mac_stats_cpu == 0:
            return 0.0
        return float('inf')  # mac_stats uses CPU but Stats doesn't
    if mac_stats_cpu == 0:
        return 100.0  # mac_stats uses 0, Stats uses some
    
    if mac_stats_cpu < stats_cpu:
        # Reduction: how much less mac_stats uses compared to Stats
        reduction = ((stats_cpu - mac_stats_cpu) / stats_cpu) * 100
        return reduction
    else:
        # mac_stats uses more - show as negative reduction (increase)
        increase = ((mac_stats_cpu - stats_cpu) / stats_cpu) * 100
        return -increase

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

def print_status_line(timestamp, mac_stats, stats_app, elapsed_seconds=0):
    """Print a single status line showing both instantaneous CPU% and CPU time delta"""
    mac_stats_cpu = mac_stats.get('cpu', 0.0)
    stats_cpu = stats_app.get('cpu', 0.0)
    mac_stats_procs = mac_stats.get('process_count', 0)
    stats_procs = stats_app.get('process_count', 0)
    
    # Get CPU time deltas
    mac_stats_cpu_time_delta = mac_stats.get('cpu_time_delta', 0.0)
    stats_cpu_time_delta = stats_app.get('cpu_time_delta', 0.0)
    
    # Calculate percentage reduction based on CPU time (more reliable)
    reduction_pct = calculate_reduction_percentage(mac_stats_cpu_time_delta, stats_cpu_time_delta)
    
    # Format CPU time deltas
    mac_stats_time_str = format_cpu_time(mac_stats_cpu_time_delta) if mac_stats_cpu_time_delta > 0 else "0.0s"
    stats_time_str = format_cpu_time(stats_cpu_time_delta) if stats_cpu_time_delta > 0 else "0.0s"
    
    # Show instantaneous CPU% for reference
    mac_stats_str = f"{mac_stats_cpu:5.2f}%"
    stats_str = f"{stats_cpu:5.2f}%"
    
    # Build comparison message based on CPU time
    if mac_stats_cpu_time_delta < stats_cpu_time_delta and stats_cpu_time_delta > 0:
        if reduction_pct == float('inf'):
            comparison = "✓ mac_stats uses LESS (Stats=0s)"
        else:
            comparison = f"✓ -{reduction_pct:.1f}% CPU time"
    elif mac_stats_cpu_time_delta > stats_cpu_time_delta:
        comparison = f"✗ +{abs(reduction_pct):.1f}% CPU time"
    else:
        comparison = "≈ EQUAL"
    
    # Show process details
    proc_detail = f"({mac_stats_procs} procs)" if mac_stats_procs > 1 else ""
    stats_detail = f"({stats_procs} procs)" if stats_procs > 1 else ""
    
    # Print with CPU time delta (primary metric) - more reliable
    print(f"{timestamp} | mac_stats: {mac_stats_time_str:>10} {proc_detail:12} | "
          f"Stats: {stats_time_str:>10} {stats_detail:12} | {comparison}")

def save_screenshot():
    """Take a screenshot of Activity Monitor or current screen"""
    screenshot_dir = "cpu-comparison-screenshots"
    os.makedirs(screenshot_dir, exist_ok=True)
    
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    filename = f"{screenshot_dir}/comparison-{timestamp}.png"
    
    print(f"\nTaking screenshot: {filename}")
    try:
        subprocess.run(['screencapture', '-x', '-t', 'png', filename], check=True)
        print(f"✓ Screenshot saved: {filename}")
        return filename
    except Exception as e:
        print(f"✗ Screenshot failed: {e}")
        return None

def generate_summary_report(data, output_file, mac_stats_procs=None, stats_procs=None, duration_seconds=0):
    """Generate a summary report"""
    mac_stats_cpus = [d['mac_stats']['cpu'] for d in data if d['mac_stats']['cpu'] > 0]
    stats_cpus = [d['stats']['cpu'] for d in data if d['stats']['cpu'] > 0]
    
    # Get final CPU time deltas (most reliable metric)
    final_mac_stats = data[-1]['mac_stats'] if data else {}
    final_stats = data[-1]['stats'] if data else {}
    mac_stats_cpu_time_delta = final_mac_stats.get('cpu_time_delta', 0.0)
    stats_cpu_time_delta = final_stats.get('cpu_time_delta', 0.0)
    
    def stats(values):
        if not values:
            return {'avg': 0, 'min': 0, 'max': 0, 'count': 0}
        return {
            'avg': sum(values) / len(values),
            'min': min(values),
            'max': max(values),
            'count': len(values)
        }
    
    mac_stats_stats = stats(mac_stats_cpus)
    stats_stats = stats(stats_cpus)
    
    # Calculate percentage reduction based on CPU time (more reliable)
    reduction_pct = calculate_reduction_percentage(mac_stats_cpu_time_delta, stats_cpu_time_delta)
    
    # Calculate average CPU% per second (CPU time / elapsed time)
    if duration_seconds > 0:
        mac_stats_avg_cpu_percent = (mac_stats_cpu_time_delta / duration_seconds) * 100
        stats_avg_cpu_percent = (stats_cpu_time_delta / duration_seconds) * 100
    else:
        mac_stats_avg_cpu_percent = mac_stats_stats['avg']
        stats_avg_cpu_percent = stats_stats['avg']
    
    # Build process list section
    process_list = ""
    if mac_stats_procs:
        process_list += "\n  Tracked Processes:\n"
        for i, proc in enumerate(mac_stats_procs, 1):
            pid = proc['pid']
            cmd = proc['cmd']
            if len(cmd) > 60:
                cmd = cmd[:57] + "..."
            process_list += f"    {i}. PID {pid}: {cmd}\n"
    
    stats_process_list = ""
    if stats_procs:
        stats_process_list += "\n  Tracked Processes:\n"
        for i, proc in enumerate(stats_procs, 1):
            pid = proc['pid']
            cmd = proc['cmd']
            if len(cmd) > 60:
                cmd = cmd[:57] + "..."
            stats_process_list += f"    {i}. PID {pid}: {cmd}\n"
    
    # Build reduction message based on CPU time
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
Duration: {duration_seconds:.1f} seconds ({len(data)} samples)

mac_stats App:
  CPU Time Consumed: {format_cpu_time(mac_stats_cpu_time_delta)}
  Average CPU%: {mac_stats_avg_cpu_percent:.2f}% (calculated from CPU time)
  Instantaneous CPU%: {mac_stats_stats['avg']:.2f}% (average), {mac_stats_stats['min']:.2f}% (min), {mac_stats_stats['max']:.2f}% (max)
  Process Count: {len(mac_stats_procs) if mac_stats_procs else 0}{process_list}

Stats App:
  CPU Time Consumed: {format_cpu_time(stats_cpu_time_delta)}
  Average CPU%: {stats_avg_cpu_percent:.2f}% (calculated from CPU time)
  Instantaneous CPU%: {stats_stats['avg']:.2f}% (average), {stats_stats['min']:.2f}% (min), {stats_stats['max']:.2f}% (max)
  Process Count: {len(stats_procs) if stats_procs else 0}{stats_process_list}

{'='*80}
RESULTS SUMMARY (Based on CPU Time - Most Reliable Metric)
{'='*80}
  mac_stats: {format_cpu_time(mac_stats_cpu_time_delta)} CPU time ({mac_stats_avg_cpu_percent:.2f}% average)
  Stats:     {format_cpu_time(stats_cpu_time_delta)} CPU time ({stats_avg_cpu_percent:.2f}% average)
  Absolute Difference: {format_cpu_time(abs(mac_stats_cpu_time_delta - stats_cpu_time_delta))}
  
  {reduction_msg}
  
  {'🎉' if mac_stats_cpu_time_delta < stats_cpu_time_delta and reduction_pct > 50 else ''}

Note: CPU time is cumulative since process start, making it more reliable than 
instantaneous CPU% which can fluctuate. All processes belonging to each app 
(including Tauri WebView, helpers, etc.) are tracked.

{'='*80}
"""
    
    with open(output_file, 'w') as f:
        f.write(report)
    
    print(report)
    return report

def save_csv(data, output_file):
    """Save data to CSV file"""
    with open(output_file, 'w') as f:
        f.write("timestamp,mac_stats_cpu_percent,mac_stats_cpu_time_delta,mac_stats_rss_mb,mac_stats_mem,mac_stats_procs,"
                "stats_cpu_percent,stats_cpu_time_delta,stats_rss_mb,stats_mem,stats_procs,"
                "reduction_percent_cpu_time,reduction_percent_instant\n")
        
        for entry in data:
            timestamp = entry['timestamp']
            mac_stats = entry['mac_stats']
            stats = entry['stats']
            
            mac_stats_cpu = mac_stats.get('cpu', 0)
            stats_cpu = stats.get('cpu', 0)
            mac_stats_cpu_time = mac_stats.get('cpu_time_delta', 0)
            stats_cpu_time = stats.get('cpu_time_delta', 0)
            
            # Calculate reduction based on CPU time (primary) and instant CPU% (secondary)
            reduction_cpu_time = calculate_reduction_percentage(mac_stats_cpu_time, stats_cpu_time)
            reduction_instant = calculate_reduction_percentage(mac_stats_cpu, stats_cpu)
            
            # Handle infinity case
            reduction_time_str = "inf" if reduction_cpu_time == float('inf') else f"{reduction_cpu_time:.2f}"
            reduction_inst_str = "inf" if reduction_instant == float('inf') else f"{reduction_instant:.2f}"
            
            f.write(f"{timestamp},"
                   f"{mac_stats_cpu:.2f},"
                   f"{mac_stats_cpu_time:.2f},"
                   f"{mac_stats.get('rss_mb', 0):.2f},"
                   f"{mac_stats.get('mem', 0):.2f},"
                   f"{mac_stats.get('process_count', 0)},"
                   f"{stats_cpu:.2f},"
                   f"{stats_cpu_time:.2f},"
                   f"{stats.get('rss_mb', 0):.2f},"
                   f"{stats.get('mem', 0):.2f},"
                   f"{stats.get('process_count', 0)},"
                   f"{reduction_time_str},"
                   f"{reduction_inst_str}\n")
    
    print(f"✓ CSV saved: {output_file}")

def main():
    duration = int(sys.argv[1]) if len(sys.argv) > 1 else 60
    interval = float(sys.argv[2]) if len(sys.argv) > 2 else 1.0
    
    print_header()
    
    # Check if processes are running
    print("Scanning for processes...")
    mac_stats_procs = find_processes("mac_stats")
    stats_procs = find_processes("Stats")
    
    print("\n" + "-"*80)
    print("mac_stats App Processes:")
    print_process_details(mac_stats_procs, "mac_stats")
    
    print("\nStats App Processes:")
    print_process_details(stats_procs, "Stats")
    print("-"*80)
    print()
    
    if not mac_stats_procs:
        print("⚠ Warning: mac_stats process not found!")
        print("  Start it with:")
        print("    cd src-tauri && cargo run")
        print("    # or")
        print("    ./target/release/mac_stats")
        response = input("  Continue anyway? (y/n): ")
        if response.lower() != 'y':
            return
    
    if not stats_procs:
        print("⚠ Warning: Stats app process not found!")
        print("  Make sure Stats.app is running")
        response = input("  Continue anyway? (y/n): ")
        if response.lower() != 'y':
            return
    
    print(f"Monitoring for {duration} seconds (interval: {interval}s)")
    print()
    print("Capturing initial CPU times...")
    
    # Capture initial CPU times for all processes
    initial_mac_stats_times = {}
    initial_stats_times = {}
    
    for proc in mac_stats_procs:
        pid = proc['pid']
        cpu_time = get_cpu_time(pid)
        initial_mac_stats_times[pid] = cpu_time
    
    for proc in stats_procs:
        pid = proc['pid']
        cpu_time = get_cpu_time(pid)
        initial_stats_times[pid] = cpu_time
    
    print("Starting monitoring...")
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
            
            mac_stats_metrics = get_all_process_cpu("mac_stats", initial_mac_stats_times)
            stats_metrics = get_all_process_cpu("Stats", initial_stats_times)
            
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
    
    print("\n" + "="*80)
    print("Monitoring complete. Generating reports...")
    print("="*80)
    
    # Generate reports
    timestamp_str = datetime.now().strftime("%Y%m%d_%H%M%S")
    report_file = f"cpu-comparison-report-{timestamp_str}.txt"
    csv_file = f"cpu-comparison-data-{timestamp_str}.csv"
    
    # Get final process lists for report
    final_mac_stats_procs = find_processes("mac_stats")
    final_stats_procs = find_processes("Stats")
    
    actual_duration = time.time() - start_time
    generate_summary_report(data, report_file, final_mac_stats_procs, final_stats_procs, actual_duration)
    save_csv(data, csv_file)
    
    # Offer to take screenshot
    print("\nWould you like to take a screenshot? (y/n): ", end='')
    try:
        response = input().strip().lower()
        if response == 'y':
            save_screenshot()
    except (EOFError, KeyboardInterrupt):
        pass
    
    print(f"\n✓ Report saved: {report_file}")
    print(f"✓ CSV saved: {csv_file}")
    print("\n✅ Monitoring complete!")

if __name__ == '__main__':
    main()
