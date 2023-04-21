//! Shortcuts used in the graph editor.

use ensogl::application::shortcut::ActionType::*;


// =======================================
// === Shortcuts for the graph editor. ===
// =======================================

/// The list of all shortcuts used in the graph editor.
pub const SHORTCUTS: &[(ensogl::application::shortcut::ActionType, &str, &str, &str)] = &[
    (Press, "!node_editing", "tab", "start_node_creation"),
    (Press, "!node_editing", "enter", "start_node_creation"),
    // === Drag ===
    (Press, "", "left-mouse-button", "node_press"),
    (Release, "", "left-mouse-button", "node_release"),
    (Press, "!node_editing", "backspace", "remove_selected_nodes"),
    (Press, "!node_editing", "delete", "remove_selected_nodes"),
    (Press, "has_detached_edge", "escape", "drop_dragged_edge"),
    (Press, "", "cmd g", "collapse_selected_nodes"),
    // === Visualization ===
    (Press, "!node_editing", "space", "press_visualization_visibility"),
    (DoublePress, "!node_editing", "space", "double_press_visualization_visibility"),
    (Release, "!node_editing", "space", "release_visualization_visibility"),
    (Press, "", "cmd i", "reload_visualization_registry"),
    (Press, "is_fs_visualization_displayed", "space", "close_fullscreen_visualization"),
    (Press, "", "cmd", "enable_quick_visualization_preview"),
    (Release, "", "cmd", "disable_quick_visualization_preview"),
    // === Selection ===
    (Press, "", "shift", "enable_node_multi_select"),
    (Press, "", "shift left-mouse-button", "enable_node_multi_select"),
    (Release, "", "shift", "disable_node_multi_select"),
    (Release, "", "shift left-mouse-button", "disable_node_multi_select"),
    (Press, "", "shift ctrl", "toggle_node_merge_select"),
    (Release, "", "shift ctrl", "toggle_node_merge_select"),
    (Press, "", "shift alt", "toggle_node_subtract_select"),
    (Release, "", "shift alt", "toggle_node_subtract_select"),
    (Press, "", "shift ctrl alt", "toggle_node_inverse_select"),
    (Release, "", "shift ctrl alt", "toggle_node_inverse_select"),
    // === Navigation ===
    (
        Press,
        "!is_fs_visualization_displayed",
        "ctrl space",
        "cycle_visualization_for_selected_node",
    ),
    (DoublePress, "", "left-mouse-button", "enter_hovered_node"),
    (DoublePress, "", "left-mouse-button", "start_node_creation_from_port"),
    (Press, "", "right-mouse-button", "start_node_creation_from_port"),
    (Press, "!node_editing", "cmd enter", "enter_selected_node"),
    (Press, "", "alt enter", "exit_node"),
    // === Node Editing ===
    (Press, "", "cmd", "edit_mode_on"),
    (Release, "", "cmd", "edit_mode_off"),
    (Press, "", "cmd left-mouse-button", "edit_mode_on"),
    (Release, "", "cmd left-mouse-button", "edit_mode_off"),
    (Press, "node_editing", "cmd enter", "stop_editing"),
    // === Profiling Mode ===
    (Press, "", "cmd p", "toggle_profiling_mode"),
    // === Execution Mode ===
    (Press, "", "shift ctrl e", "toggle_execution_environment"),
    // === Debug ===
    (Press, "debug_mode", "ctrl d", "debug_set_test_visualization_data_for_selected_node"),
    (Press, "debug_mode", "ctrl shift enter", "debug_push_breadcrumb"),
    (Press, "debug_mode", "ctrl shift up", "debug_pop_breadcrumb"),
    (Press, "debug_mode", "ctrl n", "add_node_at_cursor"),
];