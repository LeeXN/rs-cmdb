#!/usr/bin/env python3
"""
导入测试数据脚本
将生成的测试数据导入到rs-cmdb数据库中，包括客户端和硬件信息
"""

import json
import requests
import time
import sys

def import_clients(api_base_url, clients_data, token):
    """导入客户端数据 (批量导入)"""
    print(f"开始导入 {len(clients_data)} 台机器数据...")
    
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}"
    }
    
    try:
        # 使用批量导入接口
        response = requests.post(
            f"{api_base_url}/clients/import",
            json=clients_data,
            headers=headers,
            timeout=60
        )
        
        if response.status_code == 200:
            print(f"批量导入成功: {len(clients_data)} 台机器")
            return len(clients_data), 0
        else:
            print(f"批量导入失败: HTTP {response.status_code}")
            if response.text:
                print(f"  错误信息: {response.text}")
            return 0, len(clients_data)
            
    except requests.exceptions.RequestException as e:
        print(f"网络错误: {e}")
        return 0, len(clients_data)
    except Exception as e:
        print(f"未知错误: {e}")
        return 0, len(clients_data)

def import_hardware(api_base_url, hardware_data):
    """导入硬件数据"""
    success_count = 0
    error_count = 0
    
    print(f"开始导入 {len(hardware_data)} 台机器的硬件数据...")
    
    for i, hardware_info in enumerate(hardware_data, 1):
        try:
            client_id = hardware_info["client_id"]
            
            # 构造符合API期望的ClientHardwareInfo格式
            client_hardware_info = {
                "client_id": client_id,
                "hardware": {
                    "system": hardware_info.get("system"),
                    "os": hardware_info.get("os"),
                    "cpu": hardware_info["cpu"],
                    "gpus": hardware_info["gpus"],
                    "ram": hardware_info["ram"],
                    "disks": hardware_info["disks"],
                    "nics": hardware_info["nics"]
                },
                "collected_at": hardware_info.get("collected_at", hardware_info.get("created_at"))
            }
            
            # 发送POST请求更新硬件信息
            response = requests.post(
                f"{api_base_url}/clients/{client_id}/hardware",
                json=client_hardware_info,
                headers={"Content-Type": "application/json"},
                timeout=10
            )
            
            if response.status_code in [200, 201]:
                success_count += 1
                if i % 10 == 0:
                    print(f"已导入 {i}/{len(hardware_data)} 台机器硬件数据...")
            else:
                error_count += 1
                print(f"硬件导入失败 {client_id}: HTTP {response.status_code}")
                if response.text:
                    print(f"  错误信息: {response.text}")
                    
        except requests.exceptions.RequestException as e:
            error_count += 1
            print(f"网络错误 {client_id}: {e}")
        except Exception as e:
            error_count += 1
            print(f"未知错误 {client_id}: {e}")
        
        # 避免请求过于频繁
        time.sleep(0.1)
    
    return success_count, error_count

def login(api_base_url, username, password):
    """登录获取token"""
    try:
        response = requests.post(
            f"{api_base_url}/auth/login",
            json={"username": username, "password": password},
            headers={"Content-Type": "application/json"},
            timeout=10
        )
        if response.status_code == 200:
            data = response.json()
            if "data" in data and "token" in data["data"]:
                print(f"登录成功: {username}")
                return data["data"]["token"]
            else:
                print("登录失败: 响应格式错误")
        else:
            print(f"登录失败: HTTP {response.status_code}")
    except Exception as e:
        print(f"登录异常: {e}")
    return None

def import_racks(api_base_url, racks_data, token):
    """导入机柜数据"""
    success_count = 0
    error_count = 0
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}"
    }
    print(f"开始导入 {len(racks_data)} 个机柜数据...")
    for i, rack in enumerate(racks_data, 1):
        try:
            response = requests.post(f"{api_base_url}/racks", json=rack, headers=headers, timeout=10)
            if response.status_code in [200, 201]:
                success_count += 1
            else:
                error_count += 1
                print(f"机柜导入失败 {rack['name']}: HTTP {response.status_code}")
        except Exception as e:
            error_count += 1
            print(f"未知错误 {rack['name']}: {e}")
        time.sleep(0.05)
    return success_count, error_count

def import_persons(api_base_url, persons_data, token):
    """导入用户数据"""
    success_count = 0
    error_count = 0
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}"
    }
    print(f"开始导入 {len(persons_data)} 个用户数据...")
    for i, person in enumerate(persons_data, 1):
        try:
            response = requests.post(f"{api_base_url}/users", json=person, headers=headers, timeout=10)
            if response.status_code in [200, 201]:
                success_count += 1
            else:
                error_count += 1
                print(f"用户导入失败 {person['name']}: HTTP {response.status_code}")
        except Exception as e:
            error_count += 1
            print(f"未知错误 {person['name']}: {e}")
        time.sleep(0.05)
    return success_count, error_count

def import_projects(api_base_url, projects_data, token):
    """导入项目数据"""
    success_count = 0
    error_count = 0
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}"
    }
    print(f"开始导入 {len(projects_data)} 个项目数据...")
    for i, project in enumerate(projects_data, 1):
        try:
            response = requests.post(f"{api_base_url}/projects", json=project, headers=headers, timeout=10)
            if response.status_code in [200, 201]:
                success_count += 1
            else:
                error_count += 1
                print(f"项目导入失败 {project['name']}: HTTP {response.status_code}")
        except Exception as e:
            error_count += 1
            print(f"未知错误 {project['name']}: {e}")
        time.sleep(0.05)
    return success_count, error_count

