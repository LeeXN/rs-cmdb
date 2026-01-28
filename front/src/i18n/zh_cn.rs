use std::collections::HashMap;

pub fn get_translations() -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // 通用
    translations.insert("all".to_string(), "全部".to_string());
    translations.insert("unknown".to_string(), "未知".to_string());
    translations.insert("online".to_string(), "在线".to_string());
    translations.insert("offline".to_string(), "离线".to_string());
    translations.insert("none".to_string(), "无".to_string());
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
    translations.insert("menu.source_code".to_string(), "源代码".to_string());

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
        "dashboard.no_clients_registered".to_string(),
        "暂无客户端注册".to_string(),
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
        "支持模糊搜索，输入关键词即可实时筛选".to_string(),
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
        "确定要删除该设备吗？此操作无法撤销。".to_string(),
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

    translations
}
