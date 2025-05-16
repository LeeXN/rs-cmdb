#!/usr/bin/env python3
"""
生成测试数据脚本
用于为rs-cmdb系统生成100台机器的测试数据，包含完整的硬件信息
"""

import json
import random
import uuid
from datetime import datetime, timedelta

# 配置数据
OS_OPTIONS = [
    "Ubuntu 20.04 LTS", "Ubuntu 22.04 LTS", "CentOS 7", "CentOS 8", 
    "RHEL 8", "RHEL 9", "Debian 11", "Debian 12", "Rocky Linux 8", "Rocky Linux 9"
]

KERNEL_VERSIONS = [
    "5.4.0-150-generic", "5.15.0-76-generic", "3.10.0-1160.el7.x86_64", "4.18.0-348.el8.x86_64",
    "5.14.0-284.11.1.el9_2.x86_64", "5.10.0-23-amd64", "6.1.0-9-amd64"
]

VENDORS = [
    "Dell Inc.", "Hewlett-Packard", "Lenovo", "Supermicro", "ASUS", "Gigabyte"
]

DELL_MODELS = ["PowerEdge R740", "PowerEdge R750", "PowerEdge R640", "PowerEdge R650"]
HP_MODELS = ["ProLiant DL380", "ProLiant DL360", "ProLiant DL385", "ProLiant ML350"]
LENOVO_MODELS = ["ThinkSystem SR650", "ThinkSystem SR630", "ThinkSystem SR850", "ThinkSystem SR950"]
SUPERMICRO_MODELS = ["SuperServer 1029P", "SuperServer 2029P", "SuperServer 6029P", "SuperServer 4029P"]
ASUS_MODELS = ["ESC4000 G4", "ESC8000 G4", "RS720-E9", "RS500A-E10"]
GIGABYTE_MODELS = ["R282-Z92", "R181-T90", "G292-Z20", "H262-Z61"]

MODEL_MAP = {
    "Dell Inc.": DELL_MODELS,
    "Hewlett-Packard": HP_MODELS,
    "Lenovo": LENOVO_MODELS,
    "Supermicro": SUPERMICRO_MODELS,
    "ASUS": ASUS_MODELS,
    "Gigabyte": GIGABYTE_MODELS,
}

# CPU配置
CPU_MODELS = [
    "Intel Xeon Gold 6248R", "Intel Xeon Gold 6258R", "Intel Xeon Platinum 8280",
    "Intel Xeon Silver 4214", "Intel Xeon E5-2680 v4", "Intel Core i9-12900K",
    "AMD EPYC 7742", "AMD EPYC 7502", "AMD EPYC 7302", "AMD Ryzen 9 5950X"
]

# GPU配置
GPU_CONFIGS = [
    {"vendor": "NVIDIA", "model": "RTX 4090", "memory": "24GB", "driver": "535.86.05"},
    {"vendor": "NVIDIA", "model": "RTX 3080", "memory": "10GB", "driver": "535.86.05"},
    {"vendor": "NVIDIA", "model": "RTX 3070", "memory": "8GB", "driver": "535.86.05"},
    {"vendor": "NVIDIA", "model": "GTX 1080 Ti", "memory": "11GB", "driver": "470.199.02"},
    {"vendor": "AMD", "model": "RX 7900 XTX", "memory": "24GB", "driver": "23.7.1"},
    {"vendor": "AMD", "model": "RX 6800 XT", "memory": "16GB", "driver": "23.7.1"},
]

# 内存配置
MEMORY_CONFIGS = [
    {"total": 32, "modules": 4, "type": "DDR4", "speed": 3200},
    {"total": 64, "modules": 4, "type": "DDR4", "speed": 3200},
    {"total": 128, "modules": 8, "type": "DDR4", "speed": 3200},
    {"total": 256, "modules": 8, "type": "DDR5", "speed": 4800},
    {"total": 512, "modules": 16, "type": "DDR5", "speed": 4800},
]

