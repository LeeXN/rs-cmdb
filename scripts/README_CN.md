# 测试数据脚本

这个目录包含用于生成和导入测试数据的脚本，帮助您快速为rs-cmdb系统创建测试环境。

## 脚本说明

### 1. generate_test_data.py
生成100台机器的测试数据，包含真实的硬件配置信息。

**功能特性：**
- 生成100台机器的完整信息
- 包含多种操作系统（Ubuntu、CentOS、RHEL、Debian、Rocky Linux）
- 包含多个厂商的服务器（Dell、HP、Lenovo、Supermicro、ASUS、Gigabyte）
- 模拟真实的在线/离线状态（80%在线，20%离线）
- 生成合理的主机名、IP地址、序列号等信息

**使用方法：**
```bash
cd scripts
python3 generate_test_data.py
```

**输出：**
- 生成 `test_clients.json` 文件，包含100台机器的JSON数据
- 显示详细的统计信息，包括操作系统分布、厂商分布、在线率等

### 2. import_test_data.py
将生成的测试数据导入到rs-cmdb数据库中。

**功能特性：**
- 通过API接口批量导入客户端数据
- 支持错误处理和重试机制
- 显示导入进度和结果统计
- 自动检测API服务可用性

**使用方法：**
```bash
cd scripts
python3 import_test_data.py
```

**前提条件：**
- 确保rs-cmdb后端服务正在运行（默认端口8080）
- 已经运行过 `generate_test_data.py` 生成测试数据
- 安装了 `requests` 库：`pip install requests`

## 完整使用流程

1. **生成测试数据**
   ```bash
   cd scripts
   python3 generate_test_data.py
   ```

2. **启动后端服务**
   ```bash
   cd server
   cargo run
   ```

3. **导入测试数据**
   ```bash
   cd scripts
   python3 import_test_data.py
   ```

4. **启动前端服务**
   ```bash
   cd front
   trunk serve
   ```

5. **访问系统**
   打开浏览器访问 `http://localhost:8080`

## 生成的数据特点

### 操作系统分布
- Ubuntu 20.04 LTS / 22.04 LTS
- CentOS 7 / 8
- RHEL 8 / 9
- Debian 11 / 12
- Rocky Linux 8 / 9

### 服务器厂商和型号
- **Dell**: PowerEdge R740, R750, R640, R650
- **HP**: ProLiant DL380, DL360, DL385, ML350
- **Lenovo**: ThinkSystem SR650, SR630, SR850, SR950
- **Supermicro**: SuperServer 1029P, 2029P, 6029P, 8029P
- **ASUS**: ESC4000A-E10, ESC8000A-E11, RS720A-E11, RS500A-E10
- **Gigabyte**: G292-Z20, G482-Z50, G292-Z40, R282-Z90

### 主机命名规则
- 前缀：gpu, cpu, storage, compute, ai, ml, hpc
- 格式：`{prefix}-node-{number:03d}`
- 示例：`gpu-node-001`, `compute-node-042`

### 网络配置
- IP地址范围：10.x.x.x（私有网络）
- 随机分配，避免冲突

### 在线状态
- 80%的机器在线（最近5分钟内活跃）
- 20%的机器离线（5分钟到7天前最后在线）

## 故障排除

### 常见问题

1. **无法连接到API服务**
   - 检查后端服务是否正在运行
   - 确认端口8080没有被占用
   - 检查防火墙设置

2. **导入失败**
   - 检查数据库连接
   - 确认API接口正常工作
   - 查看后端服务日志

3. **Python依赖问题**
   ```bash
   pip install requests
   ```

### 自定义配置

如果需要修改API地址或其他配置，请编辑 `import_test_data.py` 中的配置部分：

```python
# 配置
api_base_url = "http://localhost:8080/api/v1"  # 修改为实际的API地址
test_data_file = "test_clients.json"
```

## 注意事项

1. **数据清理**：导入测试数据前，建议备份现有数据
2. **性能考虑**：100台机器的数据对于测试环境是合适的，生产环境可能需要更多数据
3. **数据一致性**：每次运行生成脚本都会创建新的随机数据
4. **安全性**：测试数据仅用于开发和测试，不包含敏感信息

## 扩展功能

如果需要生成更多或不同类型的测试数据，可以修改 `generate_test_data.py` 中的配置：

- 修改机器数量
- 添加新的操作系统
- 添加新的厂商和型号
- 调整在线/离线比例
- 添加更多硬件信息字段
