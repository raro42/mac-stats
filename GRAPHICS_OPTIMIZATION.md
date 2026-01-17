# Graphics and Media CPU Optimization

## Problem Identified

**"Graphics and Media" thread using 5.7% CPU**

The issue is WebKit's rendering operations:
- `RemoteLayerTreeDrawingAreaProxy::didRefreshDisplay()` - display refresh callbacks
- `CA::Transaction::commit()` - Core Animation transactions (QuartzCore)
- `displayLinkFired` - 60fps display link callbacks
- Layer tree updates happening too frequently

## Root Cause

Even though animations were throttled to 30fps, `requestAnimationFrame` was still being called at 60fps:
- Every `requestAnimationFrame` call triggers WebKit's display link
- WebKit processes these callbacks even if we skip DOM updates
- This causes unnecessary Graphics/Media CPU usage

## Solution Applied

### 1. Reduced Animation Frame Rate
- **Before**: 30fps (33ms per frame)
- **After**: 20fps (50ms per frame)
- **Impact**: 33% reduction in animation callbacks

### 2. Optimized requestAnimationFrame Usage
- **Before**: Always called `requestAnimationFrame` even when throttling
- **After**: Only schedule next frame when actually updating DOM
- **Impact**: Prevents unnecessary WebKit display link processing

### 3. Increased Direct Update Threshold
- **Before**: Animate if change > 5%
- **After**: Animate if change > 10%
- **Impact**: More direct updates, fewer animations

### 4. Faster Animation Completion
- **Before**: 0.25 interpolation speed
- **After**: 0.3 interpolation speed
- **Impact**: Animations complete faster with fewer frames

## Expected Results

- **Graphics/Media CPU**: Should drop from 5.7% to ~2-3%
- **Overall CPU**: Should drop from 1.3% average to ~0.8-1.0%
- **Animation smoothness**: Still smooth at 20fps for gauge animations

## Testing

After rebuild, monitor:
1. Graphics/Media CPU usage in Activity Monitor
2. Overall backend CPU usage
3. Animation smoothness in the UI

If still high, we can:
- Further reduce to 15fps (66ms)
- Disable animations entirely for small changes
- Use CSS transitions instead of JavaScript animations
