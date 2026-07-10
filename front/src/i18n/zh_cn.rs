use std::collections::HashMap;

pub fn get_translations() -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // 通用
    translations.insert("all".to_string(), "全部".to_string());
    translations.insert("name".to_string(), "名称".to_string());
    translations.insert("mode".to_string(), "模式".to_string());
    translations.insert("unknown".to_string(), "未知".to_string());
    translations.insert("online".to_string(), "在线".to_string());
    translations.insert("offline".to_string(), "离线".to_string());
    translations.insert("None".to_string(), "无".to_string());
    translations.insert("none".to_string(), "无".to_string());
    translations.insert("Role".to_string(), "角色".to_string());
    translations.insert("never".to_string(), "从未".to_string());
    translations.insert("count".to_string(), "个".to_string());

    // 硬件单位
    translations.insert("cores".to_string(), "核".to_string());
    translations.insert("threads".to_string(), "线程".to_string());
    translations.insert("gb".to_string(), "GB".to_string());
    translations.insert("mhz".to_string(), "MHz".to_string());
    translations.insert("ghz".to_string(), "GHz".to_string());
    translations.insert("nics".to_string(), "个网卡".to_string());

    // 硬件类别
    translations.insert("cpu_config".to_string(), "CPU配置".to_string());
    translations.insert("memory_config".to_string(), "内存配置".to_string());
    translations.insert("gpu_config".to_string(), "GPU配置".to_string());
    translations.insert("storage_config".to_string(), "存储配置".to_string());
    translations.insert("network_config".to_string(), "网卡配置".to_string());
    translations.insert("operating_system".to_string(), "操作系统".to_string());
    translations.insert("server_model".to_string(), "服务器型号".to_string());

    // Hardware Component Titles
    translations.insert("cpu.title".to_string(), "CPU".to_string());
    translations.insert("gpu.title".to_string(), "GPU".to_string());
    translations.insert("memory.title".to_string(), "内存".to_string());
    translations.insert("network.title".to_string(), "网络".to_string());
    translations.insert("storage.title".to_string(), "存储".to_string());

    // Hardware Component Titles (with hardware. prefix)
    translations.insert("hardware.cpu.title".to_string(), "CPU".to_string());
    translations.insert("hardware.gpu.title".to_string(), "GPU".to_string());
    translations.insert("hardware.memory.title".to_string(), "内存".to_string());
    translations.insert("hardware.network.title".to_string(), "网络".to_string());
    translations.insert("hardware.storage.title".to_string(), "存储".to_string());

    // Hardware Labels
    translations.insert("label.vendor".to_string(), "厂商".to_string());
    translations.insert("label.model".to_string(), "型号".to_string());
    translations.insert("label.frequency".to_string(), "频率".to_string());
    translations.insert("label.cores".to_string(), "核心数".to_string());
    translations.insert("label.threads".to_string(), "线程数".to_string());
    translations.insert("label.device_id".to_string(), "设备ID".to_string());
    translations.insert("label.driver_version".to_string(), "驱动版本".to_string());
    translations.insert("label.serial_number".to_string(), "序列号".to_string());
    translations.insert("label.capacity".to_string(), "容量".to_string());
    translations.insert("label.speed".to_string(), "速度".to_string());
    translations.insert("label.firmware".to_string(), "固件".to_string());
    translations.insert("label.interface".to_string(), "接口".to_string());
    translations.insert("label.size".to_string(), "大小".to_string());
    translations.insert("label.type".to_string(), "类型".to_string());
    translations.insert("label.manufacturer".to_string(), "制造商".to_string());
    translations.insert("label.temperature".to_string(), "温度".to_string());
    translations.insert("label.voltage".to_string(), "电压".to_string());
    translations.insert("label.power".to_string(), "功率".to_string());
    translations.insert("label.utilization".to_string(), "利用率".to_string());

    // Hardware Labels (with hardware. prefix)
    translations.insert("hardware.label.vendor".to_string(), "厂商".to_string());
    translations.insert("hardware.label.model".to_string(), "型号".to_string());
    translations.insert("hardware.label.frequency".to_string(), "频率".to_string());
    translations.insert("hardware.label.cores".to_string(), "核心数".to_string());
    translations.insert("hardware.label.threads".to_string(), "线程数".to_string());
    translations.insert("hardware.label.device_id".to_string(), "设备ID".to_string());
    translations.insert(
        "hardware.label.driver_version".to_string(),
        "驱动版本".to_string(),
    );
    translations.insert(
        "hardware.label.serial_number".to_string(),
        "序列号".to_string(),
    );
    translations.insert("hardware.label.capacity".to_string(), "容量".to_string());
    translations.insert("hardware.label.speed".to_string(), "速度".to_string());
    translations.insert(
        "hardware.label.firmware_version".to_string(),
        "固件版本".to_string(),
    );
    translations.insert("hardware.label.slot".to_string(), "插槽".to_string());
    translations.insert("hardware.label.type".to_string(), "类型".to_string());
    translations.insert(
        "hardware.label.part_number".to_string(),
        "部件号".to_string(),
    );
    translations.insert(
        "hardware.label.memory_count".to_string(),
        "内存数量".to_string(),
    );
    translations.insert("hardware.label.sticks".to_string(), "条".to_string());
    translations.insert("hardware.label.devices".to_string(), "设备".to_string());
    translations.insert("hardware.label.unknown".to_string(), "未知".to_string());
    translations.insert(
        "hardware.label.interface_name".to_string(),
        "接口名称".to_string(),
    );
    translations.insert(
        "hardware.label.nic_type".to_string(),
        "网卡类型".to_string(),
    );
    translations.insert("hardware.label.pci_slot".to_string(), "PCI插槽".to_string());
    translations.insert("hardware.label.bandwidth".to_string(), "带宽".to_string());
    translations.insert("hardware.label.status".to_string(), "状态".to_string());
    translations.insert("hardware.label.driver".to_string(), "驱动".to_string());
    translations.insert(
        "hardware.label.ib_node_type".to_string(),
        "IB节点类型".to_string(),
    );
    translations.insert("hardware.label.dhcp".to_string(), "DHCP".to_string());
    translations.insert(
        "hardware.label.ip_address".to_string(),
        "IP地址".to_string(),
    );
    translations.insert(
        "hardware.label.subnet_mask".to_string(),
        "子网掩码".to_string(),
    );
    translations.insert("hardware.label.gateway".to_string(), "网关".to_string());
    translations.insert("hardware.label.channel".to_string(), "通道".to_string());
    translations.insert("hardware.label.user_id".to_string(), "用户ID".to_string());
    translations.insert("hardware.label.username".to_string(), "用户名".to_string());
    translations.insert("hardware.label.privilege".to_string(), "权限".to_string());

    // Hardware History
    translations.insert("history.change".to_string(), "变更".to_string());
    translations.insert("history.change_type".to_string(), "变更类型".to_string());
    translations.insert("history.empty".to_string(), "无变更记录".to_string());
    translations.insert("history.loading".to_string(), "加载变更中...".to_string());
    translations.insert("history.time".to_string(), "时间".to_string());
    translations.insert("history.title".to_string(), "硬件历史".to_string());
    translations.insert("history.view_details".to_string(), "查看详情".to_string());

    // Hardware History (with hardware. prefix)
    translations.insert("hardware.history.title".to_string(), "硬件历史".to_string());
    translations.insert("hardware.history.change".to_string(), "变更".to_string());
    translations.insert(
        "hardware.history.change_type".to_string(),
        "变更类型".to_string(),
    );
    translations.insert(
        "hardware.history.empty".to_string(),
        "无历史记录".to_string(),
    );
    translations.insert(
        "hardware.history.loading".to_string(),
        "加载中...".to_string(),
    );
    translations.insert("hardware.history.time".to_string(), "时间".to_string());
    translations.insert(
        "hardware.history.view_details".to_string(),
        "查看详情".to_string(),
    );
    translations.insert("common.refresh".to_string(), "刷新".to_string());
    translations.insert("common.load_more".to_string(), "加载更多".to_string());

    // Execution
    translations.insert("execution.status.pending".to_string(), "待执行".to_string());
    translations.insert("execution.status.running".to_string(), "执行中".to_string());
    translations.insert("execution.status.success".to_string(), "成功".to_string());
    translations.insert("execution.status.failed".to_string(), "失败".to_string());
    translations.insert("execution.status.partial".to_string(), "部分成功".to_string());
    translations.insert("execution.status.timeout".to_string(), "超时".to_string());
    translations.insert("execution.status.expired".to_string(), "已过期".to_string());
    translations.insert("execution.type.terminal".to_string(), "终端".to_string());
    translations.insert("execution.type.batch".to_string(), "批量执行".to_string());
    translations.insert("execution.actions.reset_filters".to_string(), "重置筛选".to_string());
    translations.insert("execution.actions.hide".to_string(), "收起".to_string());
    translations.insert("execution.actions.details".to_string(), "详情".to_string());
    translations.insert("execution.actions.rerun".to_string(), "重新执行".to_string());
    translations.insert("execution.actions.replay".to_string(), "查看回放".to_string());
    translations.insert("execution.history.title".to_string(), "执行历史".to_string());
    translations.insert("execution.history.description".to_string(), "统一查看终端会话与批量执行的历史记录。".to_string());
    translations.insert("execution.history.records_badge".to_string(), "{count} 条记录".to_string());
    translations.insert("execution.history.search_label".to_string(), "搜索执行记录".to_string());
    translations.insert("execution.history.search_placeholder".to_string(), "命令 / 用户 / 会话 ID / client id".to_string());
    translations.insert("execution.history.client_scope".to_string(), "客户端范围".to_string());
    translations.insert("execution.history.status".to_string(), "状态".to_string());
    translations.insert("execution.history.execution_type".to_string(), "执行类型".to_string());
    translations.insert("execution.history.from".to_string(), "开始日期".to_string());
    translations.insert("execution.history.to".to_string(), "结束日期".to_string());
    translations.insert("execution.history.loading".to_string(), "正在加载执行历史...".to_string());
    translations.insert("execution.history.empty".to_string(), "当前筛选条件下没有匹配的执行记录。".to_string());
    translations.insert("execution.history.all_clients".to_string(), "全部客户端".to_string());
    translations.insert("execution.history.all_statuses".to_string(), "全部状态".to_string());
    translations.insert("execution.history.all_execution_types".to_string(), "全部执行类型".to_string());
    translations.insert("execution.history.table.time".to_string(), "时间".to_string());
    translations.insert("execution.history.table.type".to_string(), "类型".to_string());
    translations.insert("execution.history.table.targets".to_string(), "目标".to_string());
    translations.insert("execution.history.table.command".to_string(), "命令".to_string());
    translations.insert("execution.history.table.status".to_string(), "状态".to_string());
    translations.insert("execution.history.table.duration".to_string(), "耗时".to_string());
    translations.insert("execution.history.table.user".to_string(), "用户".to_string());
    translations.insert("execution.history.table.actions".to_string(), "操作".to_string());
    translations.insert("execution.history.full_command".to_string(), "完整命令 / 脚本".to_string());
    translations.insert("execution.history.status_summary".to_string(), "状态摘要".to_string());
    translations.insert("execution.history.started".to_string(), "开始时间".to_string());
    translations.insert("execution.history.ended".to_string(), "结束时间".to_string());
    translations.insert("execution.history.recording".to_string(), "录制".to_string());
    translations.insert("execution.history.recording_available".to_string(), "可用".to_string());
    translations.insert("execution.history.recording_missing".to_string(), "未录制".to_string());
    translations.insert("execution.history.details_title".to_string(), "执行详情".to_string());
    translations.insert("execution.history.details_description".to_string(), "集中查看命令、输出和执行元数据。".to_string());
    translations.insert("execution.history.open_replay".to_string(), "前往回放".to_string());
    translations.insert("execution.batch.title".to_string(), "批量执行".to_string());
    translations.insert("execution.batch.description".to_string(), "面向多客户端的命令执行工作区，保留原始命令内容并按客户端查看结果。".to_string());
    translations.insert("execution.batch.selected_badge".to_string(), "已选 {count} 台".to_string());
    translations.insert("execution.batch.limit_badge".to_string(), "超过 50 台上限".to_string());
    translations.insert("execution.batch.preloaded_badge".to_string(), "来自历史预填".to_string());
    translations.insert("execution.batch.back_to_workspace".to_string(), "返回工作区".to_string());
    translations.insert("execution.batch.targets".to_string(), "目标列表".to_string());
    translations.insert("execution.batch.running".to_string(), "运行中".to_string());
    translations.insert("execution.batch.completed".to_string(), "已完成".to_string());
    translations.insert("execution.batch.output_title".to_string(), "单客户端输出".to_string());
    translations.insert("execution.batch.output_description".to_string(), "逐台查看输出，同时保留完整批量执行上下文。".to_string());
    translations.insert("execution.batch.command_label".to_string(), "命令".to_string());
    translations.insert("execution.batch.target_client".to_string(), "目标客户端".to_string());
    translations.insert("execution.batch.task_id".to_string(), "任务 ID".to_string());
    translations.insert("execution.batch.waiting_output".to_string(), "等待客户端输出...".to_string());
    translations.insert("execution.batch.no_output_terminal".to_string(), "任务已结束，但客户端没有返回输出。若为失败任务，请检查错误日志、命令白名单或客户端执行环境。".to_string());
    translations.insert("execution.batch.select_target_prompt".to_string(), "请先从左侧选择一台目标客户端查看输出。".to_string());
    translations.insert("execution.batch.select_targets".to_string(), "选择目标".to_string());
    translations.insert("execution.batch.search_label".to_string(), "搜索客户端".to_string());
    translations.insert("execution.batch.search_placeholder".to_string(), "主机名 / IP / client id / 序列号 / 位置 / 机架".to_string());
    translations.insert("execution.batch.project_filter".to_string(), "项目".to_string());
    translations.insert("execution.batch.rack_filter".to_string(), "机架 / 位置".to_string());
    translations.insert("execution.batch.all_projects".to_string(), "全部项目".to_string());
    translations.insert("execution.batch.all_racks".to_string(), "全部机架 / 位置".to_string());
    translations.insert("execution.batch.select_visible".to_string(), "选择当前结果".to_string());
    translations.insert("execution.batch.clear_all".to_string(), "清空全部".to_string());
    translations.insert("execution.batch.matches".to_string(), "{count} 条匹配".to_string());
    translations.insert("execution.batch.summary_title".to_string(), "选择摘要".to_string());
    translations.insert("execution.batch.reduce_limit".to_string(), "请将选择数量减少到 50 台以内".to_string());
    translations.insert("execution.batch.summary_empty".to_string(), "请从左侧选择一个或多个客户端。也可以从历史记录跳转过来并带上预选目标。".to_string());
    translations.insert("execution.batch.editor_title".to_string(), "命令或脚本".to_string());
    translations.insert("execution.batch.editor_help".to_string(), "重新执行时会保留原始输入：参数、空白和多行脚本都会按下方内容原样提交。".to_string());
    translations.insert("execution.batch.editor_placeholder".to_string(), "单行命令或多行 Shell 脚本\nShift+Enter：换行\nCtrl+Enter：执行".to_string());
    translations.insert("execution.batch.shortcut_newline".to_string(), "Shift+Enter = 换行".to_string());
    translations.insert("execution.batch.shortcut_execute".to_string(), "Ctrl+Enter = 执行".to_string());
    translations.insert("execution.batch.submitting".to_string(), "提交中...".to_string());
    translations.insert("execution.batch.execute".to_string(), "对所选客户端执行".to_string());
    translations.insert("execution.batch.validation.command_required".to_string(), "请输入命令或脚本".to_string());
    translations.insert("execution.batch.validation.targets_required".to_string(), "请至少选择一台客户端".to_string());
    translations.insert("execution.batch.validation.limit".to_string(), "批量执行最多支持 50 台客户端".to_string());
    translations.insert("execution.batch.notifications.no_tasks".to_string(), "没有成功创建任何任务".to_string());
    translations.insert("execution.batch.notifications.partial_failures".to_string(), "批量执行已启动，但有 {count} 个任务提交失败".to_string());
    translations.insert("execution.batch.notifications.started".to_string(), "批量执行已启动".to_string());
    translations.insert("execution.batch.notifications.approvals_created".to_string(), "有 {count} 个目标已提交审批".to_string());
    translations.insert("execution.replay.title".to_string(), "执行回放".to_string());
    translations.insert("execution.replay.description".to_string(), "查看已完成或进行中的执行会话所录制的输出内容。".to_string());
    translations.insert("execution.replay.loading".to_string(), "正在加载回放...".to_string());
    translations.insert("execution.replay.load_session_error".to_string(), "加载会话失败".to_string());
    translations.insert("execution.replay.load_output_error".to_string(), "加载回放输出失败".to_string());
    translations.insert("execution.replay.command".to_string(), "命令".to_string());
    translations.insert("execution.replay.status".to_string(), "状态".to_string());
    translations.insert("execution.replay.output_title".to_string(), "录制输出".to_string());
    translations.insert("execution.replay.output_description".to_string(), "回放内容按会话录制时的原始结果展示。".to_string());
    translations.insert("execution.replay.back".to_string(), "返回执行历史".to_string());
    translations.insert("execution.replay.read_only".to_string(), "只读".to_string());
    translations.insert("execution.replay.waiting".to_string(), "等待回放数据...".to_string());
    translations.insert("execution.settings.title".to_string(), "远程执行设置".to_string());
    translations.insert("execution.settings.description".to_string(), "控制是否允许运维人员向已连接客户端提交远程命令。".to_string());
    translations.insert("execution.settings.enabled".to_string(), "已启用".to_string());
    translations.insert("execution.settings.disabled".to_string(), "已禁用".to_string());
    translations.insert("execution.settings.loading".to_string(), "正在加载远程执行设置...".to_string());
    translations.insert("execution.settings.toggle_title".to_string(), "启用远程命令执行".to_string());
    translations.insert("execution.settings.toggle_description".to_string(), "启用后，管理员可通过 CMDB 的执行工作流下发远程命令。".to_string());
    translations.insert("execution.settings.allow".to_string(), "允许远程执行".to_string());
    translations.insert("execution.settings.block".to_string(), "阻止远程执行".to_string());
    translations.insert("execution.settings.warning".to_string(), "关闭该开关后，应立即拒绝新的命令提交，同时保留历史记录与录制数据。".to_string());
    translations.insert("execution.settings.saving".to_string(), "保存中...".to_string());
    translations.insert("execution.settings.save".to_string(), "保存设置".to_string());
    translations.insert("execution.settings.updated".to_string(), "远程执行配置已更新".to_string());
    translations.insert("execution.actions.cancel".to_string(), "取消".to_string());
    translations.insert("execution.danger.safe".to_string(), "安全".to_string());
    translations.insert("execution.danger.warning".to_string(), "警告".to_string());
    translations.insert("execution.danger.blocked".to_string(), "已阻止".to_string());
    translations.insert("execution.terminal.title".to_string(), "Web 终端".to_string());
    translations.insert("execution.terminal.description".to_string(), "面向单台客户端的实时命令工作区，可持续查看流式输出。".to_string());
    translations.insert("execution.terminal.auto_attach_hint".to_string(), "进入页面后会自动接管活动终端；如果当前没有可用会话，系统会自动创建一个新终端。".to_string());
    translations.insert("execution.terminal.auto_create_hint".to_string(), "每次打开页面都会创建当前浏览器专属的终端会话；离开页面后会自动关闭。".to_string());
    translations.insert("execution.terminal.console_title".to_string(), "终端控制台".to_string());
    translations.insert("execution.terminal.console_description".to_string(), "进入页面后会自动接管活动终端，没有会话时自动创建；这里只保留尺寸调整、手动刷新和会话切换。".to_string());
    translations.insert("execution.terminal.console_new_session_description".to_string(), "当前页面会创建独立终端，这里只保留尺寸调整、手动重连和历史入口。".to_string());
    translations.insert("execution.terminal.mode_hint".to_string(), "受限终端并不是纯查看模式，它仍可输入命令，但只能执行策略明确放行的命令。".to_string());
    translations.insert("execution.terminal.switchable_sessions".to_string(), "可切换会话".to_string());
    translations.insert("execution.terminal.live_shell".to_string(), "实时 Shell".to_string());
    translations.insert("execution.terminal.enabled".to_string(), "远程执行已启用".to_string());
    translations.insert("execution.terminal.disabled".to_string(), "远程执行已禁用".to_string());
    translations.insert("execution.terminal.view_history".to_string(), "查看历史".to_string());
    translations.insert("execution.terminal.reconnect".to_string(), "重新连接输出流".to_string());
    translations.insert("execution.terminal.close".to_string(), "关闭会话".to_string());
    translations.insert("execution.terminal.target_client".to_string(), "目标客户端".to_string());
    translations.insert("execution.terminal.hostname".to_string(), "主机名".to_string());
    translations.insert("execution.terminal.ip".to_string(), "IP".to_string());
    translations.insert("execution.terminal.os".to_string(), "操作系统".to_string());
    translations.insert("execution.terminal.current_session".to_string(), "当前会话".to_string());
    translations.insert("execution.terminal.available_sessions".to_string(), "可附着会话".to_string());
    translations.insert("execution.terminal.no_available_sessions".to_string(), "当前客户端没有可复用的终端会话。".to_string());
    translations.insert("execution.terminal.attach".to_string(), "附着".to_string());
    translations.insert("execution.terminal.attached".to_string(), "已附着".to_string());
    translations.insert("execution.terminal.session_id".to_string(), "会话 ID".to_string());
    translations.insert("execution.terminal.created_at".to_string(), "创建时间".to_string());
    translations.insert("execution.terminal.last_activity".to_string(), "最近活动".to_string());
    translations.insert("execution.terminal.last_heartbeat".to_string(), "最近心跳".to_string());
    translations.insert("execution.terminal.lease".to_string(), "租约到期".to_string());
    translations.insert("execution.terminal.live_terminal".to_string(), "实时终端".to_string());
    translations.insert("execution.terminal.live_terminal_description".to_string(), "在浏览器中打开真正的交互式 Shell。键盘输入会直接流式发送到远端 PTY。".to_string());
    translations.insert("execution.terminal.live_terminal_empty".to_string(), "请先打开或附着到一个终端会话，然后开始交互。".to_string());
    translations.insert("execution.terminal.browser_terminal_note".to_string(), "当前视图已改为浏览器终端：按键、粘贴和回车都会作为终端输入直接发送，而不是把左侧文本框整段提交。".to_string());
    translations.insert("execution.terminal.open_terminal".to_string(), "打开终端".to_string());
    translations.insert("execution.terminal.opening_terminal".to_string(), "正在打开终端...".to_string());
    translations.insert("execution.terminal.connecting".to_string(), "连接中".to_string());
    translations.insert("execution.terminal.connected".to_string(), "已连接".to_string());
    translations.insert("execution.terminal.disconnected".to_string(), "已断开".to_string());
    translations.insert("execution.terminal.preloaded_command".to_string(), "预填命令".to_string());
    translations.insert("execution.terminal.notifications.opened".to_string(), "终端会话已打开。".to_string());
    translations.insert("execution.terminal.task_id".to_string(), "任务 ID".to_string());
    translations.insert("execution.terminal.exit_code".to_string(), "退出码".to_string());
    translations.insert("execution.terminal.shell".to_string(), "Shell".to_string());
    translations.insert("execution.terminal.size".to_string(), "终端尺寸".to_string());
    translations.insert("execution.terminal.cols".to_string(), "列数".to_string());
    translations.insert("execution.terminal.rows".to_string(), "行数".to_string());
    translations.insert("execution.terminal.state".to_string(), "状态".to_string());
    translations.insert("execution.terminal.state_closed".to_string(), "已关闭".to_string());
    translations.insert("execution.terminal.mode_read_only".to_string(), "只读".to_string());
    translations.insert("execution.terminal.mode_read_write".to_string(), "可写".to_string());
    translations.insert("execution.terminal.mode_restricted".to_string(), "受限终端".to_string());
    translations.insert("execution.terminal.mode_standard".to_string(), "标准终端".to_string());
    translations.insert("permissions.web_terminal.page_description".to_string(), "控制 Web Terminal 的访问模式、会话约束以及受限命令规则。".to_string());
    translations.insert("permissions.web_terminal.modal_description".to_string(), "终端策略独立于命令执行策略；这里只约束 Web Terminal 会话与受限命令行为。".to_string());
    translations.insert("execution.terminal.close_reason".to_string(), "关闭原因".to_string());
    translations.insert("execution.terminal.no_session".to_string(), "当前终端尚未执行任何命令。".to_string());
    translations.insert("execution.terminal.run_command".to_string(), "执行命令".to_string());
    translations.insert("execution.terminal.command_label".to_string(), "命令或脚本".to_string());
    translations.insert("execution.terminal.command_placeholder".to_string(), "支持原样输入命令、参数和多行脚本。".to_string());
    translations.insert("execution.terminal.timeout".to_string(), "超时（秒）".to_string());
    translations.insert("execution.terminal.pinned_note".to_string(), "终端会固定绑定到当前客户端；当前命令结束前不会提交新的命令。".to_string());
    translations.insert("execution.terminal.submitting".to_string(), "提交中...".to_string());
    translations.insert("execution.terminal.running".to_string(), "执行中...".to_string());
    translations.insert("execution.terminal.execute".to_string(), "在终端中执行".to_string());
    translations.insert("execution.terminal.output_title".to_string(), "实时输出".to_string());
    translations.insert("execution.terminal.output_description".to_string(), "展示当前终端命令的流式输出。".to_string());
    translations.insert("execution.terminal.current_command".to_string(), "当前命令".to_string());
    translations.insert("execution.terminal.output_empty".to_string(), "先执行一条命令以打开实时输出流。".to_string());
    translations.insert("execution.terminal.output_waiting".to_string(), "等待输出中...".to_string());
    translations.insert("execution.terminal.client_load_failed".to_string(), "无法加载所选客户端，暂时不能进入终端。".to_string());
    translations.insert("execution.terminal.validation.command_required".to_string(), "请输入命令".to_string());
    translations.insert("execution.terminal.notifications.submitted".to_string(), "命令已提交".to_string());
    translations.insert("execution.terminal.notifications.closed".to_string(), "已请求关闭终端会话".to_string());
    translations.insert("execution.terminal.danger_title".to_string(), "检测到危险命令".to_string());
    translations.insert("execution.terminal.matched_rule".to_string(), "匹配规则：{rule}".to_string());
    translations.insert("execution.terminal.confirm_prefix".to_string(), "输入 ".to_string());
    translations.insert("execution.terminal.confirm_suffix".to_string(), " 以继续。".to_string());
    translations.insert("execution.terminal.confirm_placeholder".to_string(), "CONFIRM".to_string());
    translations.insert("execution.terminal.execute_anyway".to_string(), "仍然执行".to_string());
    translations.insert("execution.terminal.disabled_message".to_string(), "当前策略已禁用远程命令执行。".to_string());
    translations.insert("execution.terminal.websocket_error".to_string(), "终端连接异常，请稍后重试。".to_string());
    translations.insert("execution.ops.summary.exec_policies".to_string(), "执行策略".to_string());
    translations.insert("execution.ops.summary.web_terminal_policies".to_string(), "终端策略".to_string());
    translations.insert("execution.ops.summary.pending_approvals".to_string(), "待审批".to_string());
    translations.insert("execution.ops.summary.active_terminal_sessions".to_string(), "活跃终端会话".to_string());
    translations.insert("execution.ops.overview.title".to_string(), "远程执行运维总览".to_string());
    translations.insert("execution.ops.overview.description".to_string(), "集中查看审批积压、终端活跃度、会话陈旧阈值和录屏存储位置。".to_string());
    translations.insert("execution.ops.overview.total_sessions".to_string(), "总终端会话".to_string());
    translations.insert("execution.ops.overview.active_clients".to_string(), "活跃客户端".to_string());
    translations.insert("execution.ops.overview.pending_active".to_string(), "待激活 / 活跃".to_string());
    translations.insert("execution.ops.overview.stale_threshold".to_string(), "陈旧阈值".to_string());
    translations.insert("execution.ops.overview.cast_storage_dir".to_string(), "Cast 存储目录".to_string());
    translations.insert("execution.ops.actions.refresh".to_string(), "刷新运维数据".to_string());
    translations.insert("execution.ops.actions.force_close".to_string(), "强制关闭".to_string());
    translations.insert("execution.ops.cleanup.retention_days".to_string(), "清理保留天数".to_string());
    translations.insert("execution.ops.cleanup.run".to_string(), "执行 Cast 清理".to_string());
    translations.insert("execution.ops.terminal_governance.title".to_string(), "终端会话治理".to_string());
    translations.insert("execution.ops.terminal_governance.empty".to_string(), "当前没有终端会话".to_string());
    translations.insert("execution.ops.terminal_governance.ended".to_string(), "已结束".to_string());
    translations.insert("execution.ops.terminal_state.pending".to_string(), "待激活".to_string());
    translations.insert("execution.ops.terminal_state.active".to_string(), "活跃".to_string());
    translations.insert("execution.ops.terminal_state.closed".to_string(), "已关闭".to_string());
    translations.insert("execution.ops.terminal_state.failed".to_string(), "失败".to_string());
    translations.insert("execution.ops.terminal_mode.read_only".to_string(), "只读".to_string());
    translations.insert("execution.ops.terminal_mode.read_write".to_string(), "读写".to_string());
    translations.insert("execution.ops.terminal_mode.restricted".to_string(), "受限终端".to_string());
    translations.insert("execution.ops.terminal_mode.standard".to_string(), "标准终端".to_string());
    translations.insert("execution.ops.table.session".to_string(), "会话".to_string());
    translations.insert("execution.ops.table.client".to_string(), "客户端".to_string());
    translations.insert("execution.ops.table.user".to_string(), "用户".to_string());
    translations.insert("execution.ops.table.mode".to_string(), "模式".to_string());
    translations.insert("execution.ops.table.state".to_string(), "状态".to_string());
    translations.insert("execution.ops.table.last_activity".to_string(), "最近活动".to_string());
    translations.insert("execution.ops.table.last_heartbeat".to_string(), "最近心跳".to_string());
    translations.insert("execution.ops.table.close_reason".to_string(), "关闭原因".to_string());
    translations.insert("execution.ops.table.operations".to_string(), "操作".to_string());
    translations.insert("execution.ops.messages.cast_cleanup_prefix".to_string(), "Cast 清理完成，删除".to_string());
    translations.insert("execution.ops.messages.cast_cleanup_suffix".to_string(), "个历史文件".to_string());
    translations.insert("execution.ops.messages.session_closed_prefix".to_string(), "终端会话".to_string());
    translations.insert("execution.ops.messages.session_closed_suffix".to_string(), "已关闭".to_string());
    translations.insert("execution.ops.none".to_string(), "无".to_string());
    translations.insert("execution.history.output_title".to_string(), "执行输出".to_string());
    translations.insert("execution.history.output_pending".to_string(), "执行中，日志会逐步补齐".to_string());
    translations.insert("execution.history.output_empty".to_string(), "暂无执行输出".to_string());
    translations.insert("execution.history.output_replay_hint".to_string(), "该终端会话的输出已写入回放文件，请点击“回放”查看完整终端内容。".to_string());
    translations.insert("permissions.manage.title".to_string(), "权限规则与分组".to_string());
    translations.insert("permissions.manage.tab.rules".to_string(), "规则".to_string());
    translations.insert("permissions.manage.tab.groups".to_string(), "分组".to_string());
    translations.insert("permissions.manage.loading".to_string(), "正在加载权限配置...".to_string());
    translations.insert("permissions.manage.empty_rules".to_string(), "当前还没有权限规则".to_string());
    translations.insert("permissions.manage.empty_groups".to_string(), "当前还没有权限分组".to_string());
    translations.insert("permissions.manage.overview.rules".to_string(), "规则数量".to_string());
    translations.insert("permissions.manage.overview.groups".to_string(), "分组数量".to_string());
    translations.insert("permissions.manage.overview.exec_policies".to_string(), "执行策略".to_string());
    translations.insert("permissions.manage.overview.web_terminal_policies".to_string(), "终端策略".to_string());
    translations.insert("permissions.manage.rules.title".to_string(), "权限规则".to_string());
    translations.insert("permissions.manage.rules.description".to_string(), "维护主体、资源和动作范围，收敛权限判断逻辑。".to_string());
    translations.insert("permissions.manage.groups.title".to_string(), "权限分组".to_string());
    translations.insert("permissions.manage.groups.description".to_string(), "维护用户分组，为更细粒度的授权和策略匹配做准备。".to_string());
    translations.insert("permissions.manage.subject.role".to_string(), "角色".to_string());
    translations.insert("permissions.manage.subject.user".to_string(), "用户".to_string());
    translations.insert("permissions.manage.subject.group".to_string(), "分组".to_string());
    translations.insert("permissions.manage.resource.client".to_string(), "客户端".to_string());
    translations.insert("permissions.manage.resource.component".to_string(), "组件".to_string());
    translations.insert("permissions.manage.resource.rack".to_string(), "机架".to_string());
    translations.insert("permissions.manage.resource.person".to_string(), "人员".to_string());
    translations.insert("permissions.manage.resource.project".to_string(), "项目".to_string());
    translations.insert("permissions.manage.resource.dictionary".to_string(), "字典".to_string());
    translations.insert("permissions.manage.resource.command".to_string(), "命令执行".to_string());
    translations.insert("permissions.manage.resource.user".to_string(), "用户管理".to_string());
    translations.insert("permissions.manage.action.view".to_string(), "查看".to_string());
    translations.insert("permissions.manage.action.create".to_string(), "创建".to_string());
    translations.insert("permissions.manage.action.update".to_string(), "更新".to_string());
    translations.insert("permissions.manage.action.delete".to_string(), "删除".to_string());
    translations.insert("permissions.manage.constraint.unset".to_string(), "未设置".to_string());
    translations.insert("permissions.manage.constraint.all".to_string(), "全部资源".to_string());
    translations.insert("permissions.manage.constraint.owned".to_string(), "仅本人资源".to_string());
    translations.insert("permissions.manage.constraint.project_scope".to_string(), "指定项目".to_string());
    translations.insert("permissions.manage.constraint.project_list".to_string(), "项目".to_string());
    translations.insert("permissions.manage.constraint.tag_scope".to_string(), "指定标签".to_string());
    translations.insert("permissions.manage.constraint.tag_list".to_string(), "标签".to_string());
    translations.insert("permissions.manage.constraint.none".to_string(), "无权限范围".to_string());
    translations.insert("permissions.manage.rule.id".to_string(), "规则 ID".to_string());
    translations.insert("permissions.manage.rule.subject_type".to_string(), "主体类型".to_string());
    translations.insert("permissions.manage.rule.subject_id".to_string(), "主体 ID".to_string());
    translations.insert("permissions.manage.rule.resource_type".to_string(), "资源类型".to_string());
    translations.insert("permissions.manage.rule.actions".to_string(), "动作".to_string());
    translations.insert("permissions.manage.rule.actions_help".to_string(), "勾选该规则允许的动作，可多选。".to_string());
    translations.insert("permissions.manage.rule.constraint".to_string(), "作用范围".to_string());
    translations.insert("permissions.manage.rule.constraint_values_placeholder".to_string(), "多个值用英文逗号分隔，例如 project-a, project-b".to_string());
    translations.insert("permissions.manage.rule.constraint_help".to_string(), "选择资源作用范围。指定项目或标签时，在下方的输入框填写具体值。".to_string());
    translations.insert("permissions.manage.rule.priority".to_string(), "优先级".to_string());
    translations.insert("permissions.manage.rule.constraint_invalid".to_string(), "约束 JSON 无法解析".to_string());
    translations.insert("permissions.manage.group.id".to_string(), "分组 ID".to_string());
    translations.insert("permissions.manage.group.name".to_string(), "分组名称".to_string());
    translations.insert("permissions.manage.group.members".to_string(), "成员 ID".to_string());
    translations.insert("permissions.manage.group.members_help".to_string(), "支持换行或逗号分隔多个成员 ID。".to_string());
    translations.insert("permissions.manage.table.id".to_string(), "ID".to_string());
    translations.insert("permissions.manage.table.subject".to_string(), "主体".to_string());
    translations.insert("permissions.manage.table.resource".to_string(), "资源".to_string());
    translations.insert("permissions.manage.table.actions".to_string(), "动作".to_string());
    translations.insert("permissions.manage.table.constraint".to_string(), "约束".to_string());
    translations.insert("permissions.manage.table.priority".to_string(), "优先级".to_string());
    translations.insert("permissions.manage.table.operations".to_string(), "操作".to_string());
    translations.insert("permissions.manage.table.name".to_string(), "名称".to_string());
    translations.insert("permissions.manage.table.member_count".to_string(), "成员数".to_string());
    translations.insert("permissions.manage.table.members".to_string(), "成员".to_string());
    translations.insert("permissions.manage.actions.cancel".to_string(), "取消".to_string());
    translations.insert("permissions.manage.actions.new_rule".to_string(), "新建规则".to_string());
    translations.insert("permissions.manage.actions.save_rule".to_string(), "保存规则".to_string());
    translations.insert("permissions.manage.actions.update_rule".to_string(), "更新规则".to_string());
    translations.insert("permissions.manage.actions.new_group".to_string(), "新建分组".to_string());
    translations.insert("permissions.manage.actions.save_group".to_string(), "保存分组".to_string());
    translations.insert("permissions.manage.actions.update_group".to_string(), "更新分组".to_string());
    translations.insert("permissions.manage.actions.edit".to_string(), "编辑".to_string());
    translations.insert("permissions.manage.actions.delete".to_string(), "删除".to_string());
    translations.insert("permissions.manage.messages.rule_saved".to_string(), "权限规则已保存".to_string());
    translations.insert("permissions.manage.messages.rule_deleted".to_string(), "权限规则已删除".to_string());
    translations.insert("permissions.manage.messages.group_saved".to_string(), "权限分组已保存".to_string());
    translations.insert("permissions.manage.messages.group_deleted".to_string(), "权限分组已删除".to_string());
    translations.insert("permissions.manage.messages.default_protected".to_string(), "默认规则/分组不允许直接删除。".to_string());
    translations.insert("permissions.manage.labels.default".to_string(), "默认".to_string());
    translations.insert("permissions.approvals.title".to_string(), "审批".to_string());
    translations.insert("permissions.approvals.tab.pending".to_string(), "待审批".to_string());
    translations.insert("permissions.approvals.tab.mine".to_string(), "我的".to_string());
    translations.insert("permissions.approvals.loading".to_string(), "加载中...".to_string());
    translations.insert("permissions.approvals.empty".to_string(), "暂无审批".to_string());
    translations.insert("permissions.approvals.none".to_string(), "无".to_string());
    translations.insert("permissions.approvals.summary.total".to_string(), "总数".to_string());
    translations.insert("permissions.approvals.summary.pending".to_string(), "待审批".to_string());
    translations.insert("permissions.approvals.summary.approved".to_string(), "已批准".to_string());
    translations.insert("permissions.approvals.summary.rejected".to_string(), "已驳回".to_string());
    translations.insert("permissions.approvals.summary.expired".to_string(), "已过期".to_string());
    translations.insert("permissions.approvals.summary.executed".to_string(), "已转执行".to_string());
    translations.insert("permissions.approvals.status.pending".to_string(), "待审批".to_string());
    translations.insert("permissions.approvals.status.approved".to_string(), "已批准".to_string());
    translations.insert("permissions.approvals.status.rejected".to_string(), "已驳回".to_string());
    translations.insert("permissions.approvals.status.expired".to_string(), "已过期".to_string());
    translations.insert("permissions.approvals.policy_type.command".to_string(), "命令执行".to_string());
    translations.insert("permissions.approvals.table.id".to_string(), "ID".to_string());
    translations.insert("permissions.approvals.table.type".to_string(), "类型".to_string());
    translations.insert("permissions.approvals.table.requester".to_string(), "申请人".to_string());
    translations.insert("permissions.approvals.table.client".to_string(), "客户端".to_string());
    translations.insert("permissions.approvals.table.command".to_string(), "命令".to_string());
    translations.insert("permissions.approvals.table.task".to_string(), "执行任务".to_string());
    translations.insert("permissions.approvals.table.status".to_string(), "状态".to_string());
    translations.insert("permissions.approvals.table.created_at".to_string(), "创建时间".to_string());
    translations.insert("permissions.approvals.table.operations".to_string(), "操作".to_string());
    translations.insert("permissions.approvals.actions.go_history".to_string(), "前往执行历史".to_string());
    translations.insert("permissions.approvals.actions.approve".to_string(), "批准".to_string());
    translations.insert("permissions.approvals.actions.reject".to_string(), "驳回".to_string());
    translations.insert("permissions.approvals.messages.approved_created_task".to_string(), "已批准，并创建任务".to_string());
    translations.insert("permissions.approvals.messages.approved_updated".to_string(), "已批准，审批单已更新".to_string());
    translations.insert("permissions.approvals.messages.approved".to_string(), "已批准".to_string());
    translations.insert("permissions.approvals.messages.rejected".to_string(), "已驳回".to_string());
    translations.insert("permissions.approvals.messages.task_created".to_string(), "已创建任务".to_string());

    // IPMI
    translations.insert("ipmi.access_denied".to_string(), "访问被拒绝".to_string());
    translations.insert("ipmi.not_available".to_string(), "不可用".to_string());
    translations.insert("ipmi.status_online".to_string(), "在线".to_string());
    translations.insert("ipmi.status_offline".to_string(), "离线".to_string());
    translations.insert("ipmi.status_unknown".to_string(), "未知".to_string());
    translations.insert("ipmi.privilege_admin".to_string(), "管理员".to_string());
    translations.insert("ipmi.privilege_user".to_string(), "用户".to_string());
    translations.insert("ipmi.privilege_operator".to_string(), "操作员".to_string());
    translations.insert("ipmi.privilege_callback".to_string(), "回调".to_string());
    translations.insert("ipmi.users".to_string(), "用户".to_string());
    translations.insert("ipmi.username".to_string(), "用户名".to_string());
    translations.insert("ipmi.password".to_string(), "密码".to_string());
    translations.insert("ipmi.channel".to_string(), "通道".to_string());
    translations.insert("ipmi.ip_address".to_string(), "IP地址".to_string());
    translations.insert("ipmi.mac_address".to_string(), "MAC地址".to_string());
    translations.insert("ipmi.netmask".to_string(), "子网掩码".to_string());
    translations.insert("ipmi.gateway".to_string(), "网关".to_string());

    // IPMI (with hardware. prefix)
    translations.insert("hardware.ipmi.users".to_string(), "用户".to_string());
    translations.insert(
        "hardware.ipmi.status_available".to_string(),
        "可用".to_string(),
    );
    translations.insert("hardware.ipmi.status_error".to_string(), "错误".to_string());
    translations.insert(
        "hardware.ipmi.status_not_configured".to_string(),
        "未配置".to_string(),
    );
    translations.insert(
        "hardware.ipmi.status_not_available".to_string(),
        "不可用".to_string(),
    );
    translations.insert(
        "hardware.ipmi.status_access_denied".to_string(),
        "访问被拒绝".to_string(),
    );
    translations.insert(
        "hardware.ipmi.not_configured".to_string(),
        "IPMI未配置".to_string(),
    );
    translations.insert(
        "hardware.ipmi.not_available".to_string(),
        "IPMI不可用".to_string(),
    );
    translations.insert(
        "hardware.ipmi.access_denied".to_string(),
        "IPMI访问被拒绝".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_callback".to_string(),
        "回调".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_user".to_string(),
        "用户".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_operator".to_string(),
        "操作员".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_admin".to_string(),
        "管理员".to_string(),
    );
    translations.insert(
        "hardware.ipmi.privilege_no_access".to_string(),
        "无访问权限".to_string(),
    );

    // Status
    translations.insert("status.online".to_string(), "在线".to_string());
    translations.insert("status.offline".to_string(), "离线".to_string());
    translations.insert("status.enabled".to_string(), "已启用".to_string());
    translations.insert("status.disabled".to_string(), "已禁用".to_string());
    translations.insert("status.active".to_string(), "活动".to_string());
    translations.insert("status.inactive".to_string(), "非活动".to_string());
    translations.insert("status.unknown".to_string(), "未知".to_string());
    translations.insert("status.available".to_string(), "可用".to_string());
    translations.insert("status.unavailable".to_string(), "不可用".to_string());

    // Status (with hardware. prefix)
    translations.insert("hardware.status.online".to_string(), "在线".to_string());
    translations.insert("hardware.status.offline".to_string(), "离线".to_string());
    translations.insert("hardware.status.enabled".to_string(), "已启用".to_string());
    translations.insert("hardware.status.disabled".to_string(), "已禁用".to_string());

    // Change Types
    translations.insert("change.added".to_string(), "已添加".to_string());
    translations.insert("change.removed".to_string(), "已移除".to_string());
    translations.insert("change.modified".to_string(), "已修改".to_string());
    translations.insert("change.upgraded".to_string(), "已升级".to_string());
    translations.insert("change.downgraded".to_string(), "已降级".to_string());
    translations.insert("change.replaced".to_string(), "已替换".to_string());
    translations.insert("change.migrated".to_string(), "已迁移".to_string());

    // Change Types (with hardware. prefix)
    translations.insert("hardware.change.added".to_string(), "已添加".to_string());
    translations.insert("hardware.change.removed".to_string(), "已移除".to_string());
    translations.insert("hardware.change.modified".to_string(), "已修改".to_string());
    translations.insert("hardware.change.upgraded".to_string(), "已升级".to_string());
    translations.insert(
        "hardware.change.downgraded".to_string(),
        "已降级".to_string(),
    );

    // Network Configuration
    translations.insert(
        "network.bonding_slaves".to_string(),
        "绑定从设备".to_string(),
    );
    translations.insert("network.config".to_string(), "网络配置".to_string());
    translations.insert("network.ipv4_config".to_string(), "IPv4配置".to_string());
    translations.insert("network.ipv6_config".to_string(), "IPv6配置".to_string());
    translations.insert("network.mac_address".to_string(), "MAC地址".to_string());
    translations.insert("network.ip_address".to_string(), "IP地址".to_string());
    translations.insert("network.subnet_mask".to_string(), "子网掩码".to_string());
    translations.insert("network.gateway".to_string(), "网关".to_string());
    translations.insert("network.dns_servers".to_string(), "DNS服务器".to_string());
    translations.insert("network.speed".to_string(), "速度".to_string());
    translations.insert("network.duplex".to_string(), "双工".to_string());
    translations.insert("network.mtu".to_string(), "MTU".to_string());
    translations.insert("network.bond_mode".to_string(), "绑定模式".to_string());
    translations.insert("network.vlan".to_string(), "VLAN".to_string());

    // Network Configuration (with hardware. prefix)
    translations.insert(
        "hardware.network.config".to_string(),
        "网络配置".to_string(),
    );
    translations.insert(
        "hardware.network.ipv4_config".to_string(),
        "IPv4配置".to_string(),
    );
    translations.insert(
        "hardware.network.ipv6_config".to_string(),
        "IPv6配置".to_string(),
    );
    translations.insert(
        "hardware.network.bonding_slaves".to_string(),
        "绑定从设备".to_string(),
    );

    // Storage (with hardware. prefix)
    translations.insert(
        "hardware.storage.partitions".to_string(),
        "分区".to_string(),
    );
    translations.insert("network.bridge".to_string(), "网桥".to_string());

    // Storage
    translations.insert("storage.partitions".to_string(), "分区".to_string());
    translations.insert("storage.partition".to_string(), "分区".to_string());
    translations.insert("storage.mount_point".to_string(), "挂载点".to_string());
    translations.insert("storage.file_system".to_string(), "文件系统".to_string());
    translations.insert("storage.used".to_string(), "已用".to_string());
    translations.insert("storage.available".to_string(), "可用".to_string());
    translations.insert("storage.usage_percent".to_string(), "使用率".to_string());
    translations.insert("storage.disk_type".to_string(), "磁盘类型".to_string());
    translations.insert("storage.rotational_speed".to_string(), "转速".to_string());
    translations.insert("storage.form_factor".to_string(), "外形尺寸".to_string());
    translations.insert("storage.smart_status".to_string(), "SMART状态".to_string());

    // Memory
    translations.insert(
        "memory.modules_detail".to_string(),
        "内存模块详情".to_string(),
    );
    translations.insert("memory.module".to_string(), "内存模块".to_string());
    translations.insert("memory.type".to_string(), "类型".to_string());
    translations.insert("memory.speed".to_string(), "速度".to_string());
    translations.insert("memory.size".to_string(), "大小".to_string());
    translations.insert("memory.bank_label".to_string(), "插槽标签".to_string());
    translations.insert("memory.manufacturer".to_string(), "制造商".to_string());
    translations.insert("memory.serial_number".to_string(), "序列号".to_string());
    translations.insert("memory.part_number".to_string(), "部件号".to_string());
    translations.insert("memory.ecc".to_string(), "ECC".to_string());
    translations.insert("memory.voltage".to_string(), "电压".to_string());
    translations.insert("memory.frequency".to_string(), "频率".to_string());
    translations.insert("memory.bandwidth".to_string(), "带宽".to_string());
    translations.insert("memory.channels".to_string(), "通道数".to_string());

    // 未知值
    translations.insert("no_discrete_gpu".to_string(), "无独立显卡".to_string());
    translations.insert("unknown_system".to_string(), "未知系统".to_string());
    translations.insert("unknown_model".to_string(), "未知型号".to_string());
    translations.insert("unknown_vendor".to_string(), "未知厂商".to_string());
    translations.insert("unknown_version".to_string(), "未知版本".to_string());
    translations.insert("unknown_kernel".to_string(), "未知内核".to_string());
    translations.insert("unknown_architecture".to_string(), "未知架构".to_string());
    translations.insert("no_driver".to_string(), "无驱动".to_string());
    translations.insert("no_storage_devices".to_string(), "无存储设备".to_string());

    // 存储类型
    translations.insert(
        "nvme_ssd_hdd_mixed".to_string(),
        "NVMe+SSD+HDD混合".to_string(),
    );
    translations.insert("nvme_ssd_mixed".to_string(), "NVMe+SSD混合".to_string());
    translations.insert("nvme_hdd_mixed".to_string(), "NVMe+HDD混合".to_string());
    translations.insert("ssd_hdd_mixed".to_string(), "SSD+HDD混合".to_string());
    translations.insert("pure_nvme".to_string(), "纯NVMe".to_string());
    translations.insert("pure_ssd".to_string(), "纯SSD".to_string());
    translations.insert("pure_hdd".to_string(), "纯HDD".to_string());
    translations.insert(
        "unknown_storage_type".to_string(),
        "未知存储类型".to_string(),
    );

    // API 成功消息
    translations.insert(
        "clients_filtered_successfully".to_string(),
        "客户端筛选成功".to_string(),
    );
    translations.insert(
        "filter_options_retrieved_successfully".to_string(),
        "筛选选项获取成功".to_string(),
    );
    translations.insert(
        "client_registered_successfully".to_string(),
        "客户端注册成功".to_string(),
    );
    translations.insert(
        "client_updated_successfully".to_string(),
        "客户端更新成功".to_string(),
    );
    translations.insert(
        "client_deleted_successfully".to_string(),
        "客户端删除成功".to_string(),
    );
    translations.insert(
        "clients_listed_successfully".to_string(),
        "客户端列表获取成功".to_string(),
    );
    translations.insert(
        "client_retrieved_successfully".to_string(),
        "客户端信息获取成功".to_string(),
    );
    translations.insert(
        "hardware_retrieved_successfully".to_string(),
        "硬件信息获取成功".to_string(),
    );
    translations.insert(
        "stats_retrieved_successfully".to_string(),
        "统计信息获取成功".to_string(),
    );

    // API 错误消息
    translations.insert(
        "empty_client_ids_provided".to_string(),
        "提供的客户端ID为空".to_string(),
    );
    translations.insert(
        "no_valid_client_ids_provided".to_string(),
        "未提供有效的客户端ID".to_string(),
    );
    translations.insert(
        "no_clients_found_with_provided_ids".to_string(),
        "未找到指定ID的客户端".to_string(),
    );

    // 错误码翻译
    translations.insert(
        "internal_server_error".to_string(),
        "内部服务器错误".to_string(),
    );
    translations.insert("invalid_request".to_string(), "无效请求".to_string());
    translations.insert("validation_error".to_string(), "验证错误".to_string());
    translations.insert("not_found".to_string(), "未找到".to_string());
    translations.insert("client_not_found".to_string(), "客户端未找到".to_string());
    translations.insert(
        "client_already_exists".to_string(),
        "客户端已存在".to_string(),
    );
    translations.insert(
        "client_registration_failed".to_string(),
        "客户端注册失败".to_string(),
    );
    translations.insert(
        "client_update_failed".to_string(),
        "客户端更新失败".to_string(),
    );
    translations.insert(
        "client_delete_failed".to_string(),
        "客户端删除失败".to_string(),
    );
    translations.insert(
        "hardware_not_found".to_string(),
        "硬件信息未找到".to_string(),
    );
    translations.insert(
        "hardware_data_invalid".to_string(),
        "硬件数据无效".to_string(),
    );
    translations.insert(
        "hardware_collection_failed".to_string(),
        "硬件信息收集失败".to_string(),
    );
    translations.insert(
        "filter_options_error".to_string(),
        "筛选选项错误".to_string(),
    );
    translations.insert(
        "filter_query_invalid".to_string(),
        "筛选查询无效".to_string(),
    );
    translations.insert(
        "filter_execution_failed".to_string(),
        "筛选执行失败".to_string(),
    );
    translations.insert(
        "database_connection_error".to_string(),
        "数据库连接错误".to_string(),
    );
    translations.insert(
        "database_query_error".to_string(),
        "数据库查询错误".to_string(),
    );
    translations.insert(
        "database_transaction_error".to_string(),
        "数据库事务错误".to_string(),
    );
    translations.insert("network_error".to_string(), "网络错误".to_string());
    translations.insert("connection_timeout".to_string(), "连接超时".to_string());
    translations.insert("request_timeout".to_string(), "请求超时".to_string());

    // UI 文本
    translations.insert(
        "search_placeholder".to_string(),
        "搜索客户端...".to_string(),
    );
    translations.insert("filter_by_os".to_string(), "按操作系统筛选".to_string());
    translations.insert("filter_by_vendor".to_string(), "按厂商筛选".to_string());
    translations.insert("filter_by_model".to_string(), "按型号筛选".to_string());
    translations.insert("clear_filters".to_string(), "清除筛选".to_string());
    translations.insert("apply_filters".to_string(), "应用筛选".to_string());
    translations.insert("total_clients".to_string(), "总客户端数".to_string());
    translations.insert("online_clients".to_string(), "在线客户端".to_string());
    translations.insert("offline_clients".to_string(), "离线客户端".to_string());
    translations.insert("loading".to_string(), "加载中...".to_string());

    // Menu
    translations.insert("menu.dashboard".to_string(), "仪表盘".to_string());
    translations.insert("menu.assets".to_string(), "资产".to_string());
    translations.insert("menu.clients".to_string(), "客户端".to_string());
    translations.insert("menu.racks".to_string(), "机架".to_string());
    translations.insert("racks.list_view".to_string(), "列表视图".to_string());
    translations.insert("racks.rack_view".to_string(), "机架视图".to_string());
    translations.insert("racks.grid_layout".to_string(), "网格布局".to_string());
    translations.insert(
        "racks.single_column_layout".to_string(),
        "单列布局".to_string(),
    );
    translations.insert("racks.capacity_status".to_string(), "容量状态".to_string());
    translations.insert("racks.power_status".to_string(), "电力状态".to_string());
    translations.insert("racks.remaining".to_string(), "剩余".to_string());
    translations.insert(
        "racks.used_no_limit".to_string(),
        "已用: {val} W (无限制)".to_string(),
    );
    translations.insert("racks.confirm_delete".to_string(), "确认删除".to_string());
    translations.insert(
        "racks.confirm_delete_msg".to_string(),
        "确定要删除这个机架吗？此操作不可恢复。".to_string(),
    );
    translations.insert("racks.rack_name".to_string(), "机架名称".to_string());
    translations.insert("racks.location".to_string(), "位置".to_string());
    translations.insert("racks.height_u".to_string(), "高度 (U)".to_string());
    translations.insert(
        "racks.power_limit_w".to_string(),
        "电力限制 (W)".to_string(),
    );
    translations.insert("racks.description".to_string(), "描述".to_string());
    translations.insert("racks.cancel".to_string(), "取消".to_string());
    translations.insert("racks.save".to_string(), "保存".to_string());
    translations.insert("racks.edit_rack".to_string(), "编辑机架".to_string());
    translations.insert("racks.add_rack".to_string(), "添加机架".to_string());
    translations.insert("racks.rack_capacity".to_string(), "机架容量".to_string());
    translations.insert("racks.used".to_string(), "已用".to_string());
    translations.insert("racks.free".to_string(), "空闲".to_string());
    translations.insert("racks.power_usage".to_string(), "电力使用".to_string());
    translations.insert("racks.total_units".to_string(), "总单元数".to_string());
    translations.insert("racks.power_limit".to_string(), "电力限制".to_string());
    translations.insert("racks.devices".to_string(), "设备数".to_string());
    translations.insert("racks.status".to_string(), "状态".to_string());
    translations.insert("racks.status.active".to_string(), "运行中".to_string());
    translations.insert("racks.status.maint".to_string(), "维护中".to_string());
    translations.insert("racks.status.stock".to_string(), "库存中".to_string());
    translations.insert("racks.status.error".to_string(), "异常".to_string());
    translations.insert("racks.delete_success".to_string(), "删除成功".to_string());
    translations.insert("racks.save_success".to_string(), "保存成功".to_string());
    translations.insert(
        "racks.save_failed".to_string(),
        "保存失败: {val}".to_string(),
    );
    translations.insert("racks.actions".to_string(), "操作".to_string());

    translations.insert("menu.components".to_string(), "组件".to_string());
    translations.insert("menu.organization".to_string(), "组织".to_string());
    translations.insert("menu.users".to_string(), "用户".to_string());
    translations.insert("menu.projects".to_string(), "项目".to_string());
    translations.insert("menu.system".to_string(), "系统".to_string());
    translations.insert("menu.analytics".to_string(), "分析".to_string());
    translations.insert("menu.setup_guide".to_string(), "安装指南".to_string());
    translations.insert("menu.base_data".to_string(), "基础数据".to_string());
    translations.insert("menu.accounts".to_string(), "账号管理".to_string());
    translations.insert("menu.permissions".to_string(), "权限管理".to_string());
    translations.insert("menu.exec_policies".to_string(), "执行策略".to_string());
    translations.insert("menu.web_terminal_policies".to_string(), "终端策略".to_string());
    translations.insert("menu.approvals".to_string(), "审批".to_string());
    translations.insert("menu.source_code".to_string(), "源代码".to_string());
    translations.insert("menu.terminal".to_string(), "终端".to_string());
    translations.insert("menu.execution".to_string(), "命令执行".to_string());
    translations.insert("menu.batch_execution".to_string(), "批量执行".to_string());
    translations.insert("menu.execution_history".to_string(), "执行历史".to_string());
    translations.insert("menu.remote_exec_settings".to_string(), "远程执行设置".to_string());
    translations.insert("client_detail.terminal".to_string(), "打开终端".to_string());

    // Terminal / Remote Command Execution
    translations.insert("terminal.title".to_string(), "远程终端".to_string());
    translations.insert("terminal.execute".to_string(), "执行".to_string());
    translations.insert("terminal.history".to_string(), "历史".to_string());
    translations.insert("terminal.history_title".to_string(), "命令历史".to_string());
    translations.insert("terminal.remote_exec".to_string(), "远程执行".to_string());
    translations.insert("terminal.enabled".to_string(), "已启用".to_string());
    translations.insert("terminal.disabled".to_string(), "已禁用".to_string());
    translations.insert("terminal.disabled_msg".to_string(), "远程命令执行已禁用，请由管理员通过上方开关启用。".to_string());
    translations.insert("terminal.select_client".to_string(), "选择客户端...".to_string());
    translations.insert("terminal.client".to_string(), "目标客户端".to_string());
    translations.insert("terminal.command".to_string(), "命令".to_string());
    translations.insert("terminal.command_placeholder".to_string(), "例如: df -h".to_string());
    translations.insert("terminal.timeout_secs".to_string(), "超时（秒）".to_string());
    translations.insert("terminal.run".to_string(), "执行命令".to_string());
    translations.insert("terminal.submitting".to_string(), "提交中...".to_string());
    translations.insert("terminal.col_client".to_string(), "客户端".to_string());
    translations.insert("terminal.col_command".to_string(), "命令".to_string());
    translations.insert("terminal.col_status".to_string(), "状态".to_string());
    translations.insert("terminal.col_danger".to_string(), "危险级别".to_string());
    translations.insert("terminal.col_submitted_by".to_string(), "提交人".to_string());
    translations.insert("terminal.col_created_at".to_string(), "创建时间".to_string());

    // Header
    translations.insert(
        "header.search_placeholder".to_string(),
        "搜索主机名, IP...".to_string(),
    );
    translations.insert("header.change_password".to_string(), "修改密码".to_string());
    translations.insert("header.logout".to_string(), "退出登录".to_string());
    translations.insert("header.switch_language".to_string(), "切换语言".to_string());

    // Auth
    translations.insert("auth.login_title".to_string(), "登录 CMDB".to_string());
    translations.insert("auth.username".to_string(), "用户名".to_string());
    translations.insert("auth.password".to_string(), "密码".to_string());
    translations.insert("auth.login_button".to_string(), "登录".to_string());
    translations.insert("auth.logging_in".to_string(), "登录中...".to_string());

    // Change Password
    translations.insert("password.change_title".to_string(), "修改密码".to_string());
    translations.insert("password.current".to_string(), "当前密码".to_string());
    translations.insert("password.new".to_string(), "新密码".to_string());
    translations.insert("password.confirm".to_string(), "确认新密码".to_string());
    translations.insert("password.submit".to_string(), "修改密码".to_string());
    translations.insert("password.submitting".to_string(), "提交中...".to_string());
    translations.insert(
        "password.success".to_string(),
        "密码修改成功，正在跳转登录页...".to_string(),
    );
    translations.insert(
        "password.mismatch".to_string(),
        "两次输入的密码不一致".to_string(),
    );
    translations.insert(
        "password.too_short".to_string(),
        "密码长度至少为6位".to_string(),
    );

    // Dashboard
    translations.insert(
        "dashboard.loading".to_string(),
        "正在加载仪表盘数据...".to_string(),
    );
    translations.insert(
        "dashboard.total_clients".to_string(),
        "总客户端".to_string(),
    );
    translations.insert(
        "dashboard.registered_nodes".to_string(),
        "已注册节点".to_string(),
    );
    translations.insert("dashboard.online_rate".to_string(), "在线率".to_string());
    translations.insert("dashboard.online".to_string(), "在线".to_string());
    translations.insert("dashboard.new_today".to_string(), "今日新增".to_string());
    translations.insert(
        "dashboard.24h_registered".to_string(),
        "24 小时注册".to_string(),
    );
    translations.insert("dashboard.system_types".to_string(), "系统类型".to_string());
    translations.insert(
        "dashboard.diverse_os".to_string(),
        "多样化操作系统".to_string(),
    );

    // Dashboard Sub-components
    translations.insert(
        "dashboard.os_dist_title".to_string(),
        "操作系统分布".to_string(),
    );
    translations.insert(
        "dashboard.os_dist_desc".to_string(),
        "按注册客户端统计".to_string(),
    );
    translations.insert("dashboard.realtime".to_string(), "实时".to_string());
    translations.insert("dashboard.no_data".to_string(), "暂无数据".to_string());
    translations.insert("dashboard.unit_machines".to_string(), " 台".to_string());

    translations.insert(
        "dashboard.system_status_title".to_string(),
        "系统状态概览".to_string(),
    );
    translations.insert(
        "dashboard.realtime_refresh".to_string(),
        "实时刷新".to_string(),
    );
    translations.insert(
        "dashboard.online_clients".to_string(),
        "在线客户端".to_string(),
    );
    translations.insert(
        "dashboard.offline_clients".to_string(),
        "离线客户端".to_string(),
    );
    translations.insert(
        "dashboard.realtime_update".to_string(),
        " 实时更新".to_string(),
    );

    translations.insert(
        "dashboard.recent_active_title".to_string(),
        "近期活跃客户端".to_string(),
    );
    translations.insert(
        "dashboard.recent_active_desc".to_string(),
        "最近 10 次心跳".to_string(),
    );
    translations.insert(
        "dashboard.recent_offline_title".to_string(),
        "近期离线客户端".to_string(),
    );
    translations.insert(
        "dashboard.recent_offline_desc".to_string(),
        "最近心跳后已离线的客户端".to_string(),
    );
    translations.insert(
        "dashboard.no_clients_registered".to_string(),
        "暂无客户端注册".to_string(),
    );
    translations.insert(
        "dashboard.no_recent_offline_clients".to_string(),
        "近期没有离线客户端".to_string(),
    );
    translations.insert("dashboard.offline".to_string(), "离线".to_string());

    translations.insert(
        "dashboard.client_status_list".to_string(),
        "客户端状态列表".to_string(),
    );
    translations.insert("dashboard.total".to_string(), "共".to_string());
    translations.insert(
        "dashboard.managed_nodes".to_string(),
        "台受管节点".to_string(),
    );
    translations.insert("dashboard.view_all".to_string(), "查看全部".to_string());
    translations.insert("dashboard.host".to_string(), "主机".to_string());
    translations.insert("dashboard.system".to_string(), "系统".to_string());
    translations.insert("dashboard.config".to_string(), "配置".to_string());
    translations.insert("dashboard.status".to_string(), "状态".to_string());

    // Clients Page
    translations.insert(
        "clients.stats.total_devices".to_string(),
        "设备总数".to_string(),
    );
    translations.insert(
        "clients.stats.filtered_results".to_string(),
        "筛选结果".to_string(),
    );
    translations.insert(
        "clients.stats.os_types".to_string(),
        "操作系统类型".to_string(),
    );
    translations.insert(
        "clients.stats.vendor_count".to_string(),
        "厂商数量".to_string(),
    );

    translations.insert(
        "clients.search.title".to_string(),
        "高级搜索与筛选".to_string(),
    );
    translations.insert(
        "clients.search.keyword_label".to_string(),
        "关键词搜索".to_string(),
    );
    translations.insert(
        "clients.search.placeholder".to_string(),
        "搜索主机名、IP地址、操作系统、厂商、型号或序列号...".to_string(),
    );
    translations.insert(
        "clients.search.hint".to_string(),
        "支持模糊搜索，停止输入后会自动更新筛选结果".to_string(),
    );
    translations.insert(
        "clients.search.export_csv".to_string(),
        "导出 CSV".to_string(),
    );
    translations.insert(
        "clients.search.export_json".to_string(),
        "导出 JSON".to_string(),
    );
    translations.insert("clients.search.import".to_string(), "导入数据".to_string());
    translations.insert("clients.search.apply".to_string(), "应用筛选".to_string());
    translations.insert("clients.search.clear".to_string(), "清除筛选".to_string());
    translations.insert(
        "clients.filter.active_filters".to_string(),
        "当前筛选条件".to_string(),
    );

    translations.insert("clients.filter.status".to_string(), "状态".to_string());
    translations.insert("clients.filter.environment".to_string(), "环境".to_string());
    translations.insert("clients.filter.rack".to_string(), "机柜".to_string());
    translations.insert("clients.filter.project".to_string(), "项目".to_string());
    translations.insert("clients.filter.owner".to_string(), "负责人".to_string());
    translations.insert("clients.filter.os".to_string(), "操作系统".to_string());
    translations.insert("clients.filter.kernel".to_string(), "内核版本".to_string());
    translations.insert(
        "clients.filter.vendor".to_string(),
        "服务器厂商".to_string(),
    );
    translations.insert(
        "clients.filter.cpu_vendor".to_string(),
        "CPU厂商".to_string(),
    );
    translations.insert(
        "clients.filter.cpu_model".to_string(),
        "CPU型号".to_string(),
    );
    translations.insert(
        "clients.filter.gpu_vendor".to_string(),
        "GPU厂商".to_string(),
    );
    translations.insert(
        "clients.filter.gpu_model".to_string(),
        "GPU型号".to_string(),
    );
    translations.insert(
        "clients.filter.memory_min".to_string(),
        "最小内存(GB)".to_string(),
    );
    translations.insert(
        "clients.filter.memory_max".to_string(),
        "最大内存(GB)".to_string(),
    );
    translations.insert(
        "clients.filter.network_type".to_string(),
        "网卡类型".to_string(),
    );
    translations.insert(
        "clients.filter.network_model".to_string(),
        "网卡型号".to_string(),
    );
    translations.insert(
        "clients.filter.storage_type".to_string(),
        "存储类型".to_string(),
    );

    translations.insert(
        "clients.table.no_data".to_string(),
        "暂无设备数据".to_string(),
    );
    translations.insert("clients.table.hostname".to_string(), "主机名".to_string());
    translations.insert(
        "clients.table.online_status".to_string(),
        "在线状态".to_string(),
    );
    translations.insert("clients.table.ip".to_string(), "IP地址".to_string());
    translations.insert("clients.table.os".to_string(), "操作系统".to_string());
    translations.insert("clients.table.owner".to_string(), "负责人".to_string());
    translations.insert("clients.table.project".to_string(), "项目".to_string());
    translations.insert("clients.table.status".to_string(), "状态".to_string());
    translations.insert("clients.table.environment".to_string(), "环境".to_string());
    translations.insert("clients.table.actions".to_string(), "操作".to_string());

    translations.insert("clients.status.active".to_string(), "运行中".to_string());
    translations.insert(
        "clients.status.maintenance".to_string(),
        "维护中".to_string(),
    );
    translations.insert("clients.status.instock".to_string(), "库存中".to_string());
    translations.insert(
        "clients.status.decommissioned".to_string(),
        "已下架".to_string(),
    );

    translations.insert("clients.env.prod".to_string(), "生产".to_string());
    translations.insert("clients.env.staging".to_string(), "预发布".to_string());
    translations.insert("clients.env.test".to_string(), "测试".to_string());
    translations.insert("clients.env.dev".to_string(), "开发".to_string());

    translations.insert("clients.actions.view".to_string(), "查看详情".to_string());
    translations.insert("clients.actions.edit".to_string(), "编辑".to_string());
    translations.insert("clients.actions.delete".to_string(), "删除".to_string());
    translations.insert(
        "clients.actions.confirm_delete".to_string(),
        "确定要删除该设备吗？注意：如果客户端 Agent 仍在运行，它会自动重新注册。请先停止客户端服务。".to_string(),
    );

    translations.insert(
        "clients.import.success".to_string(),
        "导入成功！".to_string(),
    );
    translations.insert(
        "clients.import.error_title".to_string(),
        "导入失败".to_string(),
    );
    translations.insert(
        "clients.import.error_desc".to_string(),
        "发现以下错误，导入已取消。请修正后重试。".to_string(),
    );
    translations.insert(
        "clients.import.progress".to_string(),
        "正在导入...".to_string(),
    );
    translations.insert(
        "clients.selection.selected".to_string(),
        "已选择".to_string(),
    );
    translations.insert(
        "clients.selection.export_template".to_string(),
        "导出模板".to_string(),
    );

    translations.insert("common.close".to_string(), "关闭".to_string());
    translations.insert("common.error_prefix".to_string(), "错误：".to_string());

    translations.insert("error".to_string(), "错误".to_string());
    translations.insert("success".to_string(), "成功".to_string());
    translations.insert("warning".to_string(), "警告".to_string());
    translations.insert("info".to_string(), "信息".to_string());

    // Notification
    translations.insert("notification.success".to_string(), "成功".to_string());
    translations.insert("notification.info".to_string(), "信息".to_string());
    translations.insert("notification.warning".to_string(), "警告".to_string());
    translations.insert("notification.error".to_string(), "错误".to_string());

    // 表格列标题
    translations.insert("hostname".to_string(), "主机名".to_string());
    translations.insert("ip_address".to_string(), "IP地址".to_string());
    translations.insert("os".to_string(), "操作系统".to_string());
    translations.insert("vendor".to_string(), "厂商".to_string());
    translations.insert("model".to_string(), "型号".to_string());
    translations.insert("last_seen".to_string(), "最后在线".to_string());
    translations.insert("status".to_string(), "状态".to_string());
    translations.insert("actions".to_string(), "操作".to_string());

    // 分页
    translations.insert("previous_page".to_string(), "上一页".to_string());
    translations.insert("next_page".to_string(), "下一页".to_string());
    translations.insert("page".to_string(), "第".to_string());
    translations.insert("of".to_string(), "页，共".to_string());
    translations.insert("items_per_page".to_string(), "条/页".to_string());

    // Common
    translations.insert("common.cancel".to_string(), "取消".to_string());
    translations.insert("common.save".to_string(), "保存".to_string());
    translations.insert("common.delete_success".to_string(), "删除成功".to_string());
    translations.insert("common.save_success".to_string(), "保存成功".to_string());
    translations.insert("common.save_failed".to_string(), "保存失败: {}".to_string());
    translations.insert("common.actions".to_string(), "操作".to_string());
    translations.insert("common.confirm_delete".to_string(), "确认删除".to_string());
    translations.insert("common.delete".to_string(), "删除".to_string());
    translations.insert("common.export".to_string(), "导出".to_string());
    translations.insert("common.reset".to_string(), "重置".to_string());

    // Projects
    translations.insert(
        "projects.select_cost_center".to_string(),
        "请选择成本中心".to_string(),
    );
    translations.insert(
        "projects.select_manager".to_string(),
        "请选择负责人".to_string(),
    );
    translations.insert("projects.edit_project".to_string(), "编辑项目".to_string());
    translations.insert("projects.add_project".to_string(), "添加项目".to_string());
    translations.insert("projects.name".to_string(), "项目名称".to_string());
    translations.insert("projects.code".to_string(), "项目代码".to_string());
    translations.insert("projects.department".to_string(), "所属部门".to_string());
    translations.insert("projects.cost_center".to_string(), "成本中心".to_string());
    translations.insert(
        "projects.manager".to_string(),
        "负责人 (项目经理)".to_string(),
    );
    translations.insert("projects.new_project".to_string(), "新建项目".to_string());
    translations.insert(
        "projects.confirm_delete_msg".to_string(),
        "确定要删除这个项目吗？此操作不可恢复。".to_string(),
    );

    // Persons
    translations.insert(
        "persons.select_department".to_string(),
        "请选择部门".to_string(),
    );
    translations.insert("persons.select_title".to_string(), "请选择职位".to_string());
    translations.insert("persons.edit_person".to_string(), "编辑用户".to_string());
    translations.insert("persons.add_person".to_string(), "添加用户".to_string());
    translations.insert("persons.name".to_string(), "姓名".to_string());
    translations.insert("persons.email".to_string(), "邮箱".to_string());
    translations.insert("persons.department".to_string(), "部门".to_string());
    translations.insert("persons.phone".to_string(), "电话".to_string());
    translations.insert("persons.title".to_string(), "职位".to_string());
    translations.insert("persons.delete_success".to_string(), "删除成功".to_string());
    translations.insert("persons.save_success".to_string(), "保存成功".to_string());
    translations.insert(
        "persons.save_failed".to_string(),
        "保存失败: {}".to_string(),
    );
    translations.insert("persons.new_person".to_string(), "新建用户".to_string());
    translations.insert("persons.actions".to_string(), "操作".to_string());
    translations.insert("persons.confirm_delete".to_string(), "确认删除".to_string());
    translations.insert(
        "persons.confirm_delete_msg".to_string(),
        "确定要删除这个用户吗？此操作不可恢复。".to_string(),
    );

    // Components
    translations.insert("components.type_other".to_string(), "其他".to_string());
    translations.insert("components.type_gpu".to_string(), "GPU".to_string());
    translations.insert("components.type_cpu".to_string(), "CPU".to_string());
    translations.insert("components.type_memory".to_string(), "内存".to_string());
    translations.insert("components.type_disk".to_string(), "硬盘".to_string());
    translations.insert(
        "components.type_network_card".to_string(),
        "网卡".to_string(),
    );
    translations.insert(
        "components.type_motherboard".to_string(),
        "主板".to_string(),
    );
    translations.insert(
        "components.type_power_supply".to_string(),
        "电源".to_string(),
    );
    translations.insert(
        "components.status_in_stock".to_string(),
        "库存中".to_string(),
    );
    translations.insert("components.status_in_use".to_string(), "使用中".to_string());
    translations.insert("components.status_lent_out".to_string(), "借出".to_string());
    translations.insert("components.status_faulty".to_string(), "故障".to_string());
    translations.insert(
        "components.status_decommissioned".to_string(),
        "已报废".to_string(),
    );
    translations.insert("components.status_unknown".to_string(), "未知".to_string());
    translations.insert(
        "components.new_component".to_string(),
        "新建组件".to_string(),
    );
    translations.insert(
        "components.edit_component".to_string(),
        "编辑组件: {model} ({sn})".to_string(),
    );
    translations.insert(
        "components.serial_number".to_string(),
        "序列号 (SN)".to_string(),
    );
    translations.insert("components.model".to_string(), "型号".to_string());
    translations.insert("components.type".to_string(), "类型".to_string());
    translations.insert("components.vendor".to_string(), "厂商".to_string());
    translations.insert("components.status".to_string(), "状态".to_string());
    translations.insert(
        "components.location".to_string(),
        "位置 (库存位置)".to_string(),
    );
    translations.insert(
        "components.purchase_date".to_string(),
        "购买日期".to_string(),
    );
    translations.insert(
        "components.warranty_expiration".to_string(),
        "维保到期".to_string(),
    );
    translations.insert(
        "components.batch_create_json".to_string(),
        "批量新建组件 (JSON)".to_string(),
    );
    translations.insert(
        "components.json_parse_error".to_string(),
        "JSON 解析错误: {error}".to_string(),
    );
    translations.insert(
        "components.json_input_hint".to_string(),
        "请输入包含组件信息的 JSON 数组。示例:".to_string(),
    );
    translations.insert(
        "components.batch_create".to_string(),
        "批量创建".to_string(),
    );
    translations.insert(
        "components.select_components_first".to_string(),
        "请先选择组件".to_string(),
    );
    translations.insert(
        "components.confirm_batch_status_update".to_string(),
        "确定要将选中的 {count} 个组件状态修改为 {status} 吗？".to_string(),
    );
    translations.insert(
        "components.batch_status_update_success".to_string(),
        "批量修改状态成功".to_string(),
    );
    translations.insert(
        "components.batch_status_update_failed".to_string(),
        "批量修改状态失败: {error}".to_string(),
    );
    translations.insert(
        "components.batch_edit_export".to_string(),
        "批量编辑(导出)".to_string(),
    );
    translations.insert(
        "components.json_import".to_string(),
        "JSON 导入".to_string(),
    );
    translations.insert(
        "components.excel_import".to_string(),
        "Excel 导入".to_string(),
    );
    translations.insert(
        "components.quick_status_change".to_string(),
        "快速修改状态...".to_string(),
    );
    translations.insert("components.search".to_string(), "查询".to_string());
    translations.insert("components.search_label".to_string(), "搜索".to_string());
    translations.insert(
        "components.search_placeholder".to_string(),
        "搜索序列号(SN)或型号...".to_string(),
    );
    translations.insert(
        "components.component_info".to_string(),
        "组件信息".to_string(),
    );
    translations.insert(
        "components.type_vendor".to_string(),
        "类型/厂商".to_string(),
    );
    translations.insert(
        "components.location_owner".to_string(),
        "位置/归属".to_string(),
    );
    translations.insert("components.actions".to_string(), "操作".to_string());
    translations.insert(
        "components.server_prefix".to_string(),
        "服务器: ".to_string(),
    );
    translations.insert(
        "components.importing".to_string(),
        "正在导入...".to_string(),
    );
    translations.insert(
        "components.batch_update_complete".to_string(),
        "批量更新完成: {success}/{total} 成功".to_string(),
    );
    translations.insert(
        "components.batch_create_success".to_string(),
        "成功创建 {count} 个组件".to_string(),
    );
    translations.insert(
        "components.batch_create_failed".to_string(),
        "批量创建失败: {error}".to_string(),
    );

    // Analytics
    translations.insert(
        "analytics.total_devices".to_string(),
        "总设备数".to_string(),
    );
    translations.insert(
        "analytics.online_devices".to_string(),
        "在线设备".to_string(),
    );
    translations.insert(
        "analytics.offline_devices".to_string(),
        "离线设备".to_string(),
    );
    translations.insert("analytics.online_rate".to_string(), "在线率".to_string());
    translations.insert("analytics.retry".to_string(), "重试".to_string());
    translations.insert(
        "analytics.gpu_vendor_distribution".to_string(),
        "GPU厂商分布".to_string(),
    );
    translations.insert(
        "analytics.by_machine_count".to_string(),
        "按机器数量".to_string(),
    );
    translations.insert(
        "analytics.detailed_stats".to_string(),
        "详细统计".to_string(),
    );
    translations.insert("analytics.gpu_vendor".to_string(), "GPU厂商".to_string());
    translations.insert("analytics.count".to_string(), "数量".to_string());
    translations.insert("analytics.percentage".to_string(), "占比".to_string());
    translations.insert(
        "analytics.unit_machine".to_string(),
        "{count}台".to_string(),
    );
    translations.insert(
        "analytics.gpu_model_distribution".to_string(),
        "GPU型号分布".to_string(),
    );
    translations.insert("analytics.gpu_model".to_string(), "GPU型号".to_string());
    translations.insert(
        "analytics.gpu_detailed_config".to_string(),
        "GPU详细配置".to_string(),
    );
    translations.insert(
        "analytics.by_model_and_count".to_string(),
        "按型号和数量".to_string(),
    );
    translations.insert("analytics.gpu_config".to_string(), "GPU配置".to_string());
    translations.insert(
        "analytics.cpu_model_distribution".to_string(),
        "CPU型号分布".to_string(),
    );
    translations.insert("analytics.cpu_model".to_string(), "CPU型号".to_string());
    translations.insert(
        "analytics.storage_type_distribution".to_string(),
        "存储类型分布".to_string(),
    );
    translations.insert("analytics.storage_type".to_string(), "存储类型".to_string());
    translations.insert(
        "analytics.os_distribution".to_string(),
        "操作系统分布".to_string(),
    );
    translations.insert("analytics.os".to_string(), "操作系统".to_string());
    translations.insert(
        "analytics.memory_size_distribution".to_string(),
        "内存大小分布".to_string(),
    );
    translations.insert("analytics.memory_size".to_string(), "内存大小".to_string());
    translations.insert(
        "analytics.network_type_distribution".to_string(),
        "网络类型分布".to_string(),
    );
    translations.insert("analytics.network_type".to_string(), "网络类型".to_string());
    translations.insert(
        "analytics.server_model_distribution".to_string(),
        "服务器型号分布".to_string(),
    );
    translations.insert(
        "analytics.server_model".to_string(),
        "服务器型号".to_string(),
    );

    // Stats Filter
    translations.insert("stats.filter.cpu_vendor".to_string(), "CPU厂商".to_string());
    translations.insert("stats.filter.gpu_vendor".to_string(), "GPU厂商".to_string());
    translations.insert(
        "stats.filter.memory_capacity".to_string(),
        "内存容量 (GB)".to_string(),
    );
    translations.insert("stats.filter.os".to_string(), "操作系统".to_string());

    // Client Setup
    translations.insert(
        "client_setup.parse_error".to_string(),
        "解析响应失败: {error}".to_string(),
    );
    translations.insert(
        "client_setup.request_failed".to_string(),
        "请求失败: {status}".to_string(),
    );
    translations.insert(
        "client_setup.network_error".to_string(),
        "网络请求失败: {error}".to_string(),
    );
    translations.insert(
        "client_setup.load_failed".to_string(),
        "无法加载客户端信息".to_string(),
    );
    translations.insert(
        "client_setup.guide_title".to_string(),
        "客户端安装指南".to_string(),
    );
    translations.insert(
        "client_setup.guide_subtitle".to_string(),
        "选择平台和架构以获取安装说明".to_string(),
    );
    translations.insert(
        "client_setup.select_platform".to_string(),
        "选择平台".to_string(),
    );
    translations.insert(
        "client_setup.select_arch".to_string(),
        "选择架构".to_string(),
    );
    translations.insert(
        "client_setup.step1_download".to_string(),
        "1. 下载客户端".to_string(),
    );
    translations.insert(
        "client_setup.download_url".to_string(),
        "下载地址: ".to_string(),
    );
    translations.insert(
        "client_setup.server_url".to_string(),
        "服务器地址: ".to_string(),
    );
    translations.insert(
        "client_setup.step2_quick_install".to_string(),
        "2. 快速安装（推荐）".to_string(),
    );
    translations.insert(
        "client_setup.copy_command".to_string(),
        "复制以下命令到终端执行：".to_string(),
    );
    translations.insert(
        "client_setup.quick_install_desc".to_string(),
        "此命令将自动下载、安装并启动客户端服务".to_string(),
    );
    translations.insert(
        "client_setup.step3_manual_install".to_string(),
        "3. 手动安装".to_string(),
    );
    translations.insert(
        "client_setup.step2_install_script".to_string(),
        "2. 安装脚本".to_string(),
    );
    translations.insert(
        "client_setup.save_script".to_string(),
        "保存以下脚本为文件并执行：".to_string(),
    );
    translations.insert(
        "client_setup.step4_config".to_string(),
        "4. 配置文件".to_string(),
    );
    translations.insert(
        "client_setup.step3_config".to_string(),
        "3. 配置文件".to_string(),
    );
    translations.insert(
        "client_setup.config_template_desc".to_string(),
        "配置文件模板（默认路径：/etc/rs-cmdb/client.toml）：".to_string(),
    );
    translations.insert(
        "client_setup.step5_systemd".to_string(),
        "5. Systemd 服务配置".to_string(),
    );
    translations.insert(
        "client_setup.systemd_desc".to_string(),
        "Systemd 服务文件（/etc/systemd/system/rs-cmdb-client.service）：".to_string(),
    );
    translations.insert(
        "client_setup.step6_verify".to_string(),
        "6. 验证安装".to_string(),
    );
    translations.insert(
        "client_setup.step4_verify".to_string(),
        "4. 验证安装".to_string(),
    );
    translations.insert(
        "client_setup.check_status".to_string(),
        "检查服务状态".to_string(),
    );
    translations.insert(
        "client_setup.manual_run_check".to_string(),
        "手动运行客户端，检查是否连接到服务器".to_string(),
    );
    translations.insert(
        "client_setup.check_logs".to_string(),
        "查看日志".to_string(),
    );
    translations.insert(
        "client_setup.check_logs_dir".to_string(),
        "检查应用程序日志目录".to_string(),
    );
    translations.insert(
        "client_setup.install_complete_prefix".to_string(),
        "安装完成后，客户端会自动出现在 ".to_string(),
    );
    translations.insert(
        "client_setup.client_list".to_string(),
        "客户端列表".to_string(),
    );
    translations.insert(
        "client_setup.install_complete_suffix".to_string(),
        " 中。".to_string(),
    );

    // Dictionaries
    translations.insert("dictionaries.department".to_string(), "部门".to_string());
    translations.insert("dictionaries.title".to_string(), "职位".to_string());
    translations.insert(
        "dictionaries.cost_center".to_string(),
        "成本中心".to_string(),
    );
    translations.insert(
        "dictionaries.dictionary_item".to_string(),
        "字典项".to_string(),
    );
    translations.insert("dictionaries.create_prefix".to_string(), "新建".to_string());
    translations.insert("dictionaries.edit_prefix".to_string(), "编辑".to_string());
    translations.insert(
        "dictionaries.key_label".to_string(),
        "键 (Key):".to_string(),
    );
    translations.insert(
        "dictionaries.key_desc".to_string(),
        " 系统内部使用的唯一标识符，通常使用英文或编码 (例如: 'HR', 'DEV_01')。".to_string(),
    );
    translations.insert(
        "dictionaries.value_label".to_string(),
        "值 (Value):".to_string(),
    );
    translations.insert(
        "dictionaries.value_desc".to_string(),
        " 显示给用户的名称 (例如: '人力资源部', '开发一组')。".to_string(),
    );
    translations.insert("dictionaries.key".to_string(), "键 (Key)".to_string());
    translations.insert(
        "dictionaries.key_placeholder".to_string(),
        "例如: HR".to_string(),
    );
    translations.insert("dictionaries.value".to_string(), "值 (Value)".to_string());
    translations.insert(
        "dictionaries.value_placeholder".to_string(),
        "例如: 人力资源部".to_string(),
    );
    translations.insert("dictionaries.description".to_string(), "描述".to_string());
    translations.insert(
        "dictionaries.description_placeholder".to_string(),
        "可选的描述信息".to_string(),
    );
    translations.insert("dictionaries.cancel".to_string(), "取消".to_string());
    translations.insert("dictionaries.save".to_string(), "保存".to_string());
    translations.insert(
        "dictionaries.delete_success".to_string(),
        "删除成功".to_string(),
    );
    translations.insert(
        "dictionaries.save_success".to_string(),
        "保存成功".to_string(),
    );
    translations.insert(
        "dictionaries.save_failed".to_string(),
        "保存失败: {error}".to_string(),
    );
    translations.insert(
        "dictionaries.create_department".to_string(),
        "新建部门".to_string(),
    );
    translations.insert(
        "dictionaries.create_title".to_string(),
        "新建职位".to_string(),
    );
    translations.insert(
        "dictionaries.create_cost_center".to_string(),
        "新建成本中心".to_string(),
    );
    translations.insert("dictionaries.create".to_string(), "新建".to_string());
    translations.insert(
        "dictionaries.create_item".to_string(),
        "新建项目".to_string(),
    );
    translations.insert("dictionaries.actions".to_string(), "操作".to_string());
    translations.insert(
        "dictionaries.confirm_delete_title".to_string(),
        "确认删除".to_string(),
    );
    translations.insert(
        "dictionaries.confirm_delete_message".to_string(),
        "确定要删除这个字典项吗？此操作不可恢复。".to_string(),
    );

    // Users
    translations.insert(
        "users.update_success".to_string(),
        "用户更新成功".to_string(),
    );
    translations.insert(
        "users.create_success".to_string(),
        "用户创建成功".to_string(),
    );
    translations.insert(
        "users.delete_confirm".to_string(),
        "确定要删除这个用户吗？".to_string(),
    );
    translations.insert(
        "users.delete_success".to_string(),
        "用户删除成功".to_string(),
    );
    translations.insert("users.create_user".to_string(), "新建用户".to_string());
    translations.insert("users.username".to_string(), "用户名".to_string());
    translations.insert("users.role".to_string(), "角色".to_string());
    translations.insert("users.status".to_string(), "状态".to_string());
    translations.insert("users.last_login".to_string(), "最后登录".to_string());
    translations.insert("users.actions".to_string(), "操作".to_string());
    translations.insert("users.active".to_string(), "启用".to_string());
    translations.insert("users.inactive".to_string(), "禁用".to_string());
    translations.insert("users.edit_user".to_string(), "编辑用户".to_string());
    translations.insert(
        "users.password_placeholder_edit".to_string(),
        "密码 (留空保持不变)".to_string(),
    );
    translations.insert("users.password".to_string(), "密码".to_string());
    translations.insert(
        "users.username_placeholder".to_string(),
        "输入用户名".to_string(),
    );
    translations.insert(
        "users.password_placeholder".to_string(),
        "输入密码".to_string(),
    );
    translations.insert("users.enable_account".to_string(), "启用账户".to_string());
    translations.insert("users.cancel".to_string(), "取消".to_string());
    translations.insert("users.save".to_string(), "保存".to_string());
    translations.insert("users.role_viewer".to_string(), "访客 (Viewer)".to_string());
    translations.insert("users.role_user".to_string(), "普通用户 (User)".to_string());
    translations.insert("users.role_admin".to_string(), "管理员 (Admin)".to_string());

    // Pagination
    translations.insert(
        "pagination.total_items".to_string(),
        "共 {count} 条".to_string(),
    );
    translations.insert("pagination.items_per_page".to_string(), "条/页".to_string());
    translations.insert("pagination.per_page".to_string(), "每页显示".to_string());
    translations.insert("pagination.unit".to_string(), "条".to_string());
    translations.insert("pagination.jump_to".to_string(), "跳至".to_string());
    translations.insert("pagination.go".to_string(), "GO".to_string());

    // Client Detail
    translations.insert("client_detail.title".to_string(), "客户端详情".to_string());
    translations.insert(
        "client_detail.basic_info".to_string(),
        "客户端基本信息".to_string(),
    );
    translations.insert("client_detail.edit".to_string(), "编辑".to_string());
    translations.insert("client_detail.refresh".to_string(), "刷新".to_string());
    translations.insert("client_detail.id".to_string(), "ID".to_string());
    translations.insert("client_detail.hostname".to_string(), "主机名".to_string());
    translations.insert("client_detail.ip".to_string(), "IP 地址".to_string());
    translations.insert(
        "client_detail.primary_ip".to_string(),
        "主 IP 地址".to_string(),
    );
    translations.insert("client_detail.os".to_string(), "操作系统".to_string());
    translations.insert("client_detail.kernel".to_string(), "内核版本".to_string());
    translations.insert("client_detail.location".to_string(), "位置".to_string());
    translations.insert("client_detail.rack".to_string(), "机柜".to_string());
    translations.insert(
        "client_detail.unit_position".to_string(),
        "单元位置".to_string(),
    );
    translations.insert("client_detail.u_height".to_string(), "高度 (U)".to_string());
    translations.insert(
        "client_detail.power".to_string(),
        "功率消耗 (W)".to_string(),
    );
    translations.insert("client_detail.owner".to_string(), "负责人".to_string());
    translations.insert("client_detail.project".to_string(), "项目".to_string());
    translations.insert("client_detail.serial".to_string(), "序列号".to_string());
    translations.insert(
        "client_detail.asset_tag".to_string(),
        "资产标签".to_string(),
    );
    translations.insert("client_detail.warranty".to_string(), "维保到期".to_string());
    translations.insert("client_detail.supplier".to_string(), "供应商".to_string());
    translations.insert(
        "client_detail.registered".to_string(),
        "注册时间".to_string(),
    );
    translations.insert(
        "client_detail.last_seen".to_string(),
        "最后在线".to_string(),
    );
    translations.insert("client_detail.status".to_string(), "状态".to_string());
    translations.insert("client_detail.environment".to_string(), "环境".to_string());
    translations.insert("client_detail.comment".to_string(), "备注".to_string());
    translations.insert("client_detail.tab_overview".to_string(), "概览".to_string());
    translations.insert(
        "client_detail.tab_hardware".to_string(),
        "硬件信息".to_string(),
    );
    translations.insert(
        "client_detail.tab_history".to_string(),
        "硬件历史".to_string(),
    );
    translations.insert(
        "client_detail.loading_overview".to_string(),
        "加载概览信息...".to_string(),
    );
    translations.insert(
        "client_detail.loading_hardware".to_string(),
        "加载硬件信息...".to_string(),
    );
    translations.insert(
        "client_detail.loading_client".to_string(),
        "加载客户端信息...".to_string(),
    );
    translations.insert(
        "client_detail.no_hardware".to_string(),
        "无硬件信息".to_string(),
    );
    translations.insert(
        "client_detail.update_success".to_string(),
        "更新成功".to_string(),
    );
    translations.insert(
        "client_detail.update_failed".to_string(),
        "更新失败: {}".to_string(),
    );

    // Client Status
    translations.insert("client_status.active".to_string(), "运行中".to_string());
    translations.insert(
        "client_status.maintenance".to_string(),
        "维护中".to_string(),
    );
    translations.insert("client_status.in_stock".to_string(), "库存中".to_string());
    translations.insert(
        "client_status.decommissioned".to_string(),
        "已下架".to_string(),
    );

    // Environment
    translations.insert("environment.prod".to_string(), "生产".to_string());
    translations.insert("environment.staging".to_string(), "预发布".to_string());
    translations.insert("environment.test".to_string(), "测试".to_string());
    translations.insert("environment.dev".to_string(), "开发".to_string());

    translations.insert(
        "client_edit.title".to_string(),
        "编辑客户端信息".to_string(),
    );
    translations.insert(
        "client_edit.description".to_string(),
        "修改客户端的详细信息，包括位置、关联、状态等。".to_string(),
    );
    translations.insert("client_edit.unassigned".to_string(), "未分配".to_string());
    translations.insert("client_edit.save".to_string(), "保存".to_string());
    translations.insert("client_edit.cancel".to_string(), "取消".to_string());
    translations.insert(
        "client_edit.primary_ip_hint".to_string(),
        "设置后将覆盖自动检测的主 IP 地址".to_string(),
    );
    translations.insert("client_edit.primary_ip_none".to_string(), "无".to_string());
    translations.insert(
        "client_edit.primary_ip_select".to_string(),
        "选择 IP...".to_string(),
    );
    translations.insert(
        "client_edit.primary_ip_custom".to_string(),
        "手动输入...".to_string(),
    );

    translations
}
