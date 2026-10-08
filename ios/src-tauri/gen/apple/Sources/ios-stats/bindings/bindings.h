#pragma once

namespace ffi {
    extern "C" {
        void start_app();
        // Defined in the LLM plugin (BackgroundRefresh.swift). iOS requires Background
        // App Refresh tasks to be registered before the app finishes launching.
        void ios_stats_register_background_refresh();
    }
}

