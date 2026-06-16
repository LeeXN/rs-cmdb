use std::collections::HashMap;

pub fn get_translations() -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Common validation messages
    translations.insert(
        "validation.required".to_string(),
        "此字段为必填项".to_string(),
    );
    translations.insert(
        "validation.invalid_email".to_string(),
        "邮箱格式无效".to_string(),
    );
    translations.insert(
        "validation.invalid_url".to_string(),
        "URL格式无效".to_string(),
    );
    translations.insert(
        "validation.too_short".to_string(),
        "至少需要{min}个字符".to_string(),
    );
    translations.insert(
        "validation.too_long".to_string(),
        "最多{max}个字符".to_string(),
    );
    translations.insert(
        "validation.invalid_format".to_string(),
        "格式无效".to_string(),
    );
    translations.insert(
        "validation.out_of_range".to_string(),
        "值必须在{min}和{max}之间".to_string(),
    );

    // Common API success messages
    translations.insert("success.created".to_string(), "创建成功".to_string());
    translations.insert("success.updated".to_string(), "更新成功".to_string());
    translations.insert("success.deleted".to_string(), "删除成功".to_string());
    translations.insert(
        "success.operation_complete".to_string(),
        "操作成功完成".to_string(),
    );

    // Common API error messages
    translations.insert("error.unauthorized".to_string(), "未授权".to_string());
    translations.insert("error.forbidden".to_string(), "禁止访问".to_string());
    translations.insert("error.not_found".to_string(), "资源未找到".to_string());
    translations.insert("error.conflict".to_string(), "资源已存在".to_string());
    translations.insert(
        "error.internal_server".to_string(),
        "服务器内部错误".to_string(),
    );
    translations.insert("error.bad_request".to_string(), "无效请求".to_string());
    translations.insert(
        "error.method_not_allowed".to_string(),
        "方法不允许".to_string(),
    );
    translations.insert("error.request_timeout".to_string(), "请求超时".to_string());
    translations.insert(
        "error.too_many_requests".to_string(),
        "请求过多".to_string(),
    );

    // Client-specific messages
    translations.insert("client.not_found".to_string(), "客户端未找到".to_string());
    translations.insert("client.created".to_string(), "客户端注册成功".to_string());
    translations.insert("client.updated".to_string(), "客户端更新成功".to_string());
    translations.insert("client.deleted".to_string(), "客户端删除成功".to_string());
    translations.insert(
        "client.invalid_id".to_string(),
        "无效的客户端ID".to_string(),
    );
    translations.insert(
        "client.duplicate_hostname".to_string(),
        "此主机名的客户端已存在".to_string(),
    );

    // Component-specific messages
    translations.insert("component.not_found".to_string(), "组件未找到".to_string());
    translations.insert("component.created".to_string(), "组件创建成功".to_string());
    translations.insert("component.updated".to_string(), "组件更新成功".to_string());
    translations.insert("component.deleted".to_string(), "组件删除成功".to_string());
    translations.insert(
        "component.invalid_type".to_string(),
        "无效的组件类型".to_string(),
    );

    // Rack-specific messages
    translations.insert("rack.not_found".to_string(), "机架未找到".to_string());
    translations.insert("rack.created".to_string(), "机架创建成功".to_string());
    translations.insert("rack.updated".to_string(), "机架更新成功".to_string());
    translations.insert("rack.deleted".to_string(), "机架删除成功".to_string());
    translations.insert(
        "rack.capacity_exceeded".to_string(),
        "机架容量超出".to_string(),
    );

    // Project-specific messages
    translations.insert("project.not_found".to_string(), "项目未找到".to_string());
    translations.insert("project.created".to_string(), "项目创建成功".to_string());
    translations.insert("project.updated".to_string(), "项目更新成功".to_string());
    translations.insert("project.deleted".to_string(), "项目删除成功".to_string());
    translations.insert(
        "project.invalid_manager".to_string(),
        "指定的负责人无效".to_string(),
    );

    // User-specific messages
    translations.insert("user.not_found".to_string(), "用户未找到".to_string());
    translations.insert("user.created".to_string(), "用户创建成功".to_string());
    translations.insert("user.updated".to_string(), "用户更新成功".to_string());
    translations.insert("user.deleted".to_string(), "用户删除成功".to_string());
    translations.insert(
        "user.invalid_credentials".to_string(),
        "用户名或密码无效".to_string(),
    );
    translations.insert(
        "user.username_exists".to_string(),
        "用户名已存在".to_string(),
    );

    // Authentication messages
    translations.insert("auth.login_success".to_string(), "登录成功".to_string());
    translations.insert("auth.login_failed".to_string(), "登录失败".to_string());
    translations.insert("auth.logout_success".to_string(), "退出成功".to_string());
    translations.insert(
        "auth.token_invalid".to_string(),
        "令牌无效或已过期".to_string(),
    );
    translations.insert("auth.permission_denied".to_string(), "权限拒绝".to_string());

    // Database messages
    translations.insert(
        "db.connection_error".to_string(),
        "数据库连接错误".to_string(),
    );
    translations.insert("db.query_error".to_string(), "数据库查询错误".to_string());
    translations.insert(
        "db.transaction_error".to_string(),
        "数据库事务错误".to_string(),
    );

    translations
}
