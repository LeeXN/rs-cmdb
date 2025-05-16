# Test Data Scripts

[中文文档](README_CN.md)

This directory contains scripts for generating and importing test data, helping you quickly create a test environment for the rs-cmdb system.

## Script Description

### 1. generate_test_data.py
Generates test data for 100 machines, including realistic hardware configuration information.

**Features:**
- Generates complete information for 100 machines
- Includes multiple operating systems (Ubuntu, CentOS, RHEL, Debian, Rocky Linux)
- Includes servers from multiple vendors (Dell, HP, Lenovo, Supermicro, ASUS, Gigabyte)
- Simulates realistic online/offline status (80% online, 20% offline)
- Generates reasonable hostnames, IP addresses, serial numbers, etc.

**Usage:**
```bash
cd scripts
python3 generate_test_data.py
```

**Output:**
- Generates `test_clients.json` file containing JSON data for 100 machines
- Displays detailed statistics, including OS distribution, vendor distribution, online rate, etc.

### 2. import_test_data.py
Imports the generated test data into the rs-cmdb database.

**Features:**
- Batch imports client data via API interface
- Supports error handling and retry mechanisms
- Displays import progress and result statistics
- Automatically detects API service availability

**Usage:**
```bash
cd scripts
python3 import_test_data.py
```

**Prerequisites:**
- Ensure the rs-cmdb backend service is running (default port 8080)
- Have already run `generate_test_data.py` to generate test data
- Installed `requests` library: `pip install requests`

## Complete Usage Workflow

1. **Generate Test Data**
   ```bash
   cd scripts
   python3 generate_test_data.py
   ```

2. **Start Backend Service**
   ```bash
   cd server
   cargo run
   ```

3. **Import Test Data**
   ```bash
   cd scripts
   python3 import_test_data.py
   ```

4. **Start Frontend Service**
   ```bash
   cd front
   trunk serve
   ```

5. **Access System**
   Open a browser and visit `http://localhost:8080`

## Generated Data Characteristics

### Operating System Distribution
- Ubuntu 20.04 LTS / 22.04 LTS
- CentOS 7 / 8
- RHEL 8 / 9
- Debian 11 / 12
- Rocky Linux 8 / 9

### Server Vendors and Models
- **Dell**: PowerEdge R740, R750, R640, R650
- **HP**: ProLiant DL380, DL360, DL385, ML350
- **Lenovo**: ThinkSystem SR650, SR630, SR850, SR950
- **Supermicro**: SuperServer 1029P, 2029P, 6029P, 8029P
- **ASUS**: ESC4000A-E10, ESC8000A-E11, RS720A-E11, RS500A-E10
- **Gigabyte**: G292-Z20, G482-Z50, G292-Z40, R282-Z90

### Hostname Naming Rules
- Prefixes: gpu, cpu, storage, compute, ai, ml, hpc
- Format: `{prefix}-node-{number:03d}`
- Example: `gpu-node-001`, `compute-node-042`

### Network Configuration
- IP Address Range: 10.x.x.x (Private Network)
- Randomly assigned to avoid conflicts

### Online Status
- 80% of machines are online (active within the last 5 minutes)
- 20% of machines are offline (last online between 5 minutes and 7 days ago)

## Troubleshooting

### Common Issues

1. **Cannot Connect to API Service**
   - Check if the backend service is running
   - Confirm port 8080 is not occupied
   - Check firewall settings

2. **Import Failed**
   - Check database connection
   - Confirm API interface is working properly
   - Check backend service logs

3. **Python Dependency Issues**
   ```bash
   pip install requests
   ```

### Custom Configuration

If you need to modify the API address or other configurations, please edit the configuration section in `import_test_data.py`:

```python
# Configuration
api_base_url = "http://localhost:8080/api/v1"  # Modify to actual API address
test_data_file = "test_clients.json"
```

## Notes

1. **Data Cleanup**: Before importing test data, it is recommended to backup existing data
2. **Performance Considerations**: Data for 100 machines is suitable for a test environment; production environments may require more data
3. **Data Consistency**: Each run of the generation script creates new random data
4. **Security**: Test data is for development and testing only and does not contain sensitive information

## Extended Functionality

If you need to generate more or different types of test data, you can modify the configuration in `generate_test_data.py`:

- Modify the number of machines
- Add new operating systems
- Add new vendors and models
- Adjust online/offline ratio
- Add more hardware information fields
