//! Instant-lane + Agent Ops implementation (shared module scope via include!).
use crate::config::Config;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::normalize::{contains_age_token, normalize_operator_command};

include!("agent_ops.inc.rs");
include!("keep_stats.inc.rs");
include!("resource_stat.inc.rs");
include!("insights.inc.rs");
include!("status_chips.inc.rs");
include!("inventory.inc.rs");
include!("router.inc.rs");