def import_components(api_base_url, components_data, token):
    """导入组件数据"""
    success_count = 0
    error_count = 0
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}"
    }
    print(f"开始导入 {len(components_data)} 个组件数据...")
    for i, component in enumerate(components_data, 1):
        try:
            response = requests.post(f"{api_base_url}/components", json=component, headers=headers, timeout=10)
            if response.status_code in [200, 201]:
                success_count += 1
            else:
                error_count += 1
                print(f"组件导入失败 {component['serial_number']}: HTTP {response.status_code}")
        except Exception as e:
            error_count += 1
            print(f"未知错误 {component['serial_number']}: {e}")
        time.sleep(0.05)
    return success_count, error_count

def import_dictionaries(api_base_url, dictionaries_data, token):
    """导入字典数据"""
    success_count = 0
    error_count = 0
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}"
    }
    print(f"开始导入 {len(dictionaries_data)} 个字典数据...")
    for i, dictionary in enumerate(dictionaries_data, 1):
        try:
            response = requests.post(f"{api_base_url}/dictionaries", json=dictionary, headers=headers, timeout=10)
            if response.status_code in [200, 201]:
                success_count += 1
            else:
                error_count += 1
                print(f"字典导入失败 {dictionary['key']}: HTTP {response.status_code}")
        except Exception as e:
            error_count += 1
            print(f"未知错误 {dictionary['key']}: {e}")
        time.sleep(0.05)
    return success_count, error_count

def main():
    """主函数"""
    # 配置
    api_base_url = "http://localhost:8080/api/v1"  # 根据实际情况调整
    clients_file = "test_clients.json"
    hardware_file = "test_hardware.json"
    racks_file = "test_racks.json"
    persons_file = "test_persons.json"
    projects_file = "test_projects.json"
    components_file = "test_components.json"
    dictionaries_file = "test_dictionaries.json"
    
    # 读取数据文件
    try:
        with open(clients_file, 'r', encoding='utf-8') as f:
            clients_data = json.load(f)
    except FileNotFoundError:
        print(f"错误: 找不到客户端数据文件 {clients_file}")
        sys.exit(1)
        
    hardware_data = []
    try:
        with open(hardware_file, 'r', encoding='utf-8') as f:
            hardware_data = json.load(f)
    except FileNotFoundError:
        pass
        
    racks_data = []
    try:
        with open(racks_file, 'r', encoding='utf-8') as f:
            racks_data = json.load(f)
    except FileNotFoundError:
        pass
        
    persons_data = []
    try:
        with open(persons_file, 'r', encoding='utf-8') as f:
            persons_data = json.load(f)
    except FileNotFoundError:
        pass
        
    projects_data = []
    try:
        with open(projects_file, 'r', encoding='utf-8') as f:
            projects_data = json.load(f)
    except FileNotFoundError:
        pass

    components_data = []
    try:
        with open(components_file, 'r', encoding='utf-8') as f:
            components_data = json.load(f)
    except FileNotFoundError:
        pass

    dictionaries_data = []
    try:
        with open(dictionaries_file, 'r', encoding='utf-8') as f:
            dictionaries_data = json.load(f)
    except FileNotFoundError:
        pass
    
    # 检查API服务是否可用
    try:
        response = requests.get(f"{api_base_url}/health", timeout=5)
        if response.status_code != 200:
            print(f"警告: API服务状态异常 (HTTP {response.status_code})")
    except requests.exceptions.RequestException:
        print(f"警告: 无法连接到API服务 {api_base_url}")
        print("请确保后端服务正在运行")
        choice = input("是否继续尝试导入? (y/N): ").strip().lower()
        if choice != 'y':
            sys.exit(0)
            
    # 登录获取Token
    token = login(api_base_url, "admin", "admin")
    if not token:
        print("无法获取Token，停止导入受保护的数据")
        return

    # 导入基础数据
    if racks_data:
        import_racks(api_base_url, racks_data, token)
    if persons_data:
        import_persons(api_base_url, persons_data, token)
    if projects_data:
        import_projects(api_base_url, projects_data, token)
    if components_data:
        import_components(api_base_url, components_data, token)
    if dictionaries_data:
        import_dictionaries(api_base_url, dictionaries_data, token)
    
    # 导入客户端数据
    client_success, client_error = import_clients(api_base_url, clients_data, token)
    
    # 导入硬件数据
    hardware_success = 0
    hardware_error = 0
    if hardware_data and client_success > 0:
        print("\n等待3秒后开始导入硬件数据...")
        time.sleep(3)
        hardware_success, hardware_error = import_hardware(api_base_url, hardware_data)
    
    # 输出结果
    print(f"\n=== 导入完成 ===")
    print(f"客户端导入成功: {client_success} 台")
    if hardware_data:
        print(f"硬件数据导入成功: {hardware_success} 台")
    print(f"总计: {len(clients_data)} 台")

if __name__ == "__main__":
    main() 