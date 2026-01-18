# Data Poster Charts - Backend Requirements

## Overview
The data-poster theme now uses bar graphs and line charts instead of ring gauges. The frontend implementation is complete and works with the current backend data, but there are potential optimizations that could be made.

## Current Status
✅ **Frontend is fully functional** - The charts work with the existing `get_cpu_details()` API response.

## Current Data Usage
The frontend currently uses:
- `data.temperature` - For temperature charts (updated every refresh cycle)
- `data.usage` - For CPU usage charts (updated every refresh cycle)  
- `data.frequency` - For frequency charts (updated every refresh cycle)

## Potential Backend Optimizations (Optional)

### 1. Historical Data Buffer
**Current:** Frontend maintains its own rolling buffer of values from each refresh call.

**Potential Enhancement:** Backend could maintain a shared buffer and return the last N values in the API response:
```rust
pub struct CpuDetails {
    // ... existing fields ...
    
    // Optional: Historical data for charts
    pub temperature_history: Option<Vec<f64>>,  // Last 60 values
    pub usage_history: Option<Vec<f64>>,        // Last 60 values
    pub frequency_history: Option<Vec<f64>>,    // Last 60 values
}
```

**Benefits:**
- More consistent data across window resizes/reloads
- Backend can smooth/filter data before sending
- Reduces frontend memory usage

**Priority:** Low - Current frontend implementation works well

### 2. Chart-Specific Refresh Rate
**Current:** All metrics refresh at the same rate (1 second when window is visible).

**Potential Enhancement:** Different refresh rates for different metrics:
- Temperature: 2-3 seconds (changes slowly)
- Usage: 1 second (needs to be responsive)
- Frequency: 1 second (can change quickly)

**Benefits:**
- Reduced CPU usage for temperature reads (SMC calls)
- Better battery life

**Priority:** Low - Current implementation is acceptable

### 3. Data Smoothing
**Current:** Frontend displays raw values.

**Potential Enhancement:** Backend could apply moving average or exponential smoothing to reduce noise in charts.

**Priority:** Very Low - Frontend can implement this if needed

## Implementation Notes
- The frontend chart system (`poster-charts.js`) is self-contained and works independently
- Charts automatically scale based on min/max values seen
- No breaking changes to existing API - charts gracefully handle missing data
- Charts initialize with empty buffers and populate as data arrives

## Testing
To test the charts:
1. Open the data-poster theme
2. Verify bar charts appear in top-right of each metric card
3. Verify line charts appear at bottom of each metric card
4. Verify charts update smoothly as values change
5. Verify charts handle missing data gracefully (shows empty/zero)
