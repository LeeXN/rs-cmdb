#!/usr/bin/env python3
"""
生成硬件变更测试数据
模拟硬件升级、更换、抖动、无序列号组件等场景，验证系统健壮性
"""

import json
import random
import time
import requests
import copy
from datetime import datetime, timedelta
from typing import Dict, List, Any

# API配置
API_BASE_URL = "http://localhost:8080/api/v1"

# 硬件变更场景
HARDWARE_CHANGE_SCENARIOS = [
    {
        "name": "内存升级",
        "description": "增加内存容量",
        "type": "upgrade",
        "changes": ["ram"],
        "weight": 3
    },
    {
        "name": "存储扩容", 
        "description": "添加新的存储设备",
        "type": "upgrade",
        "changes": ["disks"],
        "weight": 4
    },
    {
        "name": "显卡升级",
        "description": "更换或添加显卡",
        "type": "upgrade",
        "changes": ["gpus"],
        "weight": 2
    },
    {
        "name": "组件抖动 (Flapping)",
        "description": "模拟组件短暂丢失后恢复，验证宽限期逻辑",
        "type": "flapping",
        "changes": ["disks"], # 模拟硬盘接触不良
        "weight": 3
    },
    {
        "name": "无序列号组件",
        "description": "添加无序列号组件，验证虚拟序列号生成",
        "type": "missing_serial",
        "changes": ["ram"], # 模拟廉价内存条
        "weight": 2
    },
    {
        "name": "客户端重装 (Re-install)",
        "description": "模拟客户端重装系统丢失ID，验证基于SN的去重",
        "type": "reinstall",
        "changes": [],
        "weight": 2
    }
]

# ... (保留原有的硬件池配置) ...
# CPU型号池
CPU_TIERS = [
    [{"vendor": "Intel", "model": "Intel(R) Core(TM) i5-10400F", "cores": 6, "threads": 12, "speed": 2900}],
    [{"vendor": "Intel", "model": "Intel(R) Core(TM) i7-12700K", "cores": 12, "threads": 20, "speed": 3600}],
    [{"vendor": "Intel", "model": "Intel(R) Core(TM) i9-13900K", "cores": 24, "threads": 32, "speed": 3000}]
]

# 内存配置池
RAM_TIERS = [
    {"total": 8, "count": 1, "speed": 2666, "vendor": "Kingston"},
    {"total": 16, "count": 2, "speed": 3200, "vendor": "Kingston"},
    {"total": 32, "count": 2, "speed": 3600, "vendor": "Corsair"},
    {"total": 64, "count": 4, "speed": 3200, "vendor": "G.Skill"},
]

# GPU型号池
GPU_TIERS = [
    [{"vendor": "NVIDIA", "model": "GeForce GTX 1660", "device_id": "2184"}],
    [{"vendor": "NVIDIA", "model": "GeForce RTX 4060", "device_id": "2544"}],
    [{"vendor": "NVIDIA", "model": "GeForce RTX 4090", "device_id": "2684"}]
]

# 存储设备池
STORAGE_DEVICES = {
    "HDD": [{"vendor": "Seagate", "model": "Barracuda", "size": "1000", "size_unit": "GB", "type": "HDD"}],
    "SSD": [{"vendor": "Samsung", "model": "870 EVO", "size": "500", "size_unit": "GB", "type": "SSD"}],
    "NVMe": [{"vendor": "Samsung", "model": "980 PRO", "size": "1000", "size_unit": "GB", "type": "NVMe"}]
}

def get_existing_clients():
    try:
        response = requests.get(f"{API_BASE_URL}/clients", timeout=10)
        if response.status_code == 200:
            return response.json().get("data", [])
        return []
    except Exception as e:
        print(f"获取客户端列表失败: {e}")
        return []

def get_current_hardware(client_id: str) -> Dict[str, Any]:
    try:
        response = requests.get(f"{API_BASE_URL}/clients/{client_id}/hardware", timeout=10)
        if response.status_code == 200:
            return response.json().get("data")
        return None
    except Exception as e:
        print(f"获取客户端 {client_id} 硬件信息失败: {e}")
        return None

# ... (保留原有的升级生成函数，简化版) ...
def generate_ram_upgrade(current_ram):
    # 简单实现：增加一条内存
    new_ram = copy.deepcopy(current_ram)
    if "modules" in new_ram:
        new_module = copy.deepcopy(new_ram["modules"][0])
        new_module["serial_number"] = f"SN{random.randint(100000, 999999)}"
        new_module["slot"] = f"DIMM_{len(new_ram['modules'])}"
        new_ram["modules"].append(new_module)
        new_ram["count"] += 1
        new_ram["total_size"] += new_module["size"]
    return new_ram

def generate_disk_expansion(current_disks):
    new_disks = copy.deepcopy(current_disks)
    new_disk = random.choice(STORAGE_DEVICES["SSD"]).copy()
    new_disk["storage_type"] = new_disk.pop("type")
    new_disk["serial_number"] = f"SN{random.randint(1000000, 9999999)}"
    new_disks.append(new_disk)
    return new_disks

def generate_gpu_upgrade(current_gpus):
    if not current_gpus:
        new_gpu = random.choice(GPU_TIERS[1])[0].copy()
        new_gpu["serial_number"] = f"GPU{random.randint(1000000, 9999999)}"
        return [new_gpu]
    return current_gpus # 简化：已有显卡不升级