# 存储配置
STORAGE_CONFIGS = [
    {"type": "NVMe", "size": 1000, "vendor": "Samsung", "model": "980 PRO"},
    {"type": "NVMe", "size": 2000, "vendor": "WD", "model": "Black SN850"},
    {"type": "SSD", "size": 500, "vendor": "Samsung", "model": "860 EVO"},
    {"type": "SSD", "size": 1000, "vendor": "Crucial", "model": "MX500"},
    {"type": "HDD", "size": 4000, "vendor": "Seagate", "model": "Barracuda"},
    {"type": "HDD", "size": 8000, "vendor": "WD", "model": "Red Plus"},
]

# 网卡配置
NIC_CONFIGS = [
    {"type": "Ethernet", "speed": 1000, "vendor": "Intel", "status": "Up"},
    {"type": "Ethernet", "speed": 10000, "vendor": "Intel", "status": "Up"},
    {"type": "InfiniBand", "speed": 100000, "vendor": "Mellanox", "status": "Up"},
    {"type": "Ethernet", "speed": 25000, "vendor": "Broadcom", "status": "Up"},
]

# 新增配置
RACK_LOCATIONS = ["Data Center A", "Data Center B", "Server Room 101", "Server Room 102"]
DEPARTMENTS = ["Engineering", "Product", "Sales", "Marketing", "IT", "HR", "Operations", "Finance"]
TITLES = [
    "Software Engineer", "Senior Software Engineer", "Principal Engineer", 
    "DevOps Engineer", "Site Reliability Engineer", "System Administrator",
    "Product Manager", "Engineering Manager", "Director of Engineering",
    "IT Support Specialist", "Network Engineer", "Database Administrator",
    "Data Scientist", "QA Engineer"
]
PROJECT_NAMES = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Omega", "Phoenix", "Dragon", "Titan", "Apollo"]

def generate_dictionaries():
    """生成字典数据"""
    dictionaries = []
    
    # Departments
    for dept in DEPARTMENTS:
        dictionaries.append({
            "id": str(uuid.uuid4()),
            "category": "Department",
            "key": dept.upper(),
            "value": dept,
            "description": f"{dept} Department",
            "created_at": datetime.now().isoformat() + "Z",
            "updated_at": datetime.now().isoformat() + "Z"
        })
        
    # Titles
    for title in TITLES:
        dictionaries.append({
            "id": str(uuid.uuid4()),
            "category": "Title",
            "key": title.replace(" ", "_").upper(),
            "value": title,
            "description": None,
            "created_at": datetime.now().isoformat() + "Z",
            "updated_at": datetime.now().isoformat() + "Z"
        })
        
    # Cost Centers (Sample)
    for i in range(1, 6):
        dictionaries.append({
            "id": str(uuid.uuid4()),
            "category": "CostCenter",
            "key": f"CC-{100+i}",
            "value": f"Cost Center {100+i}",
            "description": "General Cost Center",
            "created_at": datetime.now().isoformat() + "Z",
            "updated_at": datetime.now().isoformat() + "Z"
        })
        
    return dictionaries

def generate_racks(count=10):
    """生成机柜数据"""
    racks = []
    for i in range(1, count + 1):
        location = random.choice(RACK_LOCATIONS)
        racks.append({
            "id": str(uuid.uuid4()),
            "name": f"Rack-{chr(65 + (i-1)//10)}{(i-1)%10 + 1:02d}", # Rack-A01, Rack-A02...
            "location": location,
            "height_u": 42,
            "power_limit": random.choice([5000, 8000, 10000, 12000]),
            "description": f"Standard 42U Rack in {location}",
            "created_at": datetime.now().isoformat() + "Z",
            "updated_at": datetime.now().isoformat() + "Z"
        })
    return racks

def generate_persons(count=20):
    """生成人员数据"""
    persons = []
    first_names = ["James", "Mary", "John", "Patricia", "Robert", "Jennifer", "Michael", "Linda", "William", "Elizabeth"]
    last_names = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez"]
    
    for i in range(count):
        first = random.choice(first_names)
        last = random.choice(last_names)
        name = f"{first} {last}"
        persons.append({
            "id": str(uuid.uuid4()),
            "name": name,
            "email": f"{first.lower()}.{last.lower()}@example.com",
            "phone": f"555-{random.randint(100, 999)}-{random.randint(1000, 9999)}",
            "department": random.choice(DEPARTMENTS),
            "title": random.choice(TITLES),
            "cost_center": f"CC-{random.randint(100, 999)}",
            "created_at": datetime.now().isoformat() + "Z",
            "updated_at": datetime.now().isoformat() + "Z"
        })
    return persons

def generate_projects(count=10, persons=None):
    """生成项目数据"""
    projects = []
    for i in range(count):
        name = f"Project {random.choice(PROJECT_NAMES)} {i+1}"
        manager_id = random.choice(persons)["id"] if persons else None
        projects.append({
            "id": str(uuid.uuid4()),
            "name": name,
            "code": f"PRJ-{random.randint(1000, 9999)}",
            "department": random.choice(DEPARTMENTS),
            "cost_center": f"CC-{random.randint(100, 999)}",
            "manager_id": manager_id, 
            "created_at": datetime.now().isoformat() + "Z",
            "updated_at": datetime.now().isoformat() + "Z"
        })
    return projects

def generate_components(count=50):
    """生成未使用的组件数据"""
    components = []
    component_types = ["GPU", "CPU", "Memory", "Disk", "NetworkCard", "Motherboard", "PowerSupply"]
    vendors = ["NVIDIA", "Intel", "AMD", "Samsung", "Seagate", "Cisco", "Dell"]
    
    for i in range(count):
        comp_type = random.choice(component_types)
        vendor = random.choice(vendors)
        
        components.append({
            "id": str(uuid.uuid4()),
            "serial_number": f"COMP-{random.randint(100000, 999999)}",
            "model": f"{vendor} {comp_type} Model-{random.randint(100, 999)}",
            "vendor": vendor,
            "component_type": comp_type,
            "status": "InStock",
            "client_id": None,
            "client_hostname": None,
            "location": random.choice(RACK_LOCATIONS), # In stock at a location
            "purchase_date": (datetime.now() - timedelta(days=random.randint(1, 365))).strftime("%Y-%m-%d"),
            "warranty_expiration": (datetime.now() + timedelta(days=random.randint(100, 1000))).strftime("%Y-%m-%d"),
            "created_at": datetime.now().isoformat() + "Z",
            "updated_at": datetime.now().isoformat() + "Z"
        })
    return components

def generate_client_data(index, racks=None, persons=None, projects=None, rack_usage=None):
    """生成单个客户端数据"""
    vendor = random.choice(VENDORS)
    model = random.choice(MODEL_MAP[vendor])
    
    # 生成主机名
    hostname_prefixes = ["gpu-node", "compute-node", "ai-node", "ml-server", "hpc-node"]
    hostname = f"{random.choice(hostname_prefixes)}-{index:03d}"
    
    # 生成IP地址
    ip = f"10.{random.randint(1, 254)}.{random.randint(1, 254)}.{random.randint(1, 254)}"
    
    # 生成序列号
    serial_number = f"{vendor[:3].upper()}{random.randint(100000, 999999)}"
    
    # 生成最后在线时间（80%在线，20%离线）
    is_online = random.random() < 0.8
    if is_online:
        last_seen = datetime.now() - timedelta(minutes=random.randint(0, 4))
    else:
        last_seen = datetime.now() - timedelta(hours=random.randint(1, 72))
    
    # 关联信息 - 提高关联概率
    rack = None
    unit_position = None
    
    if racks and random.random() > 0.05:  # 95% 概率尝试分配机柜
        # 尝试找到一个有空位的机柜
        shuffled_racks = list(racks)
        random.shuffle(shuffled_racks)
        
        for candidate_rack in shuffled_racks:
            rack_id = candidate_rack["id"]
            if rack_usage is not None:
                used_slots = rack_usage.get(rack_id, set())
                # 假设每个服务器占用 1U 或 2U
                height = random.choice([1, 2])
                
                # 寻找可用空间
                available_slots = []
                for start_u in range(1, 43 - height + 1):
                    # 检查该位置及所需高度是否都被占用
                    slots_needed = set(range(start_u, start_u + height))
                    if not (slots_needed & used_slots):
                        available_slots.append((start_u, slots_needed))
                
                if available_slots:
                    rack = candidate_rack
                    start_u, slots_needed = random.choice(available_slots)
                    unit_position = str(start_u)
                    rack_usage[rack_id].update(slots_needed)
                    break
            else:
                # 如果没有传入 usage 追踪，就随机分配（旧逻辑，不推荐）
                rack = candidate_rack
                unit_position = str(random.randint(1, 42))
                break

    owner = random.choice(persons) if persons and random.random() > 0.05 else None # 95% 概率分配负责人
    project = random.choice(projects) if projects and random.random() > 0.05 else None # 95% 概率分配项目

    client_data = {
        "id": str(uuid.uuid4()),
        "hostname": hostname,
        "ip_address": ip,
        "os": random.choice(OS_OPTIONS),
        "kernel_version": random.choice(KERNEL_VERSIONS),
        "sys_vendor": vendor,
        "product_name": model,
        "serial_number": serial_number,
        "last_seen": last_seen.isoformat() + "Z",
        "created_at": (datetime.now() - timedelta(days=random.randint(1, 365))).isoformat() + "Z",
        "updated_at": datetime.now().isoformat() + "Z",
        
        # 新增关联字段
        "rack": rack["id"] if rack else None,
        "location": rack["location"] if rack else None,
        "unit_position": unit_position,
        "owner_id": owner["id"] if owner else None,
        "project_id": project["id"] if project else None,
        "status": random.choice(["Active", "Maintenance", "InStock", "Decommissioned"]),
        "environment": random.choice(["Prod", "Dev", "Test", "Staging"]),
        "asset_tag": f"ASSET-{random.randint(10000, 99999)}",
        "warranty_expiration": (datetime.now() + timedelta(days=random.randint(100, 1000))).strftime("%Y-%m-%d"),
        "supplier": random.choice(["Vendor A", "Vendor B", "Vendor C"])
    }
    
    return client_data

def generate_hardware_data(client_data):
    """生成硬件数据"""
    # OS配置
    os_info = {
        "name": client_data["os"].split(" ")[0], # Ubuntu, CentOS etc
        "version": client_data["os"],
        "kernel": client_data["kernel_version"],
        "architecture": "x86_64",
        "hostname": client_data["hostname"],
        "dns": "8.8.8.8, 8.8.4.4",
        "ip_address": client_data["ip_address"]
    }

    # CPU配置
    cpu_config = {
        "vendor_id": "Intel" if "Intel" in random.choice(CPU_MODELS) else "AMD",
        "model_name": random.choice(CPU_MODELS),
        "cores": random.choice([8, 16, 24, 32, 48, 64]),
        "threads": 0,  # 会根据cores计算
        "cpus": 1,
        "speed": random.randint(2000, 4000),  # MHz
        "flags": ["fpu", "vme", "de", "pse", "tsc", "msr", "pae", "mce"]
    }
    cpu_config["threads"] = cpu_config["cores"] * 2
    
    # 内存配置
    memory_config = random.choice(MEMORY_CONFIGS)
    memory_modules = []
    for i in range(memory_config["modules"]):
        # 5% chance of missing serial number to test virtual serial generation
        serial = f"SN{random.randint(100000, 999999)}"
        if random.random() < 0.05:
            serial = random.choice(["", "N/A", "Unknown"])
            
        memory_modules.append({
            "slot": f"DIMM_{i}",
            "vendor": random.choice(["Samsung", "SK Hynix", "Micron", "Corsair"]),
            "part_number": f"M393A{random.randint(1000, 9999)}",
            "serial_number": serial,
            "size": memory_config["total"] // memory_config["modules"],
            "speed": memory_config["speed"],
            "form_factor": "DIMM",
            "memory_type": memory_config["type"],
            "locator": f"DIMM_{i}"
        })
    
    # GPU配置（30%有GPU，70%无GPU）
    gpus = []
    if random.random() < 0.3:
        gpu_count = random.choice([1, 2, 4])
        gpu_config = random.choice(GPU_CONFIGS)
        for i in range(gpu_count):
            # 5% chance of missing serial number
            serial = f"GPU{random.randint(100000, 999999)}"
            if random.random() < 0.05:
                serial = random.choice(["", "N/A", "Unknown"])
                
            gpus.append({
                "vendor": gpu_config["vendor"],
                "model": gpu_config["model"],
                "device_id": f"10de:{random.randint(1000, 9999):04x}",
                "serial_number": serial,
                "driver_version": gpu_config["driver"]
            })
    
    # 存储配置
    disks = []
    disk_count = random.choice([1, 2, 3, 4])
    for i in range(disk_count):
        storage_config = random.choice(STORAGE_CONFIGS)
        # 5% chance of missing serial number
        serial = f"S{random.randint(100000000, 999999999)}"
        if random.random() < 0.05:
            serial = random.choice(["", "N/A", "Unknown"])
            
        disks.append({
            "vendor": storage_config["vendor"],
            "size": str(storage_config["size"]),
            "size_unit": "GB",
            "model": storage_config["model"],
            "storage_type": storage_config["type"],
            "firmware_version": f"FW{random.randint(100, 999)}",
            "serial_number": serial,
            "parted": random.choice([True, False]),
            "partitions": []
        })
    
    # 网卡配置
    nics = []
    nic_count = random.choice([1, 2, 4])
    for i in range(nic_count):
        nic_config = random.choice(NIC_CONFIGS)
        nics.append({
            "name": f"eth{i}",
            "vendor": nic_config["vendor"],
            "model": f"Ethernet Controller {random.randint(1000, 9999)}",
            "speed": nic_config["speed"],
            "mac_address": ":".join([f"{random.randint(0, 255):02x}" for _ in range(6)]),
            "ipv4_address": f"10.{random.randint(1, 254)}.{random.randint(1, 254)}.{random.randint(1, 254)}",
            "ipv4_subnet_mask": "255.255.255.0",
            "ipv4_gateway": f"10.{random.randint(1, 254)}.{random.randint(1, 254)}.1",
            "ipv6_address": "",
            "ipv6_subnet_mask": "",
            "ipv6_gateway": "",
            "dhcp": random.choice([True, False]),
            "bonding_slaves": [],
            "nic_type": nic_config["type"],
            "status": nic_config["status"] if random.random() < 0.95 else "Down",
            "pci_slot": f"0000:{random.randint(0, 255):02x}:00.0" if random.random() > 0.1 else None,
            "firmware_version": f"{random.randint(1, 10)}.{random.randint(0, 99)}",
            "ib_node_type": "",
            "driver": "ixgbe" if nic_config["vendor"] == "Intel" else "bnxt_en"
        })
    
    # System配置
    system_info = {
        "sys_vendor": client_data["sys_vendor"],
        "product_name": client_data["product_name"],
        "product_version": "1.0",
        "serial_number": client_data["serial_number"]
    }

    hardware_data = {
        "client_id": client_data["id"],
        "system": system_info,
        "os": os_info,
        "cpu": cpu_config,
        "ram": {
            "vendor": random.choice(["Samsung", "SK Hynix", "Micron"]),
            "model": f"DDR{random.choice([4, 5])}-{random.choice([3200, 4800])}",
            "size": memory_config["total"] // memory_config["modules"],
            "speed": memory_config["speed"],
            "total_size": memory_config["total"],
            "count": memory_config["modules"],
            "form_factor": "DIMM",
            "modules": memory_modules
        },
        "gpus": gpus,
        "disks": disks,
        "nics": nics,
        "collected_at": datetime.now().isoformat() + "Z",
        "created_at": datetime.now().isoformat() + "Z",
        "updated_at": datetime.now().isoformat() + "Z"
    }
    
    return hardware_data

def main():
    """主函数"""
    print("开始生成测试数据...")
    
    # 生成基础数据
    racks = generate_racks(10)
    persons = generate_persons(20)
    projects = generate_projects(10, persons)
    components = generate_components(50)
    dictionaries = generate_dictionaries()
    
    print(f"已生成 {len(racks)} 个机柜")
    print(f"已生成 {len(persons)} 个用户")
    print(f"已生成 {len(projects)} 个项目")
    print(f"已生成 {len(components)} 个库存组件")
    print(f"已生成 {len(dictionaries)} 个字典项")
    
    clients_data = []
    hardware_data = []
    
    # 追踪机柜使用情况 {rack_id: set(used_u_positions)}
    rack_usage = {rack["id"]: set() for rack in racks}

    # 生成100台机器的数据
    for i in range(1, 101):
        # 生成客户端数据
        client = generate_client_data(i, racks, persons, projects, rack_usage)
        clients_data.append(client)
        
        # 生成对应的硬件数据
        hardware = generate_hardware_data(client)
        hardware_data.append(hardware)
        
        if i % 10 == 0:
            print(f"已生成 {i}/100 台机器数据...")
    
    # 保存客户端数据
    with open('test_clients.json', 'w', encoding='utf-8') as f:
        json.dump(clients_data, f, ensure_ascii=False, indent=2)
    
    # 保存硬件数据
    with open('test_hardware.json', 'w', encoding='utf-8') as f:
        json.dump(hardware_data, f, ensure_ascii=False, indent=2)
        
    # 保存基础数据
    with open('test_racks.json', 'w', encoding='utf-8') as f:
        json.dump(racks, f, ensure_ascii=False, indent=2)
        
    with open('test_persons.json', 'w', encoding='utf-8') as f:
        json.dump(persons, f, ensure_ascii=False, indent=2)
        
    with open('test_projects.json', 'w', encoding='utf-8') as f:
        json.dump(projects, f, ensure_ascii=False, indent=2)

    with open('test_components.json', 'w', encoding='utf-8') as f:
        json.dump(components, f, ensure_ascii=False, indent=2)

    with open('test_dictionaries.json', 'w', encoding='utf-8') as f:
        json.dump(dictionaries, f, ensure_ascii=False, indent=2)
    
    print(f"\n=== 数据生成完成 ===")
    print(f"客户端数据已保存到: test_clients.json")
    print(f"硬件数据已保存到: test_hardware.json")
    print(f"机柜数据已保存到: test_racks.json")
    print(f"用户数据已保存到: test_persons.json")
    print(f"项目数据已保存到: test_projects.json")
    print(f"组件数据已保存到: test_components.json")
    print(f"字典数据已保存到: test_dictionaries.json")
    print(f"总计生成: {len(clients_data)} 台机器")
    
    # 统计信息
    os_stats = {}
    vendor_stats = {}
    gpu_stats = {"有GPU": 0, "无GPU": 0}
    
    for client in clients_data:
        # 操作系统统计
        os = client["os"]
        os_stats[os] = os_stats.get(os, 0) + 1
        
        # 厂商统计
        vendor = client["sys_vendor"]
        vendor_stats[vendor] = vendor_stats.get(vendor, 0) + 1
    
    for hardware in hardware_data:
        # GPU统计
        if hardware["gpus"]:
            gpu_stats["有GPU"] += 1
        else:
            gpu_stats["无GPU"] += 1
    
    print(f"\n=== 数据分布统计 ===")
    print("操作系统分布:")
    for os, count in sorted(os_stats.items()):
        print(f"  {os}: {count} 台 ({count/len(clients_data)*100:.1f}%)")
    
    print("\n厂商分布:")
    for vendor, count in sorted(vendor_stats.items()):
        print(f"  {vendor}: {count} 台 ({count/len(clients_data)*100:.1f}%)")
    
    print("\nGPU分布:")
    for gpu_type, count in gpu_stats.items():
        print(f"  {gpu_type}: {count} 台 ({count/len(clients_data)*100:.1f}%)")

if __name__ == "__main__":
    main() 