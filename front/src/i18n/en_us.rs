use std::collections::HashMap;

pub fn get_translations() -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // 通用
    translations.insert("all".to_string(), "All".to_string());
    translations.insert("name".to_string(), "Name".to_string());
    translations.insert("mode".to_string(), "Mode".to_string());
    translations.insert("unknown".to_string(), "Unknown".to_string());
    translations.insert("online".to_string(), "Online".to_string());
    translations.insert("offline".to_string(), "Offline".to_string());
    translations.insert("None".to_string(), "None".to_string());
    translations.insert("none".to_string(), "None".to_string());
    translations.insert("Role".to_string(), "Role".to_string());
    translations.insert("never".to_string(), "Never".to_string());
    translations.insert("count".to_string(), "".to_string());

    // 硬件单位
    translations.insert("cores".to_string(), "Cores".to_string());
    translations.insert("threads".to_string(), "Threads".to_string());
    translations.insert("gb".to_string(), "GB".to_string());
    translations.insert("mhz".to_string(), "MHz".to_string());
    translations.insert("ghz".to_string(), "GHz".to_string());
    translations.insert("nics".to_string(), "NICs".to_string());

    // 硬件类别
    translations.insert("cpu_config".to_string(), "CPU Configuration".to_string());
    translations.insert(
        "memory_config".to_string(),
        "Memory Configuration".to_string(),
    );
    translations.insert("gpu_config".to_string(), "GPU Configuration".to_string());
    translations.insert(
        "storage_config".to_string(),
        "Storage Configuration".to_string(),
    );
    translations.insert(
        "network_config".to_string(),
        "Network Configuration".to_string(),
    );
    translations.insert(
        "operating_system".to_string(),
        "Operating System".to_string(),
    );
    translations.insert("server_model".to_string(), "Server Model".to_string());

    // Hardware Component Titles
    translations.insert("cpu.title".to_string(), "CPU".to_string());
    translations.insert("gpu.title".to_string(), "GPU".to_string());
    translations.insert("memory.title".to_string(), "Memory".to_string());
    translations.insert("network.title".to_string(), "Network".to_string());
    translations.insert("storage.title".to_string(), "Storage".to_string());

    // Hardware Component Titles (with hardware. prefix)
    translations.insert("hardware.cpu.title".to_string(), "CPU".to_string());
    translations.insert("hardware.gpu.title".to_string(), "GPU".to_string());
    translations.insert("hardware.memory.title".to_string(), "Memory".to_string());
    translations.insert("hardware.network.title".to_string(), "Network".to_string());
    translations.insert("hardware.storage.title".to_string(), "Storage".to_string());

    // Hardware Labels
    translations.insert("label.vendor".to_string(), "Vendor".to_string());
    translations.insert("label.model".to_string(), "Model".to_string());
    translations.insert("label.frequency".to_string(), "Frequency".to_string());
    translations.insert("label.cores".to_string(), "Cores".to_string());
    translations.insert("label.threads".to_string(), "Threads".to_string());
    translations.insert("label.device_id".to_string(), "Device ID".to_string());
    translations.insert(
        "label.driver_version".to_string(),
        "Driver Version".to_string(),
    );
    translations.insert(
        "label.serial_number".to_string(),
        "Serial Number".to_string(),
    );
    translations.insert("label.capacity".to_string(), "Capacity".to_string());
    translations.insert("label.speed".to_string(), "Speed".to_string());
    translations.insert("label.firmware".to_string(), "Firmware".to_string());
    translations.insert("label.interface".to_string(), "Interface".to_string());
    translations.insert("label.size".to_string(), "Size".to_string());
    translations.insert("label.type".to_string(), "Type".to_string());
    translations.insert("label.manufacturer".to_string(), "Manufacturer".to_string());
    translations.insert("label.temperature".to_string(), "Temperature".to_string());
    translations.insert("label.voltage".to_string(), "Voltage".to_string());
    translations.insert("label.power".to_string(), "Power".to_string());
    translations.insert("label.utilization".to_string(), "Utilization".to_string());

    // Hardware Labels (with hardware. prefix)
    translations.insert("hardware.label.vendor".to_string(), "Vendor".to_string());
    translations.insert("hardware.label.model".to_string(), "Model".to_string());
    translations.insert(
        "hardware.label.frequency".to_string(),
        "Frequency".to_string(),
    );
    translations.insert("hardware.label.cores".to_string(), "Cores".to_string());
    translations.insert("hardware.label.threads".to_string(), "Threads".to_string());
    translations.insert(
        "hardware.label.device_id".to_string(),
        "Device ID".to_string(),
    );
    translations.insert(
        "hardware.label.driver_version".to_string(),
        "Driver Version".to_string(),
    );
    translations.insert(
        "hardware.label.serial_number".to_string(),
        "Serial Number".to_string(),
    );
    translations.insert(
        "hardware.label.capacity".to_string(),
        "Capacity".to_string(),
    );
    translations.insert("hardware.label.speed".to_string(), "Speed".to_string());
    translations.insert(
        "hardware.label.firmware_version".to_string(),
        "Firmware Version".to_string(),
    );
    translations.insert("hardware.label.slot".to_string(), "Slot".to_string());
    translations.insert("hardware.label.type".to_string(), "Type".to_string());
    translations.insert(
        "hardware.label.part_number".to_string(),
        "Part Number".to_string(),
    );
    translations.insert(
        "hardware.label.memory_count".to_string(),
        "Memory Count".to_string(),
    );
    translations.insert("hardware.label.sticks".to_string(), "Sticks".to_string());
    translations.insert("hardware.label.devices".to_string(), "Devices".to_string());
    translations.insert("hardware.label.unknown".to_string(), "Unknown".to_string());
    translations.insert(
        "hardware.label.interface_name".to_string(),
        "Interface Name".to_string(),
    );
    translations.insert(
        "hardware.label.nic_type".to_string(),
        "NIC Type".to_string(),
    );
    translations.insert(
        "hardware.label.pci_slot".to_string(),
        "PCI Slot".to_string(),
    );
    translations.insert(
        "hardware.label.bandwidth".to_string(),
        "Bandwidth".to_string(),
    );
    translations.insert("hardware.label.status".to_string(), "Status".to_string());
    translations.insert("hardware.label.driver".to_string(), "Driver".to_string());
    translations.insert(
        "hardware.label.ib_node_type".to_string(),
        "IB Node Type".to_string(),
    );
    translations.insert("hardware.label.dhcp".to_string(), "DHCP".to_string());
    translations.insert(
        "hardware.label.ip_address".to_string(),
        "IP Address".to_string(),
    );
    translations.insert(
        "hardware.label.subnet_mask".to_string(),
        "Subnet Mask".to_string(),
    );
    translations.insert("hardware.label.gateway".to_string(), "Gateway".to_string());
    translations.insert("hardware.label.channel".to_string(), "Channel".to_string());
    translations.insert("hardware.label.user_id".to_string(), "User ID".to_string());
    translations.insert(
        "hardware.label.username".to_string(),
        "Username".to_string(),
    );
    translations.insert(
        "hardware.label.privilege".to_string(),
        "Privilege".to_string(),
    );

    // Hardware History
    translations.insert("history.change".to_string(), "Change".to_string());
    translations.insert("history.change_type".to_string(), "Change Type".to_string());
    translations.insert(
        "history.empty".to_string(),
        "No changes recorded".to_string(),
    );
    translations.insert(
        "history.loading".to_string(),
        "Loading changes...".to_string(),
    );
    translations.insert("history.time".to_string(), "Time".to_string());
    translations.insert("history.title".to_string(), "Hardware History".to_string());
    translations.insert(
        "history.view_details".to_string(),
        "View Details".to_string(),
    );

    // Hardware History (with hardware. prefix)
    translations.insert(
        "hardware.history.title".to_string(),
        "Hardware History".to_string(),
    );
    translations.insert("hardware.history.change".to_string(), "Change".to_string());
    translations.insert(
        "hardware.history.change_type".to_string(),
        "Change Type".to_string(),
    );
    translations.insert(
        "hardware.history.empty".to_string(),
        "No History".to_string(),
    );
    translations.insert(
        "hardware.history.loading".to_string(),
        "Loading...".to_string(),
    );
    translations.insert("hardware.history.time".to_string(), "Time".to_string());
    translations.insert(
        "hardware.history.view_details".to_string(),
        "View Details".to_string(),
    );
    translations.insert("common.refresh".to_string(), "Refresh".to_string());
    translations.insert("common.load_more".to_string(), "Load More".to_string());

    // Execution
    translations.insert(
        "execution.status.pending".to_string(),
        "Pending".to_string(),
    );
    translations.insert(
        "execution.status.running".to_string(),
        "Running".to_string(),
    );
    translations.insert(
        "execution.status.success".to_string(),
        "Success".to_string(),
    );
    translations.insert("execution.status.failed".to_string(), "Failed".to_string());
    translations.insert(
        "execution.status.partial".to_string(),
        "Partial".to_string(),
    );
    translations.insert(
        "execution.status.timeout".to_string(),
        "Timeout".to_string(),
    );
    translations.insert(
        "execution.status.expired".to_string(),
        "Expired".to_string(),
    );
    translations.insert(
        "execution.type.terminal".to_string(),
        "Terminal".to_string(),
    );
    translations.insert("execution.type.batch".to_string(), "Batch".to_string());
    translations.insert(
        "execution.actions.reset_filters".to_string(),
        "Reset filters".to_string(),
    );
    translations.insert("execution.actions.hide".to_string(), "Hide".to_string());
    translations.insert(
        "execution.actions.details".to_string(),
        "Details".to_string(),
    );
    translations.insert("execution.actions.rerun".to_string(), "Rerun".to_string());
    translations.insert(
        "execution.actions.replay".to_string(),
        "View Replay".to_string(),
    );
    translations.insert(
        "execution.history.title".to_string(),
        "Execution History".to_string(),
    );
    translations.insert(
        "execution.history.description".to_string(),
        "Unified operations history for terminal sessions and batch executions.".to_string(),
    );
    translations.insert(
        "execution.history.records_badge".to_string(),
        "{count} records".to_string(),
    );
    translations.insert(
        "execution.history.search_label".to_string(),
        "Search executions".to_string(),
    );
    translations.insert(
        "execution.history.search_placeholder".to_string(),
        "command / user / session id / client id".to_string(),
    );
    translations.insert(
        "execution.history.client_scope".to_string(),
        "Client scope".to_string(),
    );
    translations.insert("execution.history.status".to_string(), "Status".to_string());
    translations.insert(
        "execution.history.execution_type".to_string(),
        "Execution type".to_string(),
    );
    translations.insert("execution.history.from".to_string(), "From".to_string());
    translations.insert("execution.history.to".to_string(), "To".to_string());
    translations.insert(
        "execution.history.loading".to_string(),
        "Loading execution history...".to_string(),
    );
    translations.insert(
        "execution.history.empty".to_string(),
        "No execution records match the current filters.".to_string(),
    );
    translations.insert(
        "execution.history.all_clients".to_string(),
        "All clients".to_string(),
    );
    translations.insert(
        "execution.history.all_statuses".to_string(),
        "All statuses".to_string(),
    );
    translations.insert(
        "execution.history.all_execution_types".to_string(),
        "All execution types".to_string(),
    );
    translations.insert(
        "execution.history.table.time".to_string(),
        "Time".to_string(),
    );
    translations.insert(
        "execution.history.table.type".to_string(),
        "Type".to_string(),
    );
    translations.insert(
        "execution.history.table.targets".to_string(),
        "Targets".to_string(),
    );
    translations.insert(
        "execution.history.table.command".to_string(),
        "Command".to_string(),
    );
    translations.insert(
        "execution.history.table.status".to_string(),
        "Status".to_string(),
    );
    translations.insert(
        "execution.history.table.duration".to_string(),
        "Duration".to_string(),
    );
    translations.insert(
        "execution.history.table.user".to_string(),
        "User".to_string(),
    );
    translations.insert(
        "execution.history.table.actions".to_string(),
        "Actions".to_string(),
    );
    translations.insert(
        "execution.history.full_command".to_string(),
        "Full command / script".to_string(),
    );
    translations.insert(
        "execution.history.status_summary".to_string(),
        "Status summary".to_string(),
    );
    translations.insert(
        "execution.history.started".to_string(),
        "Started".to_string(),
    );
    translations.insert("execution.history.ended".to_string(), "Ended".to_string());
    translations.insert(
        "execution.history.recording".to_string(),
        "Recording".to_string(),
    );
    translations.insert(
        "execution.history.recording_available".to_string(),
        "Available".to_string(),
    );
    translations.insert(
        "execution.history.recording_missing".to_string(),
        "Not recorded".to_string(),
    );
    translations.insert(
        "execution.history.details_title".to_string(),
        "Execution details".to_string(),
    );
    translations.insert(
        "execution.history.details_description".to_string(),
        "Review the full command, output, and execution metadata in one place.".to_string(),
    );
    translations.insert(
        "execution.history.open_replay".to_string(),
        "Open replay".to_string(),
    );
    translations.insert(
        "execution.batch.title".to_string(),
        "Batch Execution".to_string(),
    );
    translations.insert("execution.batch.description".to_string(), "Fast multi-client command workspace with exact command preservation and per-client results.".to_string());
    translations.insert(
        "execution.batch.selected_badge".to_string(),
        "{count} selected".to_string(),
    );
    translations.insert(
        "execution.batch.limit_badge".to_string(),
        "Over 50-client limit".to_string(),
    );
    translations.insert(
        "execution.batch.preloaded_badge".to_string(),
        "Preloaded from history".to_string(),
    );
    translations.insert(
        "execution.batch.back_to_workspace".to_string(),
        "Back to workspace".to_string(),
    );
    translations.insert("execution.batch.targets".to_string(), "Targets".to_string());
    translations.insert("execution.batch.running".to_string(), "Running".to_string());
    translations.insert(
        "execution.batch.completed".to_string(),
        "Completed".to_string(),
    );
    translations.insert(
        "execution.batch.output_title".to_string(),
        "Per-client output".to_string(),
    );
    translations.insert(
        "execution.batch.output_description".to_string(),
        "Inspect one target at a time without losing the full batch context.".to_string(),
    );
    translations.insert(
        "execution.batch.single_output".to_string(),
        "Single node".to_string(),
    );
    translations.insert(
        "execution.batch.all_output".to_string(),
        "All nodes".to_string(),
    );
    translations.insert(
        "execution.batch.all_output_title".to_string(),
        "All node outputs".to_string(),
    );
    translations.insert(
        "execution.batch.all_output_description".to_string(),
        "Review every node result from this batch in one place.".to_string(),
    );
    translations.insert(
        "execution.batch.command_label".to_string(),
        "Command".to_string(),
    );
    translations.insert(
        "execution.batch.target_client".to_string(),
        "Target client".to_string(),
    );
    translations.insert("execution.batch.task_id".to_string(), "Task ID".to_string());
    translations.insert(
        "execution.batch.waiting_output".to_string(),
        "Waiting for client output...".to_string(),
    );
    translations.insert(
        "execution.batch.output_load_failed".to_string(),
        "Failed to load output. Switch nodes to retry.".to_string(),
    );
    translations.insert("execution.batch.no_output_terminal".to_string(), "The task has finished, but the client returned no output. For failed tasks, check the error log, command whitelist, or client execution environment.".to_string());
    translations.insert(
        "execution.batch.select_target_prompt".to_string(),
        "Select a target from the left to inspect its output.".to_string(),
    );
    translations.insert(
        "execution.batch.select_targets".to_string(),
        "Select targets".to_string(),
    );
    translations.insert(
        "execution.batch.search_label".to_string(),
        "Search clients".to_string(),
    );
    translations.insert(
        "execution.batch.search_placeholder".to_string(),
        "hostname / ip / client id / serial / location / rack".to_string(),
    );
    translations.insert(
        "execution.batch.project_filter".to_string(),
        "Project".to_string(),
    );
    translations.insert(
        "execution.batch.rack_filter".to_string(),
        "Rack / location".to_string(),
    );
    translations.insert(
        "execution.batch.all_projects".to_string(),
        "All projects".to_string(),
    );
    translations.insert(
        "execution.batch.all_racks".to_string(),
        "All racks / locations".to_string(),
    );
    translations.insert(
        "execution.batch.select_visible".to_string(),
        "Select visible".to_string(),
    );
    translations.insert(
        "execution.batch.clear_all".to_string(),
        "Clear all".to_string(),
    );
    translations.insert(
        "execution.batch.matches".to_string(),
        "{count} matches".to_string(),
    );
    translations.insert(
        "execution.batch.summary_title".to_string(),
        "Selection summary".to_string(),
    );
    translations.insert(
        "execution.batch.reduce_limit".to_string(),
        "Reduce selection to 50 or fewer".to_string(),
    );
    translations.insert("execution.batch.summary_empty".to_string(), "Choose one or more clients from the left. You can also land here from history with clients preselected.".to_string());
    translations.insert(
        "execution.batch.editor_title".to_string(),
        "Command or script".to_string(),
    );
    translations.insert("execution.batch.editor_help".to_string(), "Exact input is preserved for reruns: flags, whitespace, and multi-line script content are submitted exactly as shown below.".to_string());
    translations.insert(
        "execution.batch.editor_placeholder".to_string(),
        "Single-line command or multi-line shell script\nShift+Enter: newline\nCtrl+Enter: execute"
            .to_string(),
    );
    translations.insert(
        "execution.batch.shortcut_newline".to_string(),
        "Shift+Enter = newline".to_string(),
    );
    translations.insert(
        "execution.batch.shortcut_execute".to_string(),
        "Ctrl+Enter = execute".to_string(),
    );
    translations.insert(
        "execution.batch.submitting".to_string(),
        "Submitting...".to_string(),
    );
    translations.insert(
        "execution.batch.execute".to_string(),
        "Execute on selected clients".to_string(),
    );
    translations.insert(
        "execution.batch.validation.command_required".to_string(),
        "Please enter a command or script".to_string(),
    );
    translations.insert(
        "execution.batch.validation.targets_required".to_string(),
        "Select at least one client".to_string(),
    );
    translations.insert(
        "execution.batch.validation.limit".to_string(),
        "Batch execution is limited to 50 clients".to_string(),
    );
    translations.insert(
        "execution.batch.notifications.no_tasks".to_string(),
        "No tasks were created".to_string(),
    );
    translations.insert(
        "execution.batch.notifications.partial_failures".to_string(),
        "Batch started with {count} failed task submissions".to_string(),
    );
    translations.insert(
        "execution.batch.notifications.started".to_string(),
        "Batch execution started".to_string(),
    );
    translations.insert(
        "execution.batch.notifications.approvals_created".to_string(),
        "{count} target(s) were submitted for approval".to_string(),
    );
    translations.insert(
        "execution.replay.title".to_string(),
        "Execution Replay".to_string(),
    );
    translations.insert(
        "execution.replay.description".to_string(),
        "Review the recorded output captured for a finished or in-progress execution session."
            .to_string(),
    );
    translations.insert(
        "execution.replay.loading".to_string(),
        "Loading replay...".to_string(),
    );
    translations.insert(
        "execution.replay.load_session_error".to_string(),
        "Failed to load session".to_string(),
    );
    translations.insert(
        "execution.replay.load_cast_error".to_string(),
        "Failed to load replay file".to_string(),
    );
    translations.insert(
        "execution.replay.load_output_error".to_string(),
        "Failed to load replay output".to_string(),
    );
    translations.insert(
        "execution.replay.command".to_string(),
        "Command".to_string(),
    );
    translations.insert("execution.replay.status".to_string(), "Status".to_string());
    translations.insert(
        "execution.replay.output_title".to_string(),
        "Recorded output".to_string(),
    );
    translations.insert(
        "execution.replay.output_description".to_string(),
        "Replay data is shown exactly as captured by the session recorder.".to_string(),
    );
    translations.insert(
        "execution.replay.back".to_string(),
        "Back to history".to_string(),
    );
    translations.insert(
        "execution.replay.read_only".to_string(),
        "Read-only".to_string(),
    );
    translations.insert(
        "execution.replay.waiting".to_string(),
        "Waiting for replay data...".to_string(),
    );
    translations.insert("execution.replay.play".to_string(), "Play".to_string());
    translations.insert("execution.replay.pause".to_string(), "Pause".to_string());
    translations.insert(
        "execution.replay.restart".to_string(),
        "Restart replay".to_string(),
    );
    translations.insert(
        "execution.settings.title".to_string(),
        "Remote Execution Settings".to_string(),
    );
    translations.insert(
        "execution.settings.description".to_string(),
        "Control whether operators are allowed to submit remote commands to connected agents."
            .to_string(),
    );
    translations.insert(
        "execution.settings.enabled".to_string(),
        "Enabled".to_string(),
    );
    translations.insert(
        "execution.settings.disabled".to_string(),
        "Disabled".to_string(),
    );
    translations.insert(
        "execution.settings.loading".to_string(),
        "Loading remote execution settings...".to_string(),
    );
    translations.insert(
        "execution.settings.toggle_title".to_string(),
        "Enable remote command execution".to_string(),
    );
    translations.insert("execution.settings.toggle_description".to_string(), "When enabled, administrators can dispatch remote commands from the CMDB execution workflows.".to_string());
    translations.insert(
        "execution.settings.allow".to_string(),
        "Allow remote execution".to_string(),
    );
    translations.insert(
        "execution.settings.block".to_string(),
        "Block remote execution".to_string(),
    );
    translations.insert("execution.settings.warning".to_string(), "Disabling this switch should immediately prevent new command submissions while preserving existing history and recordings.".to_string());
    translations.insert(
        "execution.settings.saving".to_string(),
        "Saving...".to_string(),
    );
    translations.insert(
        "execution.settings.save".to_string(),
        "Save settings".to_string(),
    );
    translations.insert(
        "execution.settings.updated".to_string(),
        "Remote execution settings updated".to_string(),
    );
    translations.insert("execution.actions.cancel".to_string(), "Cancel".to_string());
    translations.insert("execution.danger.safe".to_string(), "Safe".to_string());
    translations.insert(
        "execution.danger.warning".to_string(),
        "Warning".to_string(),
    );
    translations.insert(
        "execution.danger.blocked".to_string(),
        "Blocked".to_string(),
    );
    translations.insert(
        "execution.terminal.title".to_string(),
        "Web Terminal".to_string(),
    );
    translations.insert(
        "execution.terminal.description".to_string(),
        "Single-client live command workspace with streamed output.".to_string(),
    );
    translations.insert("execution.terminal.auto_attach_hint".to_string(), "This page automatically reuses an active terminal session. If no reusable session exists, a new one is created for you.".to_string());
    translations.insert("execution.terminal.auto_create_hint".to_string(), "Each page visit creates a terminal session dedicated to this browser, and the session is closed automatically when you leave.".to_string());
    translations.insert(
        "execution.terminal.console_title".to_string(),
        "Terminal Console".to_string(),
    );
    translations.insert("execution.terminal.console_description".to_string(), "The page auto-attaches to an active session or creates a new one, so this panel only keeps sizing, refresh, and session switching controls.".to_string());
    translations.insert("execution.terminal.console_new_session_description".to_string(), "This page owns its own terminal session, so this panel only keeps sizing, reconnect, and history controls.".to_string());
    translations.insert("execution.terminal.mode_hint".to_string(), "Restricted terminal is not a view-only console. It still accepts input, but only commands explicitly allowed by terminal policy can run.".to_string());
    translations.insert(
        "execution.terminal.switchable_sessions".to_string(),
        "Switchable sessions".to_string(),
    );
    translations.insert(
        "execution.terminal.live_shell".to_string(),
        "Live Shell".to_string(),
    );
    translations.insert(
        "execution.terminal.enabled".to_string(),
        "Remote exec enabled".to_string(),
    );
    translations.insert(
        "execution.terminal.disabled".to_string(),
        "Remote exec disabled".to_string(),
    );
    translations.insert(
        "execution.terminal.view_history".to_string(),
        "View history".to_string(),
    );
    translations.insert(
        "execution.terminal.reconnect".to_string(),
        "Reconnect stream".to_string(),
    );
    translations.insert(
        "execution.terminal.close".to_string(),
        "Close session".to_string(),
    );
    translations.insert(
        "execution.terminal.target_client".to_string(),
        "Target client".to_string(),
    );
    translations.insert(
        "execution.terminal.hostname".to_string(),
        "Hostname".to_string(),
    );
    translations.insert("execution.terminal.ip".to_string(), "IP".to_string());
    translations.insert("execution.terminal.os".to_string(), "OS".to_string());
    translations.insert(
        "execution.terminal.current_session".to_string(),
        "Current session".to_string(),
    );
    translations.insert(
        "execution.terminal.available_sessions".to_string(),
        "Available sessions".to_string(),
    );
    translations.insert(
        "execution.terminal.no_available_sessions".to_string(),
        "No reusable terminal sessions were found for this client.".to_string(),
    );
    translations.insert(
        "execution.terminal.attach".to_string(),
        "Attach".to_string(),
    );
    translations.insert(
        "execution.terminal.attached".to_string(),
        "Attached".to_string(),
    );
    translations.insert(
        "execution.terminal.session_id".to_string(),
        "Session ID".to_string(),
    );
    translations.insert(
        "execution.terminal.created_at".to_string(),
        "Created at".to_string(),
    );
    translations.insert(
        "execution.terminal.last_activity".to_string(),
        "Last activity".to_string(),
    );
    translations.insert(
        "execution.terminal.last_heartbeat".to_string(),
        "Last heartbeat".to_string(),
    );
    translations.insert(
        "execution.terminal.lease".to_string(),
        "Lease expires".to_string(),
    );
    translations.insert(
        "execution.terminal.live_terminal".to_string(),
        "Live terminal".to_string(),
    );
    translations.insert("execution.terminal.live_terminal_description".to_string(), "Open a real interactive shell in the browser. Keyboard input is streamed directly to the remote PTY.".to_string());
    translations.insert(
        "execution.terminal.live_terminal_empty".to_string(),
        "Open or attach to a terminal session to start interactive work.".to_string(),
    );
    translations.insert("execution.terminal.browser_terminal_note".to_string(), "This view now behaves like a browser terminal: keystrokes, paste, and Enter are sent as terminal input instead of submitting a textarea as one batch command.".to_string());
    translations.insert(
        "execution.terminal.open_terminal".to_string(),
        "Open terminal".to_string(),
    );
    translations.insert(
        "execution.terminal.opening_terminal".to_string(),
        "Opening terminal...".to_string(),
    );
    translations.insert(
        "execution.terminal.connecting".to_string(),
        "Connecting".to_string(),
    );
    translations.insert(
        "execution.terminal.connected".to_string(),
        "Connected".to_string(),
    );
    translations.insert(
        "execution.terminal.disconnected".to_string(),
        "Disconnected".to_string(),
    );
    translations.insert(
        "execution.terminal.preloaded_command".to_string(),
        "Preloaded command".to_string(),
    );
    translations.insert(
        "execution.terminal.notifications.opened".to_string(),
        "Terminal session opened.".to_string(),
    );
    translations.insert(
        "execution.terminal.task_id".to_string(),
        "Task ID".to_string(),
    );
    translations.insert(
        "execution.terminal.exit_code".to_string(),
        "Exit code".to_string(),
    );
    translations.insert("execution.terminal.shell".to_string(), "Shell".to_string());
    translations.insert(
        "execution.terminal.size".to_string(),
        "Terminal size".to_string(),
    );
    translations.insert("execution.terminal.cols".to_string(), "Columns".to_string());
    translations.insert("execution.terminal.rows".to_string(), "Rows".to_string());
    translations.insert("execution.terminal.state".to_string(), "State".to_string());
    translations.insert(
        "execution.terminal.state_closed".to_string(),
        "Closed".to_string(),
    );
    translations.insert(
        "execution.terminal.mode_read_only".to_string(),
        "Read-only".to_string(),
    );
    translations.insert(
        "execution.terminal.mode_read_write".to_string(),
        "Read-write".to_string(),
    );
    translations.insert(
        "execution.terminal.mode_restricted".to_string(),
        "Restricted terminal".to_string(),
    );
    translations.insert(
        "execution.terminal.mode_standard".to_string(),
        "Standard terminal".to_string(),
    );
    translations.insert(
        "permissions.web_terminal.page_description".to_string(),
        "Control Web Terminal access mode, session constraints, and restricted command rules."
            .to_string(),
    );
    translations.insert("permissions.web_terminal.modal_description".to_string(), "Terminal policy is separate from command execution policy; it only constrains Web Terminal sessions and restricted command behavior.".to_string());
    translations.insert(
        "execution.terminal.close_reason".to_string(),
        "Close reason".to_string(),
    );
    translations.insert(
        "execution.terminal.no_session".to_string(),
        "No command has been executed in this terminal yet.".to_string(),
    );
    translations.insert(
        "execution.terminal.run_command".to_string(),
        "Run command".to_string(),
    );
    translations.insert(
        "execution.terminal.command_label".to_string(),
        "Command or script".to_string(),
    );
    translations.insert(
        "execution.terminal.command_placeholder".to_string(),
        "Supports exact command text, flags, and multi-line scripts.".to_string(),
    );
    translations.insert(
        "execution.terminal.timeout".to_string(),
        "Timeout (seconds)".to_string(),
    );
    translations.insert("execution.terminal.pinned_note".to_string(), "The terminal stays pinned to this client. New commands are blocked until the current command finishes.".to_string());
    translations.insert(
        "execution.terminal.submitting".to_string(),
        "Submitting...".to_string(),
    );
    translations.insert(
        "execution.terminal.running".to_string(),
        "Running...".to_string(),
    );
    translations.insert(
        "execution.terminal.execute".to_string(),
        "Run in terminal".to_string(),
    );
    translations.insert(
        "execution.terminal.output_title".to_string(),
        "Live output".to_string(),
    );
    translations.insert(
        "execution.terminal.output_description".to_string(),
        "Streaming output from the current terminal command.".to_string(),
    );
    translations.insert(
        "execution.terminal.current_command".to_string(),
        "Current command".to_string(),
    );
    translations.insert(
        "execution.terminal.output_empty".to_string(),
        "Run a command to open the live terminal output stream.".to_string(),
    );
    translations.insert(
        "execution.terminal.output_waiting".to_string(),
        "Waiting for output...".to_string(),
    );
    translations.insert(
        "execution.terminal.client_load_failed".to_string(),
        "Unable to load the selected client for terminal access.".to_string(),
    );
    translations.insert(
        "execution.terminal.validation.command_required".to_string(),
        "Please enter a command".to_string(),
    );
    translations.insert(
        "execution.terminal.notifications.submitted".to_string(),
        "Command submitted".to_string(),
    );
    translations.insert(
        "execution.terminal.notifications.closed".to_string(),
        "Terminal session close requested".to_string(),
    );
    translations.insert(
        "execution.terminal.danger_title".to_string(),
        "Dangerous command detected".to_string(),
    );
    translations.insert(
        "execution.terminal.matched_rule".to_string(),
        "Matched rule: {rule}".to_string(),
    );
    translations.insert(
        "execution.terminal.confirm_prefix".to_string(),
        "Type ".to_string(),
    );
    translations.insert(
        "execution.terminal.confirm_suffix".to_string(),
        " to continue.".to_string(),
    );
    translations.insert(
        "execution.terminal.confirm_placeholder".to_string(),
        "CONFIRM".to_string(),
    );
    translations.insert(
        "execution.terminal.execute_anyway".to_string(),
        "Execute anyway".to_string(),
    );
    translations.insert(
        "execution.terminal.disabled_message".to_string(),
        "Remote command execution is currently disabled by policy.".to_string(),
    );
    translations.insert(
        "execution.terminal.websocket_error".to_string(),
        "Terminal connection error. Please try again.".to_string(),
    );
    translations.insert(
        "execution.ops.summary.exec_policies".to_string(),
        "Execution policies".to_string(),
    );
    translations.insert(
        "execution.ops.summary.web_terminal_policies".to_string(),
        "Terminal policies".to_string(),
    );
    translations.insert(
        "execution.ops.summary.pending_approvals".to_string(),
        "Pending approvals".to_string(),
    );
    translations.insert(
        "execution.ops.summary.active_terminal_sessions".to_string(),
        "Active terminal sessions".to_string(),
    );
    translations.insert(
        "execution.ops.overview.title".to_string(),
        "Remote Execution Operations".to_string(),
    );
    translations.insert("execution.ops.overview.description".to_string(), "Review approval backlog, terminal activity, stale-session threshold, and cast storage in one place.".to_string());
    translations.insert(
        "execution.ops.overview.total_sessions".to_string(),
        "Total terminal sessions".to_string(),
    );
    translations.insert(
        "execution.ops.overview.active_clients".to_string(),
        "Active clients".to_string(),
    );
    translations.insert(
        "execution.ops.overview.pending_active".to_string(),
        "Pending / active".to_string(),
    );
    translations.insert(
        "execution.ops.overview.stale_threshold".to_string(),
        "Stale threshold".to_string(),
    );
    translations.insert(
        "execution.ops.overview.cast_storage_dir".to_string(),
        "Cast storage directory".to_string(),
    );
    translations.insert(
        "execution.ops.actions.refresh".to_string(),
        "Refresh ops data".to_string(),
    );
    translations.insert(
        "execution.ops.actions.force_close".to_string(),
        "Force close".to_string(),
    );
    translations.insert(
        "execution.ops.cleanup.retention_days".to_string(),
        "Retention days".to_string(),
    );
    translations.insert(
        "execution.ops.cleanup.run".to_string(),
        "Run cast cleanup".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_governance.title".to_string(),
        "Terminal session governance".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_governance.empty".to_string(),
        "No terminal sessions currently exist".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_governance.ended".to_string(),
        "Ended".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_state.pending".to_string(),
        "Pending".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_state.active".to_string(),
        "Active".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_state.closed".to_string(),
        "Closed".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_state.failed".to_string(),
        "Failed".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_mode.read_only".to_string(),
        "Read-only".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_mode.read_write".to_string(),
        "Read-write".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_mode.restricted".to_string(),
        "Restricted terminal".to_string(),
    );
    translations.insert(
        "execution.ops.terminal_mode.standard".to_string(),
        "Standard terminal".to_string(),
    );
    translations.insert(
        "execution.ops.table.session".to_string(),
        "Session".to_string(),
    );
    translations.insert(
        "execution.ops.table.client".to_string(),
        "Client".to_string(),
    );
    translations.insert("execution.ops.table.user".to_string(), "User".to_string());
    translations.insert("execution.ops.table.mode".to_string(), "Mode".to_string());
    translations.insert("execution.ops.table.state".to_string(), "State".to_string());
    translations.insert(
        "execution.ops.table.last_activity".to_string(),
        "Last activity".to_string(),
    );
    translations.insert(
        "execution.ops.table.last_heartbeat".to_string(),
        "Last heartbeat".to_string(),
    );
    translations.insert(
        "execution.ops.table.close_reason".to_string(),
        "Close reason".to_string(),
    );
    translations.insert(
        "execution.ops.table.operations".to_string(),
        "Actions".to_string(),
    );
    translations.insert(
        "execution.ops.messages.cast_cleanup_prefix".to_string(),
        "Cast cleanup finished. Deleted".to_string(),
    );
    translations.insert(
        "execution.ops.messages.cast_cleanup_suffix".to_string(),
        "history files".to_string(),
    );
    translations.insert(
        "execution.ops.messages.session_closed_prefix".to_string(),
        "Terminal session".to_string(),
    );
    translations.insert(
        "execution.ops.messages.session_closed_suffix".to_string(),
        "has been closed".to_string(),
    );
    translations.insert("execution.ops.none".to_string(), "None".to_string());
    translations.insert(
        "execution.history.output_title".to_string(),
        "Execution output".to_string(),
    );
    translations.insert(
        "execution.history.output_pending".to_string(),
        "Execution is still running. Logs will appear here as they arrive.".to_string(),
    );
    translations.insert(
        "execution.history.output_empty".to_string(),
        "No execution output is available.".to_string(),
    );
    translations.insert("execution.history.output_replay_hint".to_string(), "This terminal session wrote its output to a replay file. Use Replay to inspect the full terminal transcript.".to_string());
    translations.insert(
        "permissions.manage.title".to_string(),
        "Permission Rules and Groups".to_string(),
    );
    translations.insert(
        "permissions.manage.tab.rules".to_string(),
        "Rules".to_string(),
    );
    translations.insert(
        "permissions.manage.tab.groups".to_string(),
        "Groups".to_string(),
    );
    translations.insert(
        "permissions.manage.loading".to_string(),
        "Loading permission configuration...".to_string(),
    );
    translations.insert(
        "permissions.manage.empty_rules".to_string(),
        "No permission rules yet".to_string(),
    );
    translations.insert(
        "permissions.manage.empty_groups".to_string(),
        "No permission groups yet".to_string(),
    );
    translations.insert(
        "permissions.manage.overview.rules".to_string(),
        "Rules".to_string(),
    );
    translations.insert(
        "permissions.manage.overview.groups".to_string(),
        "Groups".to_string(),
    );
    translations.insert(
        "permissions.manage.overview.exec_policies".to_string(),
        "Execution policies".to_string(),
    );
    translations.insert(
        "permissions.manage.overview.web_terminal_policies".to_string(),
        "Terminal policies".to_string(),
    );
    translations.insert(
        "permissions.manage.rules.title".to_string(),
        "Permission Rules".to_string(),
    );
    translations.insert(
        "permissions.manage.rules.description".to_string(),
        "Maintain subjects, resources, and action scopes in one place.".to_string(),
    );
    translations.insert(
        "permissions.manage.groups.title".to_string(),
        "Permission Groups".to_string(),
    );
    translations.insert(
        "permissions.manage.groups.description".to_string(),
        "Manage user groups for finer-grained authorization and future policy matching."
            .to_string(),
    );
    translations.insert(
        "permissions.manage.subject.role".to_string(),
        "Role".to_string(),
    );
    translations.insert(
        "permissions.manage.subject.user".to_string(),
        "User".to_string(),
    );
    translations.insert(
        "permissions.manage.subject.group".to_string(),
        "Group".to_string(),
    );
    translations.insert(
        "permissions.manage.resource.client".to_string(),
        "Client".to_string(),
    );
    translations.insert(
        "permissions.manage.resource.component".to_string(),
        "Component".to_string(),
    );
    translations.insert(
        "permissions.manage.resource.rack".to_string(),
        "Rack".to_string(),
    );
    translations.insert(
        "permissions.manage.resource.person".to_string(),
        "Person".to_string(),
    );
    translations.insert(
        "permissions.manage.resource.project".to_string(),
        "Project".to_string(),
    );
    translations.insert(
        "permissions.manage.resource.dictionary".to_string(),
        "Dictionary".to_string(),
    );
    translations.insert(
        "permissions.manage.resource.command".to_string(),
        "Command Execution".to_string(),
    );
    translations.insert(
        "permissions.manage.resource.user".to_string(),
        "User Management".to_string(),
    );
    translations.insert(
        "permissions.manage.action.view".to_string(),
        "View".to_string(),
    );
    translations.insert(
        "permissions.manage.action.create".to_string(),
        "Create".to_string(),
    );
    translations.insert(
        "permissions.manage.action.update".to_string(),
        "Update".to_string(),
    );
    translations.insert(
        "permissions.manage.action.delete".to_string(),
        "Delete".to_string(),
    );
    translations.insert(
        "permissions.manage.constraint.unset".to_string(),
        "Unset".to_string(),
    );
    translations.insert(
        "permissions.manage.constraint.all".to_string(),
        "All resources".to_string(),
    );
    translations.insert(
        "permissions.manage.constraint.owned".to_string(),
        "Owned resources only".to_string(),
    );
    translations.insert(
        "permissions.manage.constraint.project_scope".to_string(),
        "Specific projects".to_string(),
    );
    translations.insert(
        "permissions.manage.constraint.project_list".to_string(),
        "Projects".to_string(),
    );
    translations.insert(
        "permissions.manage.constraint.tag_scope".to_string(),
        "Specific tags".to_string(),
    );
    translations.insert(
        "permissions.manage.constraint.tag_list".to_string(),
        "Tags".to_string(),
    );
    translations.insert(
        "permissions.manage.constraint.none".to_string(),
        "No access scope".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.id".to_string(),
        "Rule ID".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.subject_type".to_string(),
        "Subject type".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.subject_id".to_string(),
        "Subject ID".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.resource_type".to_string(),
        "Resource type".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.actions".to_string(),
        "Actions".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.actions_help".to_string(),
        "Select the actions this rule allows. Multiple choices are supported.".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.constraint".to_string(),
        "Scope".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.constraint_values_placeholder".to_string(),
        "Separate multiple values with commas, e.g. project-a, project-b".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.constraint_help".to_string(),
        "Choose the resource scope. For specific projects or tags, enter the values below."
            .to_string(),
    );
    translations.insert(
        "permissions.manage.rule.priority".to_string(),
        "Priority".to_string(),
    );
    translations.insert(
        "permissions.manage.rule.constraint_invalid".to_string(),
        "Constraint JSON could not be parsed".to_string(),
    );
    translations.insert(
        "permissions.manage.group.id".to_string(),
        "Group ID".to_string(),
    );
    translations.insert(
        "permissions.manage.group.name".to_string(),
        "Group name".to_string(),
    );
    translations.insert(
        "permissions.manage.group.members".to_string(),
        "Member IDs".to_string(),
    );
    translations.insert(
        "permissions.manage.group.members_help".to_string(),
        "Use commas or new lines to separate multiple member IDs.".to_string(),
    );
    translations.insert("permissions.manage.table.id".to_string(), "ID".to_string());
    translations.insert(
        "permissions.manage.table.subject".to_string(),
        "Subject".to_string(),
    );
    translations.insert(
        "permissions.manage.table.resource".to_string(),
        "Resource".to_string(),
    );
    translations.insert(
        "permissions.manage.table.actions".to_string(),
        "Actions".to_string(),
    );
    translations.insert(
        "permissions.manage.table.constraint".to_string(),
        "Constraint".to_string(),
    );
    translations.insert(
        "permissions.manage.table.priority".to_string(),
        "Priority".to_string(),
    );
    translations.insert(
        "permissions.manage.table.operations".to_string(),
        "Actions".to_string(),
    );
    translations.insert(
        "permissions.manage.table.name".to_string(),
        "Name".to_string(),
    );
    translations.insert(
        "permissions.manage.table.member_count".to_string(),
        "Members".to_string(),
    );
    translations.insert(
        "permissions.manage.table.members".to_string(),
        "Member list".to_string(),
    );
    translations.insert(
        "permissions.manage.actions.cancel".to_string(),
        "Cancel".to_string(),
    );
    translations.insert(
        "permissions.manage.actions.new_rule".to_string(),
        "New rule".to_string(),
    );
    translations.insert(
        "permissions.manage.actions.save_rule".to_string(),
        "Save rule".to_string(),
    );
    translations.insert(
        "permissions.manage.actions.update_rule".to_string(),
        "Update rule".to_string(),
    );
    translations.insert(
        "permissions.manage.actions.new_group".to_string(),
        "New group".to_string(),
    );
    translations.insert(
        "permissions.manage.actions.save_group".to_string(),
        "Save group".to_string(),
    );
    translations.insert(
        "permissions.manage.actions.update_group".to_string(),
        "Update group".to_string(),
    );
    translations.insert(
        "permissions.manage.actions.edit".to_string(),
        "Edit".to_string(),
    );
    translations.insert(
        "permissions.manage.actions.delete".to_string(),
        "Delete".to_string(),
    );
    translations.insert(
        "permissions.manage.messages.rule_saved".to_string(),
        "Permission rule saved".to_string(),
    );
    translations.insert(
        "permissions.manage.messages.rule_deleted".to_string(),
        "Permission rule deleted".to_string(),
    );
    translations.insert(
        "permissions.manage.messages.group_saved".to_string(),
        "Permission group saved".to_string(),
    );
    translations.insert(
        "permissions.manage.messages.group_deleted".to_string(),
        "Permission group deleted".to_string(),
    );
    translations.insert(
        "permissions.manage.messages.default_protected".to_string(),
        "Default rules or groups cannot be deleted directly.".to_string(),
    );
    translations.insert(
        "permissions.manage.labels.default".to_string(),
        "Default".to_string(),
    );
    translations.insert(
        "permissions.approvals.title".to_string(),
        "Approvals".to_string(),
    );
    translations.insert(
        "permissions.approvals.tab.pending".to_string(),
        "Pending".to_string(),
    );
    translations.insert(
        "permissions.approvals.tab.mine".to_string(),
        "Mine".to_string(),
    );
    translations.insert(
        "permissions.approvals.loading".to_string(),
        "Loading...".to_string(),
    );
    translations.insert(
        "permissions.approvals.empty".to_string(),
        "No approvals".to_string(),
    );
    translations.insert("permissions.approvals.none".to_string(), "None".to_string());
    translations.insert(
        "permissions.approvals.summary.total".to_string(),
        "Total".to_string(),
    );
    translations.insert(
        "permissions.approvals.summary.pending".to_string(),
        "Pending".to_string(),
    );
    translations.insert(
        "permissions.approvals.summary.approved".to_string(),
        "Approved".to_string(),
    );
    translations.insert(
        "permissions.approvals.summary.rejected".to_string(),
        "Rejected".to_string(),
    );
    translations.insert(
        "permissions.approvals.summary.expired".to_string(),
        "Expired".to_string(),
    );
    translations.insert(
        "permissions.approvals.summary.executed".to_string(),
        "Executed".to_string(),
    );
    translations.insert(
        "permissions.approvals.status.pending".to_string(),
        "Pending".to_string(),
    );
    translations.insert(
        "permissions.approvals.status.approved".to_string(),
        "Approved".to_string(),
    );
    translations.insert(
        "permissions.approvals.status.rejected".to_string(),
        "Rejected".to_string(),
    );
    translations.insert(
        "permissions.approvals.status.expired".to_string(),
        "Expired".to_string(),
    );
    translations.insert(
        "permissions.approvals.policy_type.command".to_string(),
        "Command execution".to_string(),
    );
    translations.insert(
        "permissions.approvals.table.id".to_string(),
        "ID".to_string(),
    );
    translations.insert(
        "permissions.approvals.table.type".to_string(),
        "Type".to_string(),
    );
    translations.insert(
        "permissions.approvals.table.requester".to_string(),
        "Requester".to_string(),
    );
    translations.insert(
        "permissions.approvals.table.client".to_string(),
        "Client".to_string(),
    );
    translations.insert(
        "permissions.approvals.table.command".to_string(),
        "Command".to_string(),
    );
    translations.insert(
        "permissions.approvals.table.task".to_string(),
        "Task".to_string(),
    );
    translations.insert(
        "permissions.approvals.table.status".to_string(),
        "Status".to_string(),
    );
    translations.insert(
        "permissions.approvals.table.created_at".to_string(),
        "Created at".to_string(),
    );
    translations.insert(
        "permissions.approvals.table.operations".to_string(),
        "Actions".to_string(),
    );
    translations.insert(
        "permissions.approvals.actions.go_history".to_string(),
        "Go to execution history".to_string(),
    );
    translations.insert(
        "permissions.approvals.actions.approve".to_string(),
        "Approve".to_string(),
    );
    translations.insert(
        "permissions.approvals.actions.reject".to_string(),
        "Reject".to_string(),
    );
    translations.insert(
        "permissions.approvals.messages.approved_created_task".to_string(),
        "Approved and created task".to_string(),
    );
    translations.insert(
        "permissions.approvals.messages.approved_updated".to_string(),
        "Approved and updated approval".to_string(),
    );
    translations.insert(
        "permissions.approvals.messages.approved".to_string(),
        "Approved".to_string(),
    );
    translations.insert(
        "permissions.approvals.messages.rejected".to_string(),
        "Rejected".to_string(),
    );
    translations.insert(
        "permissions.approvals.messages.task_created".to_string(),
        "Created task".to_string(),
    );

    // IPMI
    translations.insert(
        "ipmi.access_denied".to_string(),
        "Access Denied".to_string(),
    );
    translations.insert(
        "ipmi.not_available".to_string(),
        "Not Available".to_string(),
    );
    translations.insert("ipmi.status_online".to_string(), "Online".to_string());
    translations.insert("ipmi.status_offline".to_string(), "Offline".to_string());
    translations.insert("ipmi.status_unknown".to_string(), "Unknown".to_string());
    translations.insert(
        "ipmi.privilege_admin".to_string(),
        "Administrator".to_string(),
    );
    translations.insert("ipmi.privilege_user".to_string(), "User".to_string());
    translations.insert(
        "ipmi.privilege_operator".to_string(),
        "Operator".to_string(),
    );
    translations.insert(
        "ipmi.privilege_callback".to_string(),
        "Callback".to_string(),
    );
    translations.insert("ipmi.users".to_string(), "Users".to_string());
    translations.insert("ipmi.username".to_string(), "Username".to_string());
    translations.insert("ipmi.password".to_string(), "Password".to_string());
    translations.insert("ipmi.channel".to_string(), "Channel".to_string());
    translations.insert("ipmi.ip_address".to_string(), "IP Address".to_string());
    translations.insert("ipmi.mac_address".to_string(), "MAC Address".to_string());
    translations.insert("ipmi.netmask".to_string(), "Netmask".to_string());
    translations.insert("ipmi.gateway".to_string(), "Gateway".to_string());

    // IPMI (with hardware. prefix)
    translations.insert("hardware.ipmi.users".to_string(), "Users".to_string());
    translations.insert(
        "hardware.ipmi.status_available".to_string(),
        "Available".to_string(),
    );
    translations.insert(
        "hardware.ipmi.status_error".to_string(),
        "Error".to_string(),
    );
    translations.insert(
        "hardware.ipmi.status_not_configured".to_string(),
        "Not Configured".to_string(),
    );
    translations.insert(
        "hardware.ipmi.status_not_available".to_string(),
        "Not Available".to_string(),
    );
    translations.insert(
        "hardware.ipmi.status_access_denied".to_string(),
        "Access Denied".to_string(),
    );
    translations.insert(
        "hardware.ipmi.not_configured".to_string(),
        "IPMI Not Configured".to_string(),
    );
    translations.insert(
        "hardware.ipmi.not_available".to_string(),
        "IPMI Not Available".to_string(),
    );
    translations.insert(
        "hardware.ipmi.access_denied".to_string(),
        "IPMI Access Denied".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_callback".to_string(),
        "Callback".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_user".to_string(),
        "User".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_operator".to_string(),
        "Operator".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_admin".to_string(),
        "Administrator".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_no_access".to_string(),
        "No Access".to_string(),
    );

    // Status (with prefix to avoid conflicts)
    translations.insert("status.online".to_string(), "Online".to_string());
    translations.insert("status.offline".to_string(), "Offline".to_string());
    translations.insert("status.enabled".to_string(), "Enabled".to_string());
    translations.insert("status.disabled".to_string(), "Disabled".to_string());
    translations.insert("status.active".to_string(), "Active".to_string());
    translations.insert("status.inactive".to_string(), "Inactive".to_string());
    translations.insert("status.unknown".to_string(), "Unknown".to_string());
    translations.insert("status.available".to_string(), "Available".to_string());
    translations.insert("status.unavailable".to_string(), "Unavailable".to_string());

    // Status (with hardware. prefix)
    translations.insert("hardware.status.online".to_string(), "Online".to_string());
    translations.insert("hardware.status.offline".to_string(), "Offline".to_string());
    translations.insert("hardware.status.enabled".to_string(), "Enabled".to_string());
    translations.insert(
        "hardware.status.disabled".to_string(),
        "Disabled".to_string(),
    );

    // Change Types
    translations.insert("change.added".to_string(), "Added".to_string());
    translations.insert("change.removed".to_string(), "Removed".to_string());
    translations.insert("change.modified".to_string(), "Modified".to_string());
    translations.insert("change.upgraded".to_string(), "Upgraded".to_string());
    translations.insert("change.downgraded".to_string(), "Downgraded".to_string());
    translations.insert("change.replaced".to_string(), "Replaced".to_string());
    translations.insert("change.migrated".to_string(), "Migrated".to_string());

    // Change Types (with hardware. prefix)
    translations.insert("hardware.change.added".to_string(), "Added".to_string());
    translations.insert("hardware.change.removed".to_string(), "Removed".to_string());
    translations.insert(
        "hardware.change.modified".to_string(),
        "Modified".to_string(),
    );
    translations.insert(
        "hardware.change.upgraded".to_string(),
        "Upgraded".to_string(),
    );
    translations.insert(
        "hardware.change.downgraded".to_string(),
        "Downgraded".to_string(),
    );

    // Network Configuration (with hardware. prefix)
    translations.insert(
        "hardware.network.config".to_string(),
        "Network Configuration".to_string(),
    );
    translations.insert(
        "hardware.network.ipv4_config".to_string(),
        "IPv4 Configuration".to_string(),
    );
    translations.insert(
        "hardware.network.ipv6_config".to_string(),
        "IPv6 Configuration".to_string(),
    );
    translations.insert(
        "hardware.network.bonding_slaves".to_string(),
        "Bonding Slaves".to_string(),
    );

    // Storage (with hardware. prefix)
    translations.insert(
        "hardware.storage.partitions".to_string(),
        "Partitions".to_string(),
    );

    // Network Configuration
    translations.insert(
        "network.bonding_slaves".to_string(),
        "Bonding Slaves".to_string(),
    );
    translations.insert(
        "network.config".to_string(),
        "Network Configuration".to_string(),
    );
    translations.insert(
        "network.ipv4_config".to_string(),
        "IPv4 Configuration".to_string(),
    );
    translations.insert(
        "network.ipv6_config".to_string(),
        "IPv6 Configuration".to_string(),
    );
    translations.insert("network.mac_address".to_string(), "MAC Address".to_string());
    translations.insert("network.ip_address".to_string(), "IP Address".to_string());
    translations.insert("network.subnet_mask".to_string(), "Subnet Mask".to_string());
    translations.insert("network.gateway".to_string(), "Gateway".to_string());
    translations.insert("network.dns_servers".to_string(), "DNS Servers".to_string());
    translations.insert("network.speed".to_string(), "Speed".to_string());
    translations.insert("network.duplex".to_string(), "Duplex".to_string());
    translations.insert("network.mtu".to_string(), "MTU".to_string());
    translations.insert("network.bond_mode".to_string(), "Bond Mode".to_string());
    translations.insert("network.vlan".to_string(), "VLAN".to_string());
    translations.insert("network.bridge".to_string(), "Bridge".to_string());

    // Storage
    translations.insert("storage.partitions".to_string(), "Partitions".to_string());
    translations.insert("storage.partition".to_string(), "Partition".to_string());
    translations.insert("storage.mount_point".to_string(), "Mount Point".to_string());
    translations.insert("storage.file_system".to_string(), "File System".to_string());
    translations.insert("storage.used".to_string(), "Used".to_string());
    translations.insert("storage.available".to_string(), "Available".to_string());
    translations.insert("storage.usage_percent".to_string(), "Usage %".to_string());
    translations.insert("storage.disk_type".to_string(), "Disk Type".to_string());
    translations.insert(
        "storage.rotational_speed".to_string(),
        "Rotational Speed".to_string(),
    );
    translations.insert("storage.form_factor".to_string(), "Form Factor".to_string());
    translations.insert(
        "storage.smart_status".to_string(),
        "SMART Status".to_string(),
    );

    // Memory
    translations.insert(
        "memory.modules_detail".to_string(),
        "Memory Modules Detail".to_string(),
    );
    translations.insert("memory.module".to_string(), "Memory Module".to_string());
    translations.insert("memory.type".to_string(), "Type".to_string());
    translations.insert("memory.speed".to_string(), "Speed".to_string());
    translations.insert("memory.size".to_string(), "Size".to_string());
    translations.insert("memory.bank_label".to_string(), "Bank Label".to_string());
    translations.insert(
        "memory.manufacturer".to_string(),
        "Manufacturer".to_string(),
    );
    translations.insert(
        "memory.serial_number".to_string(),
        "Serial Number".to_string(),
    );
    translations.insert("memory.part_number".to_string(), "Part Number".to_string());
    translations.insert("memory.ecc".to_string(), "ECC".to_string());
    translations.insert("memory.voltage".to_string(), "Voltage".to_string());
    translations.insert("memory.frequency".to_string(), "Frequency".to_string());
    translations.insert("memory.bandwidth".to_string(), "Bandwidth".to_string());
    translations.insert("memory.channels".to_string(), "Channels".to_string());

    // 未知值
    translations.insert("no_discrete_gpu".to_string(), "No Discrete GPU".to_string());
    translations.insert("unknown_system".to_string(), "Unknown System".to_string());
    translations.insert("unknown_model".to_string(), "Unknown Model".to_string());
    translations.insert("unknown_vendor".to_string(), "Unknown Vendor".to_string());
    translations.insert("unknown_version".to_string(), "Unknown Version".to_string());
    translations.insert("unknown_kernel".to_string(), "Unknown Kernel".to_string());
    translations.insert(
        "unknown_architecture".to_string(),
        "Unknown Architecture".to_string(),
    );
    translations.insert("no_driver".to_string(), "No Driver".to_string());
    translations.insert(
        "no_storage_devices".to_string(),
        "No Storage Devices".to_string(),
    );

    // 存储类型
    translations.insert(
        "nvme_ssd_hdd_mixed".to_string(),
        "NVMe+SSD+HDD Mixed".to_string(),
    );
    translations.insert("nvme_ssd_mixed".to_string(), "NVMe+SSD Mixed".to_string());
    translations.insert("nvme_hdd_mixed".to_string(), "NVMe+HDD Mixed".to_string());
    translations.insert("ssd_hdd_mixed".to_string(), "SSD+HDD Mixed".to_string());
    translations.insert("pure_nvme".to_string(), "Pure NVMe".to_string());
    translations.insert("pure_ssd".to_string(), "Pure SSD".to_string());
    translations.insert("pure_hdd".to_string(), "Pure HDD".to_string());
    translations.insert(
        "unknown_storage_type".to_string(),
        "Unknown Storage Type".to_string(),
    );

    // API 成功消息
    translations.insert(
        "clients_filtered_successfully".to_string(),
        "Clients filtered successfully".to_string(),
    );
    translations.insert(
        "filter_options_retrieved_successfully".to_string(),
        "Filter options retrieved successfully".to_string(),
    );
    translations.insert(
        "client_registered_successfully".to_string(),
        "Client registered successfully".to_string(),
    );
    translations.insert(
        "client_updated_successfully".to_string(),
        "Client updated successfully".to_string(),
    );
    translations.insert(
        "client_deleted_successfully".to_string(),
        "Client deleted successfully".to_string(),
    );
    translations.insert(
        "clients_listed_successfully".to_string(),
        "Clients listed successfully".to_string(),
    );
    translations.insert(
        "client_retrieved_successfully".to_string(),
        "Client retrieved successfully".to_string(),
    );
    translations.insert(
        "hardware_retrieved_successfully".to_string(),
        "Hardware retrieved successfully".to_string(),
    );
    translations.insert(
        "stats_retrieved_successfully".to_string(),
        "Statistics retrieved successfully".to_string(),
    );

    // API 错误消息
    translations.insert(
        "empty_client_ids_provided".to_string(),
        "Empty client IDs provided".to_string(),
    );
    translations.insert(
        "no_valid_client_ids_provided".to_string(),
        "No valid client IDs provided".to_string(),
    );
    translations.insert(
        "no_clients_found_with_provided_ids".to_string(),
        "No clients found with provided IDs".to_string(),
    );

    // 错误码翻译
    translations.insert(
        "internal_server_error".to_string(),
        "Internal Server Error".to_string(),
    );
    translations.insert("invalid_request".to_string(), "Invalid Request".to_string());
    translations.insert(
        "validation_error".to_string(),
        "Validation Error".to_string(),
    );
    translations.insert("not_found".to_string(), "Not Found".to_string());
    translations.insert(
        "client_not_found".to_string(),
        "Client Not Found".to_string(),
    );
    translations.insert(
        "client_already_exists".to_string(),
        "Client Already Exists".to_string(),
    );
    translations.insert(
        "client_registration_failed".to_string(),
        "Client Registration Failed".to_string(),
    );
    translations.insert(
        "client_update_failed".to_string(),
        "Client Update Failed".to_string(),
    );
    translations.insert(
        "client_delete_failed".to_string(),
        "Client Delete Failed".to_string(),
    );
    translations.insert(
        "hardware_not_found".to_string(),
        "Hardware Not Found".to_string(),
    );
    translations.insert(
        "hardware_data_invalid".to_string(),
        "Hardware Data Invalid".to_string(),
    );
    translations.insert(
        "hardware_collection_failed".to_string(),
        "Hardware Collection Failed".to_string(),
    );
    translations.insert(
        "filter_options_error".to_string(),
        "Filter Options Error".to_string(),
    );
    translations.insert(
        "filter_query_invalid".to_string(),
        "Filter Query Invalid".to_string(),
    );
    translations.insert(
        "filter_execution_failed".to_string(),
        "Filter Execution Failed".to_string(),
    );
    translations.insert(
        "database_connection_error".to_string(),
        "Database Connection Error".to_string(),
    );
    translations.insert(
        "database_query_error".to_string(),
        "Database Query Error".to_string(),
    );
    translations.insert(
        "database_transaction_error".to_string(),
        "Database Transaction Error".to_string(),
    );
    translations.insert("network_error".to_string(), "Network Error".to_string());
    translations.insert(
        "connection_timeout".to_string(),
        "Connection Timeout".to_string(),
    );
    translations.insert("request_timeout".to_string(), "Request Timeout".to_string());

    // UI 文本
    translations.insert(
        "search_placeholder".to_string(),
        "Search clients...".to_string(),
    );
    translations.insert("filter_by_os".to_string(), "Filter by OS".to_string());
    translations.insert(
        "filter_by_vendor".to_string(),
        "Filter by Vendor".to_string(),
    );
    translations.insert("filter_by_model".to_string(), "Filter by Model".to_string());
    translations.insert("clear_filters".to_string(), "Clear Filters".to_string());
    translations.insert("apply_filters".to_string(), "Apply Filters".to_string());
    translations.insert("total_clients".to_string(), "Total Clients".to_string());
    translations.insert("online_clients".to_string(), "Online Clients".to_string());
    translations.insert("offline_clients".to_string(), "Offline Clients".to_string());
    translations.insert("loading".to_string(), "Loading...".to_string());

    // Menu
    translations.insert("menu.dashboard".to_string(), "Dashboard".to_string());
    translations.insert("menu.assets".to_string(), "Assets".to_string());
    translations.insert("menu.clients".to_string(), "Clients".to_string());
    translations.insert("menu.racks".to_string(), "Racks".to_string());
    translations.insert("racks.list_view".to_string(), "List View".to_string());
    translations.insert("racks.rack_view".to_string(), "Rack View".to_string());
    translations.insert("racks.grid_layout".to_string(), "Grid Layout".to_string());
    translations.insert(
        "racks.single_column_layout".to_string(),
        "Single Column".to_string(),
    );
    translations.insert("racks.capacity_status".to_string(), "Capacity".to_string());
    translations.insert("racks.power_status".to_string(), "Power".to_string());
    translations.insert("racks.remaining".to_string(), "Remaining".to_string());
    translations.insert(
        "racks.used_no_limit".to_string(),
        "Used: {val} W (No Limit)".to_string(),
    );
    translations.insert(
        "racks.confirm_delete".to_string(),
        "Confirm Delete".to_string(),
    );
    translations.insert(
        "racks.confirm_delete_msg".to_string(),
        "Are you sure you want to delete this rack? This action cannot be undone.".to_string(),
    );
    translations.insert("racks.rack_name".to_string(), "Rack Name".to_string());
    translations.insert("racks.location".to_string(), "Location".to_string());
    translations.insert("racks.height_u".to_string(), "Height (U)".to_string());
    translations.insert(
        "racks.power_limit_w".to_string(),
        "Power Limit (W)".to_string(),
    );
    translations.insert("racks.description".to_string(), "Description".to_string());
    translations.insert("racks.cancel".to_string(), "Cancel".to_string());
    translations.insert("racks.save".to_string(), "Save".to_string());
    translations.insert("racks.edit_rack".to_string(), "Edit Rack".to_string());
    translations.insert("racks.add_rack".to_string(), "Add Rack".to_string());
    translations.insert(
        "racks.rack_capacity".to_string(),
        "Rack Capacity".to_string(),
    );
    translations.insert("racks.used".to_string(), "Used".to_string());
    translations.insert("racks.free".to_string(), "Free".to_string());
    translations.insert("racks.power_usage".to_string(), "Power Usage".to_string());
    translations.insert("racks.total_units".to_string(), "Total Units".to_string());
    translations.insert("racks.power_limit".to_string(), "Power Limit".to_string());
    translations.insert("racks.devices".to_string(), "Devices".to_string());
    translations.insert("racks.status".to_string(), "Status".to_string());
    translations.insert("racks.status.active".to_string(), "Active".to_string());
    translations.insert("racks.status.maint".to_string(), "Maintenance".to_string());
    translations.insert("racks.status.stock".to_string(), "In Stock".to_string());
    translations.insert("racks.status.error".to_string(), "Error".to_string());
    translations.insert(
        "racks.delete_success".to_string(),
        "Deleted successfully".to_string(),
    );
    translations.insert(
        "racks.save_success".to_string(),
        "Saved successfully".to_string(),
    );
    translations.insert(
        "racks.save_failed".to_string(),
        "Save failed: {val}".to_string(),
    );
    translations.insert("racks.actions".to_string(), "Actions".to_string());

    translations.insert("menu.components".to_string(), "Components".to_string());
    translations.insert("menu.organization".to_string(), "Organization".to_string());
    translations.insert("menu.users".to_string(), "Users".to_string());
    translations.insert("menu.projects".to_string(), "Projects".to_string());
    translations.insert("menu.system".to_string(), "System".to_string());
    translations.insert("menu.analytics".to_string(), "Analytics".to_string());
    translations.insert("menu.setup_guide".to_string(), "Setup Guide".to_string());
    translations.insert("menu.base_data".to_string(), "Base Data".to_string());
    translations.insert("menu.accounts".to_string(), "Accounts".to_string());
    translations.insert("menu.permissions".to_string(), "Permissions".to_string());
    translations.insert(
        "menu.exec_policies".to_string(),
        "Exec Policies".to_string(),
    );
    translations.insert(
        "menu.web_terminal_policies".to_string(),
        "Web Terminal".to_string(),
    );
    translations.insert("menu.approvals".to_string(), "Approvals".to_string());
    translations.insert("menu.source_code".to_string(), "Source Code".to_string());
    translations.insert("menu.terminal".to_string(), "Terminal".to_string());
    translations.insert("menu.execution".to_string(), "Execution".to_string());
    translations.insert("menu.batch_execution".to_string(), "Batch Exec".to_string());
    translations.insert("menu.execution_history".to_string(), "History".to_string());
    translations.insert(
        "menu.remote_exec_settings".to_string(),
        "Remote Exec Settings".to_string(),
    );
    translations.insert("client_detail.terminal".to_string(), "Terminal".to_string());

    // Terminal / Remote Command Execution
    translations.insert("terminal.title".to_string(), "Remote Terminal".to_string());
    translations.insert("terminal.execute".to_string(), "Execute".to_string());
    translations.insert("terminal.history".to_string(), "History".to_string());
    translations.insert(
        "terminal.history_title".to_string(),
        "Command History".to_string(),
    );
    translations.insert(
        "terminal.remote_exec".to_string(),
        "Remote Exec".to_string(),
    );
    translations.insert("terminal.enabled".to_string(), "Enabled".to_string());
    translations.insert("terminal.disabled".to_string(), "Disabled".to_string());
    translations.insert(
        "terminal.disabled_msg".to_string(),
        "Remote command execution is disabled. Enable it from the toggle above (Admin only)."
            .to_string(),
    );
    translations.insert(
        "terminal.select_client".to_string(),
        "Select a client...".to_string(),
    );
    translations.insert("terminal.client".to_string(), "Target Client".to_string());
    translations.insert("terminal.command".to_string(), "Command".to_string());
    translations.insert(
        "terminal.command_placeholder".to_string(),
        "e.g. df -h".to_string(),
    );
    translations.insert(
        "terminal.timeout_secs".to_string(),
        "Timeout (seconds)".to_string(),
    );
    translations.insert("terminal.run".to_string(), "Run Command".to_string());
    translations.insert(
        "terminal.submitting".to_string(),
        "Submitting...".to_string(),
    );
    translations.insert("terminal.col_client".to_string(), "Client".to_string());
    translations.insert("terminal.col_command".to_string(), "Command".to_string());
    translations.insert("terminal.col_status".to_string(), "Status".to_string());
    translations.insert("terminal.col_danger".to_string(), "Danger".to_string());
    translations.insert(
        "terminal.col_submitted_by".to_string(),
        "Submitted By".to_string(),
    );
    translations.insert(
        "terminal.col_created_at".to_string(),
        "Created At".to_string(),
    );

    // Header
    translations.insert(
        "header.search_placeholder".to_string(),
        "Search hostname, IP...".to_string(),
    );
    translations.insert(
        "header.change_password".to_string(),
        "Change Password".to_string(),
    );
    translations.insert("header.logout".to_string(), "Logout".to_string());
    translations.insert(
        "header.switch_language".to_string(),
        "Switch Language".to_string(),
    );

    // Auth
    translations.insert("auth.login_title".to_string(), "Login to CMDB".to_string());
    translations.insert("auth.username".to_string(), "Username".to_string());
    translations.insert("auth.password".to_string(), "Password".to_string());
    translations.insert("auth.login_button".to_string(), "Sign In".to_string());
    translations.insert("auth.logging_in".to_string(), "Signing in...".to_string());

    // Change Password
    translations.insert(
        "password.change_title".to_string(),
        "Change Password".to_string(),
    );
    translations.insert(
        "password.current".to_string(),
        "Current Password".to_string(),
    );
    translations.insert("password.new".to_string(), "New Password".to_string());
    translations.insert(
        "password.confirm".to_string(),
        "Confirm New Password".to_string(),
    );
    translations.insert("password.submit".to_string(), "Change Password".to_string());
    translations.insert(
        "password.submitting".to_string(),
        "Submitting...".to_string(),
    );
    translations.insert(
        "password.success".to_string(),
        "Password changed successfully. Redirecting to login...".to_string(),
    );
    translations.insert(
        "password.mismatch".to_string(),
        "New passwords do not match".to_string(),
    );
    translations.insert(
        "password.too_short".to_string(),
        "Password must be at least 6 characters".to_string(),
    );

    // Dashboard
    translations.insert(
        "dashboard.loading".to_string(),
        "Loading dashboard data...".to_string(),
    );
    translations.insert(
        "dashboard.total_clients".to_string(),
        "Total Clients".to_string(),
    );
    translations.insert(
        "dashboard.registered_nodes".to_string(),
        "Registered Nodes".to_string(),
    );
    translations.insert(
        "dashboard.online_rate".to_string(),
        "Online Rate".to_string(),
    );
    translations.insert("dashboard.online".to_string(), "Online".to_string());
    translations.insert("dashboard.new_today".to_string(), "New Today".to_string());
    translations.insert(
        "dashboard.24h_registered".to_string(),
        "Registered in 24h".to_string(),
    );
    translations.insert(
        "dashboard.system_types".to_string(),
        "System Types".to_string(),
    );
    translations.insert("dashboard.diverse_os".to_string(), "Diverse OS".to_string());

    // Dashboard Sub-components
    translations.insert(
        "dashboard.os_dist_title".to_string(),
        "OS Distribution".to_string(),
    );
    translations.insert(
        "dashboard.os_dist_desc".to_string(),
        "By Registered Clients".to_string(),
    );
    translations.insert("dashboard.realtime".to_string(), "Realtime".to_string());
    translations.insert("dashboard.no_data".to_string(), "No Data".to_string());
    translations.insert(
        "analytics.unit_machine".to_string(),
        "{count} units".to_string(),
    );
    translations.insert(
        "analytics.gpu_model_distribution".to_string(),
        "GPU Model Distribution".to_string(),
    );
    translations.insert(
        "dashboard.realtime_refresh".to_string(),
        "Realtime Refresh".to_string(),
    );
    translations.insert(
        "dashboard.online_clients".to_string(),
        "Online Clients".to_string(),
    );
    translations.insert(
        "dashboard.offline_clients".to_string(),
        "Offline Clients".to_string(),
    );
    translations.insert(
        "dashboard.realtime_update".to_string(),
        " Realtime Update".to_string(),
    );

    translations.insert(
        "dashboard.recent_active_title".to_string(),
        "Recently Active".to_string(),
    );
    translations.insert(
        "dashboard.recent_active_desc".to_string(),
        "Last 10 Heartbeats".to_string(),
    );
    translations.insert(
        "dashboard.recent_offline_title".to_string(),
        "Recently Offline".to_string(),
    );
    translations.insert(
        "dashboard.recent_offline_desc".to_string(),
        "Most recently seen offline clients".to_string(),
    );
    translations.insert(
        "dashboard.no_clients_registered".to_string(),
        "No Clients Registered".to_string(),
    );
    translations.insert(
        "dashboard.no_recent_offline_clients".to_string(),
        "No offline clients recently".to_string(),
    );
    translations.insert("dashboard.offline".to_string(), "Offline".to_string());

    translations.insert(
        "dashboard.client_status_list".to_string(),
        "Client Status List".to_string(),
    );
    translations.insert("dashboard.total".to_string(), "Total".to_string());
    translations.insert(
        "dashboard.managed_nodes".to_string(),
        "Managed Nodes".to_string(),
    );
    translations.insert("dashboard.view_all".to_string(), "View All".to_string());
    translations.insert("dashboard.host".to_string(), "Host".to_string());
    translations.insert("dashboard.system".to_string(), "System".to_string());
    translations.insert("dashboard.config".to_string(), "Config".to_string());
    translations.insert("dashboard.status".to_string(), "Status".to_string());

    // Clients Page
    translations.insert(
        "clients.stats.total_devices".to_string(),
        "Total Devices".to_string(),
    );
    translations.insert(
        "clients.stats.filtered_results".to_string(),
        "Filtered Results".to_string(),
    );
    translations.insert("clients.stats.os_types".to_string(), "OS Types".to_string());
    translations.insert(
        "clients.stats.vendor_count".to_string(),
        "Vendor Count".to_string(),
    );

    translations.insert(
        "clients.search.title".to_string(),
        "Advanced Search & Filter".to_string(),
    );
    translations.insert(
        "clients.search.keyword_label".to_string(),
        "Keyword Search".to_string(),
    );
    translations.insert(
        "clients.search.placeholder".to_string(),
        "Search hostname, IP, OS, vendor, model or serial...".to_string(),
    );
    translations.insert(
        "clients.search.hint".to_string(),
        "Supports fuzzy search, updates automatically after typing stops".to_string(),
    );
    translations.insert(
        "clients.search.export_csv".to_string(),
        "Export CSV".to_string(),
    );
    translations.insert(
        "clients.search.export_json".to_string(),
        "Export JSON".to_string(),
    );
    translations.insert(
        "clients.search.import".to_string(),
        "Import Data".to_string(),
    );
    translations.insert(
        "clients.search.apply".to_string(),
        "Apply Filters".to_string(),
    );
    translations.insert(
        "clients.search.clear".to_string(),
        "Clear Filters".to_string(),
    );
    translations.insert(
        "clients.filter.active_filters".to_string(),
        "Active Filters".to_string(),
    );

    translations.insert("clients.filter.status".to_string(), "Status".to_string());
    translations.insert(
        "clients.filter.environment".to_string(),
        "Environment".to_string(),
    );
    translations.insert("clients.filter.rack".to_string(), "Rack".to_string());
    translations.insert("clients.filter.project".to_string(), "Project".to_string());
    translations.insert("clients.filter.owner".to_string(), "Owner".to_string());
    translations.insert("clients.filter.os".to_string(), "OS".to_string());
    translations.insert("clients.filter.kernel".to_string(), "Kernel".to_string());
    translations.insert(
        "clients.filter.vendor".to_string(),
        "Server Vendor".to_string(),
    );
    translations.insert(
        "clients.filter.cpu_vendor".to_string(),
        "CPU Vendor".to_string(),
    );
    translations.insert(
        "clients.filter.cpu_model".to_string(),
        "CPU Model".to_string(),
    );
    translations.insert(
        "clients.filter.gpu_vendor".to_string(),
        "GPU Vendor".to_string(),
    );
    translations.insert(
        "clients.filter.gpu_model".to_string(),
        "GPU Model".to_string(),
    );
    translations.insert(
        "clients.filter.memory_min".to_string(),
        "Min Memory (GB)".to_string(),
    );
    translations.insert(
        "clients.filter.memory_max".to_string(),
        "Max Memory (GB)".to_string(),
    );
    translations.insert(
        "clients.filter.network_type".to_string(),
        "Network Type".to_string(),
    );
    translations.insert(
        "clients.filter.network_model".to_string(),
        "Network Model".to_string(),
    );
    translations.insert(
        "clients.filter.storage_type".to_string(),
        "Storage Type".to_string(),
    );

    translations.insert(
        "clients.table.no_data".to_string(),
        "No device data available".to_string(),
    );
    translations.insert("clients.table.hostname".to_string(), "Hostname".to_string());
    translations.insert(
        "clients.table.online_status".to_string(),
        "Online".to_string(),
    );
    translations.insert("clients.table.ip".to_string(), "IP Address".to_string());
    translations.insert("clients.table.os".to_string(), "OS".to_string());
    translations.insert("clients.table.owner".to_string(), "Owner".to_string());
    translations.insert("clients.table.project".to_string(), "Project".to_string());
    translations.insert("clients.table.status".to_string(), "Status".to_string());
    translations.insert(
        "clients.table.environment".to_string(),
        "Environment".to_string(),
    );
    translations.insert("clients.table.actions".to_string(), "Actions".to_string());

    translations.insert("clients.status.active".to_string(), "Active".to_string());
    translations.insert(
        "clients.status.maintenance".to_string(),
        "Maintenance".to_string(),
    );
    translations.insert("clients.status.instock".to_string(), "In Stock".to_string());
    translations.insert(
        "clients.status.decommissioned".to_string(),
        "Decommissioned".to_string(),
    );

    translations.insert("clients.env.prod".to_string(), "Prod".to_string());
    translations.insert("clients.env.staging".to_string(), "Staging".to_string());
    translations.insert("clients.env.test".to_string(), "Test".to_string());
    translations.insert("clients.env.dev".to_string(), "Dev".to_string());

    translations.insert(
        "clients.actions.view".to_string(),
        "View Details".to_string(),
    );
    translations.insert("clients.actions.edit".to_string(), "Edit".to_string());
    translations.insert("clients.actions.delete".to_string(), "Delete".to_string());
    translations.insert(
        "clients.actions.confirm_delete".to_string(),
        "Are you sure you want to delete this device? NOTE: If the client agent is still running, it will re-register automatically. Stop the client service first.".to_string(),
    );

    translations.insert(
        "clients.import.success".to_string(),
        "Import Successful!".to_string(),
    );
    translations.insert(
        "clients.import.error_title".to_string(),
        "Import Failed".to_string(),
    );
    translations.insert(
        "clients.import.error_desc".to_string(),
        "Errors found, import cancelled. Please fix and retry.".to_string(),
    );
    translations.insert(
        "clients.import.progress".to_string(),
        "Importing...".to_string(),
    );
    translations.insert(
        "clients.selection.selected".to_string(),
        "Selected".to_string(),
    );
    translations.insert(
        "clients.selection.export_template".to_string(),
        "Export Template".to_string(),
    );

    translations.insert("common.close".to_string(), "Close".to_string());
    translations.insert("common.error_prefix".to_string(), "Error: ".to_string());

    translations.insert("error".to_string(), "Error".to_string());
    translations.insert("success".to_string(), "Success".to_string());
    translations.insert("warning".to_string(), "Warning".to_string());
    translations.insert("info".to_string(), "Info".to_string());

    // Notification
    translations.insert("notification.success".to_string(), "Success".to_string());
    translations.insert("notification.info".to_string(), "Information".to_string());
    translations.insert("notification.warning".to_string(), "Warning".to_string());
    translations.insert("notification.error".to_string(), "Error".to_string());

    // 表格列标题
    translations.insert("hostname".to_string(), "Hostname".to_string());
    translations.insert("ip_address".to_string(), "IP Address".to_string());
    translations.insert("os".to_string(), "Operating System".to_string());
    translations.insert("vendor".to_string(), "Vendor".to_string());
    translations.insert("model".to_string(), "Model".to_string());
    translations.insert("last_seen".to_string(), "Last Seen".to_string());
    translations.insert("status".to_string(), "Status".to_string());
    translations.insert("actions".to_string(), "Actions".to_string());

    // 分页
    translations.insert("previous_page".to_string(), "Previous".to_string());
    translations.insert("next_page".to_string(), "Next".to_string());
    translations.insert("page".to_string(), "Page".to_string());
    translations.insert("of".to_string(), "of".to_string());
    translations.insert("items_per_page".to_string(), "items per page".to_string());

    // Common
    translations.insert("common.cancel".to_string(), "Cancel".to_string());
    translations.insert("common.save".to_string(), "Save".to_string());
    translations.insert(
        "common.delete_success".to_string(),
        "Delete Successful".to_string(),
    );
    translations.insert(
        "common.save_success".to_string(),
        "Save Successful".to_string(),
    );
    translations.insert(
        "common.save_failed".to_string(),
        "Save Failed: {}".to_string(),
    );
    translations.insert("common.actions".to_string(), "Actions".to_string());
    translations.insert(
        "common.confirm_delete".to_string(),
        "Confirm Delete".to_string(),
    );
    translations.insert("common.delete".to_string(), "Delete".to_string());
    translations.insert("common.export".to_string(), "Export".to_string());
    translations.insert("common.reset".to_string(), "Reset".to_string());

    // Projects
    translations.insert(
        "projects.select_cost_center".to_string(),
        "Select Cost Center".to_string(),
    );
    translations.insert(
        "projects.select_manager".to_string(),
        "Select Manager".to_string(),
    );
    translations.insert(
        "projects.edit_project".to_string(),
        "Edit Project".to_string(),
    );
    translations.insert(
        "projects.add_project".to_string(),
        "Add Project".to_string(),
    );
    translations.insert("projects.name".to_string(), "Project Name".to_string());
    translations.insert("projects.code".to_string(), "Project Code".to_string());
    translations.insert("projects.department".to_string(), "Department".to_string());
    translations.insert(
        "projects.cost_center".to_string(),
        "Cost Center".to_string(),
    );
    translations.insert("projects.manager".to_string(), "Manager".to_string());
    translations.insert(
        "projects.new_project".to_string(),
        "New Project".to_string(),
    );
    translations.insert(
        "projects.confirm_delete_msg".to_string(),
        "Are you sure you want to delete this project? This action cannot be undone.".to_string(),
    );

    // Persons
    translations.insert(
        "persons.select_department".to_string(),
        "Select Department".to_string(),
    );
    translations.insert(
        "persons.select_title".to_string(),
        "Select Title".to_string(),
    );
    translations.insert("persons.edit_person".to_string(), "Edit Person".to_string());
    translations.insert("persons.add_person".to_string(), "Add Person".to_string());
    translations.insert("persons.name".to_string(), "Name".to_string());
    translations.insert("persons.email".to_string(), "Email".to_string());
    translations.insert("persons.department".to_string(), "Department".to_string());
    translations.insert("persons.phone".to_string(), "Phone".to_string());
    translations.insert("persons.title".to_string(), "Title".to_string());
    translations.insert(
        "persons.delete_success".to_string(),
        "Delete Successful".to_string(),
    );
    translations.insert(
        "persons.save_success".to_string(),
        "Save Successful".to_string(),
    );
    translations.insert(
        "persons.save_failed".to_string(),
        "Save Failed: {}".to_string(),
    );
    translations.insert("persons.new_person".to_string(), "New Person".to_string());
    translations.insert("persons.actions".to_string(), "Actions".to_string());
    translations.insert(
        "persons.confirm_delete".to_string(),
        "Confirm Delete".to_string(),
    );
    translations.insert(
        "persons.confirm_delete_msg".to_string(),
        "Are you sure you want to delete this person? This action cannot be undone.".to_string(),
    );

    // Components
    translations.insert("components.type_other".to_string(), "Other".to_string());
    translations.insert("components.type_gpu".to_string(), "GPU".to_string());
    translations.insert("components.type_cpu".to_string(), "CPU".to_string());
    translations.insert("components.type_memory".to_string(), "Memory".to_string());
    translations.insert("components.type_disk".to_string(), "Disk".to_string());
    translations.insert(
        "components.type_network_card".to_string(),
        "Network Card".to_string(),
    );
    translations.insert(
        "components.type_motherboard".to_string(),
        "Motherboard".to_string(),
    );
    translations.insert(
        "components.type_power_supply".to_string(),
        "Power Supply".to_string(),
    );
    translations.insert(
        "components.status_in_stock".to_string(),
        "In Stock".to_string(),
    );
    translations.insert("components.status_in_use".to_string(), "In Use".to_string());
    translations.insert(
        "components.status_lent_out".to_string(),
        "Lent Out".to_string(),
    );
    translations.insert("components.status_faulty".to_string(), "Faulty".to_string());
    translations.insert(
        "components.status_decommissioned".to_string(),
        "Decommissioned".to_string(),
    );
    translations.insert(
        "components.status_unknown".to_string(),
        "Unknown".to_string(),
    );
    translations.insert(
        "components.new_component".to_string(),
        "New Component".to_string(),
    );
    translations.insert(
        "components.edit_component".to_string(),
        "Edit Component: {model} ({sn})".to_string(),
    );
    translations.insert(
        "components.serial_number".to_string(),
        "Serial Number (SN)".to_string(),
    );
    translations.insert("components.model".to_string(), "Model".to_string());
    translations.insert("components.type".to_string(), "Type".to_string());
    translations.insert("components.vendor".to_string(), "Vendor".to_string());
    translations.insert("components.status".to_string(), "Status".to_string());
    translations.insert("components.location".to_string(), "Location".to_string());
    translations.insert(
        "components.purchase_date".to_string(),
        "Purchase Date".to_string(),
    );
    translations.insert(
        "components.warranty_expiration".to_string(),
        "Warranty Expiration".to_string(),
    );
    translations.insert(
        "components.batch_create_json".to_string(),
        "Batch Create (JSON)".to_string(),
    );
    translations.insert(
        "components.json_parse_error".to_string(),
        "JSON Parse Error: {error}".to_string(),
    );
    translations.insert(
        "components.json_input_hint".to_string(),
        "Please enter a JSON array containing component information. Example:".to_string(),
    );
    translations.insert(
        "components.batch_create".to_string(),
        "Batch Create".to_string(),
    );
    translations.insert(
        "components.select_components_first".to_string(),
        "Please select components first".to_string(),
    );
    translations.insert(
        "components.confirm_batch_status_update".to_string(),
        "Are you sure you want to update the status of {count} selected components to {status}?"
            .to_string(),
    );
    translations.insert(
        "components.batch_status_update_success".to_string(),
        "Batch status update successful".to_string(),
    );
    translations.insert(
        "components.batch_status_update_failed".to_string(),
        "Batch status update failed: {error}".to_string(),
    );
    translations.insert(
        "components.batch_edit_export".to_string(),
        "Batch Edit (Export)".to_string(),
    );
    translations.insert(
        "components.json_import".to_string(),
        "JSON Import".to_string(),
    );
    translations.insert(
        "components.excel_import".to_string(),
        "Excel Import".to_string(),
    );
    translations.insert(
        "components.quick_status_change".to_string(),
        "Quick Status Change...".to_string(),
    );
    translations.insert("components.search".to_string(), "Search".to_string());
    translations.insert("components.search_label".to_string(), "Search".to_string());
    translations.insert(
        "components.search_placeholder".to_string(),
        "Search SN or Model...".to_string(),
    );
    translations.insert(
        "components.component_info".to_string(),
        "Component Info".to_string(),
    );
    translations.insert(
        "components.type_vendor".to_string(),
        "Type/Vendor".to_string(),
    );
    translations.insert(
        "components.location_owner".to_string(),
        "Location/Owner".to_string(),
    );
    translations.insert("components.actions".to_string(), "Actions".to_string());
    translations.insert(
        "components.server_prefix".to_string(),
        "Server: ".to_string(),
    );
    translations.insert(
        "components.importing".to_string(),
        "Importing...".to_string(),
    );
    translations.insert(
        "components.batch_update_complete".to_string(),
        "Batch update complete: {success}/{total} successful".to_string(),
    );
    translations.insert(
        "components.batch_create_success".to_string(),
        "Successfully created {count} components".to_string(),
    );
    translations.insert(
        "components.batch_create_failed".to_string(),
        "Batch create failed: {error}".to_string(),
    );

    // Analytics
    translations.insert(
        "analytics.total_devices".to_string(),
        "Total Devices".to_string(),
    );
    translations.insert(
        "analytics.online_devices".to_string(),
        "Online Devices".to_string(),
    );
    translations.insert(
        "analytics.offline_devices".to_string(),
        "Offline Devices".to_string(),
    );
    translations.insert(
        "analytics.online_rate".to_string(),
        "Online Rate".to_string(),
    );
    translations.insert("analytics.retry".to_string(), "Retry".to_string());
    translations.insert(
        "analytics.gpu_vendor_distribution".to_string(),
        "GPU Vendor Distribution".to_string(),
    );
    translations.insert(
        "analytics.by_machine_count".to_string(),
        "By Machine Count".to_string(),
    );
    translations.insert(
        "analytics.detailed_stats".to_string(),
        "Detailed Stats".to_string(),
    );
    translations.insert("analytics.gpu_vendor".to_string(), "GPU Vendor".to_string());
    translations.insert("analytics.count".to_string(), "Count".to_string());
    translations.insert("analytics.percentage".to_string(), "Percentage".to_string());
    translations.insert("analytics.unit_machine".to_string(), "{} units".to_string());
    translations.insert(
        "analytics.gpu_model_distribution".to_string(),
        "GPU Model Distribution".to_string(),
    );
    translations.insert("analytics.gpu_model".to_string(), "GPU Model".to_string());
    translations.insert(
        "analytics.gpu_detailed_config".to_string(),
        "GPU Detailed Config".to_string(),
    );
    translations.insert(
        "analytics.by_model_and_count".to_string(),
        "By Model and Count".to_string(),
    );
    translations.insert("analytics.gpu_config".to_string(), "GPU Config".to_string());
    translations.insert(
        "analytics.cpu_model_distribution".to_string(),
        "CPU Model Distribution".to_string(),
    );
    translations.insert("analytics.cpu_model".to_string(), "CPU Model".to_string());
    translations.insert(
        "analytics.storage_type_distribution".to_string(),
        "Storage Type Distribution".to_string(),
    );
    translations.insert(
        "analytics.storage_type".to_string(),
        "Storage Type".to_string(),
    );
    translations.insert(
        "analytics.os_distribution".to_string(),
        "OS Distribution".to_string(),
    );
    translations.insert("analytics.os".to_string(), "OS".to_string());
    translations.insert(
        "analytics.memory_size_distribution".to_string(),
        "Memory Size Distribution".to_string(),
    );
    translations.insert(
        "analytics.memory_size".to_string(),
        "Memory Size".to_string(),
    );
    translations.insert(
        "analytics.network_type_distribution".to_string(),
        "Network Type Distribution".to_string(),
    );
    translations.insert(
        "analytics.network_type".to_string(),
        "Network Type".to_string(),
    );
    translations.insert(
        "analytics.server_model_distribution".to_string(),
        "Server Model Distribution".to_string(),
    );
    translations.insert(
        "analytics.server_model".to_string(),
        "Server Model".to_string(),
    );

    // Stats Filter
    translations.insert(
        "stats.filter.cpu_vendor".to_string(),
        "CPU Vendor".to_string(),
    );
    translations.insert(
        "stats.filter.gpu_vendor".to_string(),
        "GPU Vendor".to_string(),
    );
    translations.insert(
        "stats.filter.memory_capacity".to_string(),
        "Memory Capacity (GB)".to_string(),
    );
    translations.insert(
        "stats.filter.os".to_string(),
        "Operating System".to_string(),
    );

    // Client Setup
    translations.insert(
        "client_setup.parse_error".to_string(),
        "Failed to parse response: {error}".to_string(),
    );
    translations.insert(
        "client_setup.request_failed".to_string(),
        "Request failed: {status}".to_string(),
    );
    translations.insert(
        "client_setup.network_error".to_string(),
        "Network request failed: {error}".to_string(),
    );
    translations.insert(
        "client_setup.load_failed".to_string(),
        "Failed to load client info".to_string(),
    );
    translations.insert(
        "client_setup.guide_title".to_string(),
        "Client Installation Guide".to_string(),
    );
    translations.insert(
        "client_setup.guide_subtitle".to_string(),
        "Select platform and architecture for installation instructions".to_string(),
    );
    translations.insert(
        "client_setup.select_platform".to_string(),
        "Select Platform".to_string(),
    );
    translations.insert(
        "client_setup.select_arch".to_string(),
        "Select Architecture".to_string(),
    );
    translations.insert(
        "client_setup.step1_download".to_string(),
        "1. Download Client".to_string(),
    );
    translations.insert(
        "client_setup.download_url".to_string(),
        "Download URL: ".to_string(),
    );
    translations.insert(
        "client_setup.server_url".to_string(),
        "Server URL: ".to_string(),
    );
    translations.insert(
        "client_setup.step2_quick_install".to_string(),
        "2. Quick Install (Recommended)".to_string(),
    );
    translations.insert(
        "client_setup.copy_command".to_string(),
        "Copy and run the following command in terminal:".to_string(),
    );
    translations.insert(
        "client_setup.quick_install_desc".to_string(),
        "This command will automatically download, install and start the client service"
            .to_string(),
    );
    translations.insert(
        "client_setup.agent_upgrade_title".to_string(),
        "Agent Upgrade".to_string(),
    );
    translations.insert(
        "client_setup.agent_upgrade_desc".to_string(),
        "The upgrade script replaces only the client binary and does not overwrite the existing configuration or credentials.".to_string(),
    );
    translations.insert(
        "client_setup.agent_upgrade_preserve".to_string(),
        "The script preserves client.toml, the client ID and the agent token. A running service is restarted after the upgrade, and the previous binary is restored if startup fails.".to_string(),
    );
    translations.insert(
        "client_setup.agent_upgrade_command_label".to_string(),
        "Quick upgrade command (run it on the target Linux host):".to_string(),
    );
    translations.insert(
        "client_setup.agent_upgrade_command_desc".to_string(),
        "You can also save the script below as upgrade.sh and run it. Verify the server URL and download source before execution.".to_string(),
    );
    translations.insert(
        "client_setup.ansible_title".to_string(),
        "Ansible Install/Upgrade Example".to_string(),
    );
    translations.insert(
        "client_setup.ansible_desc".to_string(),
        "For Linux hosts. The first run creates a default configuration; later runs preserve the existing configuration. For production, set rs_cmdb_agent_checksum to a trusted SHA-256 value.".to_string(),
    );
    translations.insert(
        "client_setup.step3_manual_install".to_string(),
        "3. Manual Install".to_string(),
    );
    translations.insert(
        "client_setup.step2_install_script".to_string(),
        "2. Install Script".to_string(),
    );
    translations.insert(
        "client_setup.save_script".to_string(),
        "Save the following script to a file and run it:".to_string(),
    );
    translations.insert(
        "client_setup.step4_config".to_string(),
        "4. Configuration".to_string(),
    );
    translations.insert(
        "client_setup.step3_config".to_string(),
        "3. Configuration".to_string(),
    );
    translations.insert(
        "client_setup.config_template_desc".to_string(),
        "Config template (Default path: /etc/rs-cmdb/client.toml):".to_string(),
    );
    translations.insert(
        "client_setup.step5_systemd".to_string(),
        "5. Systemd Service".to_string(),
    );
    translations.insert(
        "client_setup.systemd_desc".to_string(),
        "Systemd service file (/etc/systemd/system/rs-cmdb-client.service):".to_string(),
    );
    translations.insert(
        "client_setup.step6_verify".to_string(),
        "6. Verify Installation".to_string(),
    );
    translations.insert(
        "client_setup.step4_verify".to_string(),
        "4. Verify Installation".to_string(),
    );
    translations.insert(
        "client_setup.check_status".to_string(),
        "Check Service Status".to_string(),
    );
    translations.insert(
        "client_setup.manual_run_check".to_string(),
        "Run client manually and check connection to server".to_string(),
    );
    translations.insert(
        "client_setup.check_logs".to_string(),
        "Check Logs".to_string(),
    );
    translations.insert(
        "client_setup.check_logs_dir".to_string(),
        "Check application log directory".to_string(),
    );
    translations.insert(
        "client_setup.install_complete_prefix".to_string(),
        "After installation, the client will automatically appear in ".to_string(),
    );
    translations.insert(
        "client_setup.client_list".to_string(),
        "Client List".to_string(),
    );
    translations.insert(
        "client_setup.install_complete_suffix".to_string(),
        ".".to_string(),
    );

    // Dictionaries
    translations.insert(
        "dictionaries.department".to_string(),
        "Department".to_string(),
    );
    translations.insert("dictionaries.title".to_string(), "Title".to_string());
    translations.insert(
        "dictionaries.cost_center".to_string(),
        "Cost Center".to_string(),
    );
    translations.insert(
        "dictionaries.dictionary_item".to_string(),
        "Dictionary Item".to_string(),
    );
    translations.insert(
        "dictionaries.create_prefix".to_string(),
        "Create ".to_string(),
    );
    translations.insert("dictionaries.edit_prefix".to_string(), "Edit ".to_string());
    translations.insert("dictionaries.key_label".to_string(), "Key:".to_string());
    translations.insert(
        "dictionaries.key_desc".to_string(),
        " Unique identifier used internally, usually in English or code (e.g., 'HR', 'DEV_01')."
            .to_string(),
    );
    translations.insert("dictionaries.value_label".to_string(), "Value:".to_string());
    translations.insert(
        "dictionaries.value_desc".to_string(),
        " Name displayed to users (e.g., 'Human Resources', 'Dev Team 1').".to_string(),
    );
    translations.insert("dictionaries.key".to_string(), "Key".to_string());
    translations.insert(
        "dictionaries.key_placeholder".to_string(),
        "e.g., HR".to_string(),
    );
    translations.insert("dictionaries.value".to_string(), "Value".to_string());
    translations.insert(
        "dictionaries.value_placeholder".to_string(),
        "e.g., Human Resources".to_string(),
    );
    translations.insert(
        "dictionaries.description".to_string(),
        "Description".to_string(),
    );
    translations.insert(
        "dictionaries.description_placeholder".to_string(),
        "Optional description".to_string(),
    );
    translations.insert("dictionaries.cancel".to_string(), "Cancel".to_string());
    translations.insert("dictionaries.save".to_string(), "Save".to_string());
    translations.insert(
        "dictionaries.delete_success".to_string(),
        "Deleted successfully".to_string(),
    );
    translations.insert(
        "dictionaries.save_success".to_string(),
        "Saved successfully".to_string(),
    );
    translations.insert(
        "dictionaries.save_failed".to_string(),
        "Save failed: {error}".to_string(),
    );
    translations.insert(
        "dictionaries.create_department".to_string(),
        "Create Department".to_string(),
    );
    translations.insert(
        "dictionaries.create_title".to_string(),
        "Create Title".to_string(),
    );
    translations.insert(
        "dictionaries.create_cost_center".to_string(),
        "Create Cost Center".to_string(),
    );
    translations.insert("dictionaries.create".to_string(), "Create".to_string());
    translations.insert(
        "dictionaries.create_item".to_string(),
        "Create Item".to_string(),
    );
    translations.insert("dictionaries.actions".to_string(), "Actions".to_string());
    translations.insert(
        "dictionaries.confirm_delete_title".to_string(),
        "Confirm Delete".to_string(),
    );
    translations.insert(
        "dictionaries.confirm_delete_message".to_string(),
        "Are you sure you want to delete this item? This action cannot be undone.".to_string(),
    );

    // Users
    translations.insert(
        "users.update_success".to_string(),
        "User updated successfully".to_string(),
    );
    translations.insert(
        "users.create_success".to_string(),
        "User created successfully".to_string(),
    );
    translations.insert(
        "users.delete_confirm".to_string(),
        "Are you sure you want to delete this user?".to_string(),
    );
    translations.insert(
        "users.delete_success".to_string(),
        "User deleted successfully".to_string(),
    );
    translations.insert("users.create_user".to_string(), "Create User".to_string());
    translations.insert("users.username".to_string(), "Username".to_string());
    translations.insert("users.role".to_string(), "Role".to_string());
    translations.insert("users.status".to_string(), "Status".to_string());
    translations.insert("users.last_login".to_string(), "Last Login".to_string());
    translations.insert("users.actions".to_string(), "Actions".to_string());
    translations.insert("users.active".to_string(), "Active".to_string());
    translations.insert("users.inactive".to_string(), "Inactive".to_string());
    translations.insert("users.edit_user".to_string(), "Edit User".to_string());
    translations.insert(
        "users.password_placeholder_edit".to_string(),
        "Password (leave blank to keep unchanged)".to_string(),
    );
    translations.insert("users.password".to_string(), "Password".to_string());
    translations.insert(
        "users.username_placeholder".to_string(),
        "Enter username".to_string(),
    );
    translations.insert(
        "users.password_placeholder".to_string(),
        "Enter password".to_string(),
    );
    translations.insert(
        "users.enable_account".to_string(),
        "Enable Account".to_string(),
    );
    translations.insert("users.cancel".to_string(), "Cancel".to_string());
    translations.insert("users.save".to_string(), "Save".to_string());
    translations.insert("users.role_viewer".to_string(), "Viewer".to_string());
    translations.insert("users.role_user".to_string(), "User".to_string());
    translations.insert("users.role_admin".to_string(), "Admin".to_string());

    translations.insert(
        "analytics.unit_machine".to_string(),
        "{count} units".to_string(),
    );

    // Dashboard
    translations.insert("dashboard.unit_machines".to_string(), " units".to_string());
    translations.insert(
        "dashboard.system_status_title".to_string(),
        "System Status Overview".to_string(),
    );

    // Pagination
    translations.insert(
        "pagination.total_items".to_string(),
        "Total {count} items".to_string(),
    );
    translations.insert(
        "pagination.items_per_page".to_string(),
        "items/page".to_string(),
    );
    translations.insert("pagination.per_page".to_string(), "Per page".to_string());
    translations.insert("pagination.unit".to_string(), "items".to_string());
    translations.insert("pagination.jump_to".to_string(), "Go to".to_string());
    translations.insert("pagination.go".to_string(), "GO".to_string());

    // Client Detail
    translations.insert(
        "client_detail.title".to_string(),
        "Client Details".to_string(),
    );
    translations.insert(
        "client_detail.basic_info".to_string(),
        "Basic Information".to_string(),
    );
    translations.insert("client_detail.edit".to_string(), "Edit".to_string());
    translations.insert("client_detail.refresh".to_string(), "Refresh".to_string());
    translations.insert("client_detail.id".to_string(), "ID".to_string());
    translations.insert("client_detail.hostname".to_string(), "Hostname".to_string());
    translations.insert("client_detail.ip".to_string(), "IP Address".to_string());
    translations.insert(
        "client_detail.primary_ip".to_string(),
        "Primary IP".to_string(),
    );
    translations.insert("client_detail.os".to_string(), "OS".to_string());
    translations.insert("client_detail.kernel".to_string(), "Kernel".to_string());
    translations.insert("client_detail.location".to_string(), "Location".to_string());
    translations.insert("client_detail.rack".to_string(), "Rack".to_string());
    translations.insert(
        "client_detail.unit_position".to_string(),
        "Unit Position".to_string(),
    );
    translations.insert(
        "client_detail.u_height".to_string(),
        "Height (U)".to_string(),
    );
    translations.insert("client_detail.power".to_string(), "Power (W)".to_string());
    translations.insert("client_detail.owner".to_string(), "Owner".to_string());
    translations.insert("client_detail.project".to_string(), "Project".to_string());
    translations.insert(
        "client_detail.serial".to_string(),
        "Serial Number".to_string(),
    );
    translations.insert(
        "client_detail.asset_tag".to_string(),
        "Asset Tag".to_string(),
    );
    translations.insert(
        "client_detail.warranty".to_string(),
        "Warranty Expiration".to_string(),
    );
    translations.insert("client_detail.supplier".to_string(), "Supplier".to_string());
    translations.insert(
        "client_detail.registered".to_string(),
        "Registered At".to_string(),
    );
    translations.insert(
        "client_detail.last_seen".to_string(),
        "Last Seen".to_string(),
    );
    translations.insert("client_detail.status".to_string(), "Status".to_string());
    translations.insert(
        "client_detail.environment".to_string(),
        "Environment".to_string(),
    );
    translations.insert("client_detail.comment".to_string(), "Comment".to_string());
    translations.insert(
        "client_detail.tab_overview".to_string(),
        "Overview".to_string(),
    );
    translations.insert(
        "client_detail.tab_hardware".to_string(),
        "Hardware".to_string(),
    );
    translations.insert(
        "client_detail.tab_history".to_string(),
        "History".to_string(),
    );
    translations.insert(
        "client_detail.loading_overview".to_string(),
        "Loading overview...".to_string(),
    );
    translations.insert(
        "client_detail.loading_hardware".to_string(),
        "Loading hardware...".to_string(),
    );
    translations.insert(
        "client_detail.loading_client".to_string(),
        "Loading client info...".to_string(),
    );
    translations.insert(
        "client_detail.no_hardware".to_string(),
        "No hardware info".to_string(),
    );
    translations.insert(
        "client_detail.update_success".to_string(),
        "Update successful".to_string(),
    );
    translations.insert(
        "client_detail.update_failed".to_string(),
        "Update failed: {}".to_string(),
    );

    // Client Status
    translations.insert("client_status.active".to_string(), "Active".to_string());
    translations.insert(
        "client_status.maintenance".to_string(),
        "Maintenance".to_string(),
    );
    translations.insert("client_status.in_stock".to_string(), "In Stock".to_string());
    translations.insert(
        "client_status.decommissioned".to_string(),
        "Decommissioned".to_string(),
    );

    // Environment
    translations.insert("environment.prod".to_string(), "Production".to_string());
    translations.insert("environment.staging".to_string(), "Staging".to_string());
    translations.insert("environment.test".to_string(), "Test".to_string());
    translations.insert("environment.dev".to_string(), "Development".to_string());

    translations.insert(
        "client_edit.title".to_string(),
        "Edit Client Info".to_string(),
    );
    translations.insert(
        "client_edit.description".to_string(),
        "Modify client details including location, associations, status, etc.".to_string(),
    );
    translations.insert(
        "client_edit.unassigned".to_string(),
        "Unassigned".to_string(),
    );
    translations.insert("client_edit.save".to_string(), "Save".to_string());
    translations.insert("client_edit.cancel".to_string(), "Cancel".to_string());
    translations.insert(
        "client_edit.primary_ip_hint".to_string(),
        "Setting this overrides auto-detected primary IP".to_string(),
    );
    translations.insert(
        "client_edit.primary_ip_none".to_string(),
        "None".to_string(),
    );
    translations.insert(
        "client_edit.primary_ip_select".to_string(),
        "Select an IP...".to_string(),
    );
    translations.insert(
        "client_edit.primary_ip_custom".to_string(),
        "Custom...".to_string(),
    );

    translations
}