def apply_hardware_changes(hardware: Dict[str, Any], changes: List[str]) -> Dict[str, Any]:
    new_hardware = copy.deepcopy(hardware)
    for change in changes:
        if change == "ram":
            new_hardware["ram"] = generate_ram_upgrade(hardware["ram"])
        elif change == "disks":
            new_hardware["disks"] = generate_disk_expansion(hardware["disks"])
        elif change == "gpus":
            new_hardware["gpus"] = generate_gpu_upgrade(hardware["gpus"])
    return new_hardware

def push_hardware_change(client_id: str, hardware: Dict[str, Any], timestamp: str):
    client_hardware_info = {
        "client_id": client_id,
        "hardware": hardware,
        "collected_at": timestamp
    }
    try:
        response = requests.post(
            f"{API_BASE_URL}/clients/{client_id}/hardware",
            json=client_hardware_info,
            headers={"Content-Type": "application/json"},
            timeout=10
        )
        return response.status_code in [200, 201]
    except Exception as e:
        print(f"推送失败: {e}")
        return False

def simulate_flapping(client_id, current_hardware):
    """模拟组件抖动：移除 -> 推送 -> 恢复 -> 推送"""
    print(f"    执行组件抖动测试 (Flapping)...")
    
    # 1. 移除一个磁盘
    flapped_hardware = copy.deepcopy(current_hardware)
    if not flapped_hardware["disks"]:
        print("    无磁盘可移除，跳过")
        return
    
    removed_disk = flapped_hardware["disks"].pop()
    print(f"    暂时移除磁盘: {removed_disk.get('model')} ({removed_disk.get('serial_number')})")
    
    # 推送缺失状态
    timestamp = datetime.now().isoformat()
    if push_hardware_change(client_id, flapped_hardware, timestamp):
        print(f"    ✓ 推送缺失状态成功 (组件应标记为 Missing)")
    
    time.sleep(2)
    
    # 2. 恢复磁盘
    print(f"    恢复磁盘...")
    timestamp = datetime.now().isoformat()
    if push_hardware_change(client_id, current_hardware, timestamp):
        print(f"    ✓ 推送恢复状态成功 (组件应恢复为 InUse)")

def simulate_missing_serial(client_id, current_hardware):
    """模拟无序列号组件"""
    print(f"    执行无序列号组件测试...")
    
    new_hardware = copy.deepcopy(current_hardware)
    
    # 添加一个无序列号的内存条
    if "modules" in new_hardware["ram"]:
        new_module = copy.deepcopy(new_hardware["ram"]["modules"][0])
        new_module["serial_number"] = "" # 空序列号
        new_module["slot"] = f"DIMM_NO_SERIAL"
        new_hardware["ram"]["modules"].append(new_module)
        print(f"    添加无序列号内存条")
        
        timestamp = datetime.now().isoformat()
        if push_hardware_change(client_id, new_hardware, timestamp):
            print(f"    ✓ 推送成功 (应生成虚拟序列号)")

def simulate_reinstall(client):
    """模拟客户端重装（无ID注册）"""
    print(f"    执行客户端重装测试 (Re-install)...")
    
    # 构造注册请求，不带 client_id，但带相同的 serial_number
    register_data = {
        "hostname": client["hostname"],
        "ip_address": client["ip_address"],
        "sys_vendor": client["sys_vendor"],
        "product_name": client["product_name"],
        "serial_number": client["serial_number"], # 关键：相同的SN
        "os": client["os"]
    }
    
    try:
        response = requests.post(
            f"{API_BASE_URL}/clients/register",
            json=register_data,
            headers={"Content-Type": "application/json"},
            timeout=10
        )
        
        if response.status_code == 200:
            data = response.json()
            returned_id = data.get("data", {}).get("id")
            if returned_id == client["id"]:
                print(f"    ✓ 重装测试成功: 服务器返回了旧的 Client ID ({returned_id})")
            else:
                print(f"    ✗ 重装测试失败: 服务器返回了新的 Client ID ({returned_id})，期望 ({client['id']})")
        else:
            print(f"    ✗ 注册请求失败: {response.status_code}")
            
    except Exception as e:
        print(f"    ✗ 请求异常: {e}")

def main():
    print("开始生成硬件变更测试数据...")
    clients = get_existing_clients()
    if not clients:
        print("未找到客户端")
        return
    
    # 随机选择客户端进行测试
    selected_clients = random.sample(clients, min(10, len(clients)))
    
    for client in selected_clients:
        client_id = client["id"]
        hostname = client["hostname"]
        print(f"\n处理客户端: {hostname} ({client_id})")
        
        current_hardware = get_current_hardware(client_id)
        if not current_hardware:
            continue
            
        scenario = random.choice(HARDWARE_CHANGE_SCENARIOS)
        print(f"  场景: {scenario['name']}")
        
        if scenario["type"] == "upgrade":
            # 普通升级逻辑
            new_hardware = apply_hardware_changes(current_hardware, scenario["changes"])
            push_hardware_change(client_id, new_hardware, datetime.now().isoformat())
            print(f"    ✓ 硬件升级推送完成")
            
        elif scenario["type"] == "flapping":
            simulate_flapping(client_id, current_hardware)
            
        elif scenario["type"] == "missing_serial":
            simulate_missing_serial(client_id, current_hardware)
            
        elif scenario["type"] == "reinstall":
            simulate_reinstall(client)
            
        time.sleep(0.5)

if __name__ == "__main__":
    main()
