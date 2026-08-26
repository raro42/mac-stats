# Menu bar disk % stuck at ~94% after cleanup (2026-08-26)

## Symptom
System had ~375–400 GB free (~55–59% used on Data). Menu bar still showed ~94% SSD.

## Causes
1. `get_disk_usage_percent` refreshed disks **once** at process start, then never again. Long-lived process (~19d uptime) kept the pre-cleanup reading.
2. Using `disks.list().first()` can pick a nearly-full **external** volume (e.g. `/Volumes/x9pro 1` ~95%) instead of Macintosh HD / Data.

## Fix (v0.1.649)
- Refresh disk list about every 60s.
- Prefer mount `/System/Volumes/Data`, then `/`, then largest non-removable disk.

## Truth check (same day)
`df -h /System/Volumes/Data` → ~58% used, ~375 GB avail.
