#!/bin/bash
# Script to trace the mac_stats process

# Find the backend process
PID=$(ps aux | grep -E "mac_stats" | grep -v grep | awk '{print $2}' | head -1)

if [ -z "$PID" ]; then
    echo "Error: mac_stats process not found"
    echo "Please start the backend first with: sudo cargo run -- --cpu"
    exit 1
fi

echo "Found backend process: PID $PID"
echo ""
echo "Choose tracing method:"
echo "1. dtruss - System call tracing (like strace on Linux) - shows file/network operations"
echo "2. sample - CPU sampling (shows what functions are consuming CPU)"
echo "3. dtrace - Trace specific syscalls (mach_msg, read, write, etc.)"
echo "4. dtruss filtered - Only show expensive syscalls (read, write, open, etc.)"
echo ""
read -p "Enter choice (1-4): " choice

case $choice in
    1)
        echo "Tracing ALL system calls with dtruss (Ctrl+C to stop)..."
        echo "This will show all system calls made by the backend"
        echo "Output saved to backend_syscalls.log"
        echo ""
        sudo dtruss -p $PID 2>&1 | tee backend_syscalls.log
        ;;
    2)
        echo "Sampling CPU usage for 10 seconds..."
        echo "This will show which functions are consuming CPU"
        echo ""
        sample $PID 10 -f backend_sample.txt
        echo "Sample saved to backend_sample.txt"
        echo ""
        echo "=== Top CPU consumers ==="
        cat backend_sample.txt | grep -E "^[0-9]+ " | head -30
        ;;
    3)
        echo "Tracing expensive syscalls with dtrace (Ctrl+C to stop)..."
        echo "This will count syscalls by type"
        echo ""
        sudo dtrace -n 'syscall:::entry /pid == '$PID'/ { @[probefunc] = count(); }' -o backend_dtrace.txt
        echo "Trace saved to backend_dtrace.txt"
        cat backend_dtrace.txt
        ;;
    4)
        echo "Tracing filtered system calls (read, write, open, etc.)..."
        echo "This will show only expensive operations"
        echo "Output saved to backend_syscalls_filtered.log"
        echo ""
        sudo dtruss -p $PID 2>&1 | grep -E "(read|write|open|close|stat|mach_msg|select|poll)" | tee backend_syscalls_filtered.log
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac
