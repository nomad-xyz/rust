//! Watcher public configuration

use crate::decl_config;

// Current watcher setup is home-centric, meaning one watcher will watch the
// home and flag fraud on any corresponding replica chains. We assume the
// watcher has permissions over connection managers on each replica chain for
// now. This is likely to change in the future.
decl_config!(Watcher {});
