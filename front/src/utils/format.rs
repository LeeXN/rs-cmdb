use chrono::{DateTime, Local, Utc};

/// 将 ISO8601/RFC3339 时间格式化为本地时间字符串
pub fn format_datetime(iso_time: &str) -> String {
    match DateTime::parse_from_rfc3339(iso_time) {
        Ok(dt) => {
            let local_time = dt.with_timezone(&Local);
            local_time.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        Err(_) => "Unknown".to_string(),
    }
}

/// 格式化距今时间，例如"3小时前"
pub fn format_time_ago(iso_time: &str) -> String {
    match DateTime::parse_from_rfc3339(iso_time) {
        Ok(dt) => {
            let now = Utc::now();
            let dt_utc = dt.with_timezone(&Utc);
            let duration = now.signed_duration_since(dt_utc);

            if duration.num_days() > 30 {
                let months = duration.num_days() / 30;
                format!("{}个月前", months)
            } else if duration.num_days() > 0 {
                format!("{}天前", duration.num_days())
            } else if duration.num_hours() > 0 {
                format!("{}小时前", duration.num_hours())
            } else if duration.num_minutes() > 0 {
                format!("{}分钟前", duration.num_minutes())
            } else {
                "刚刚".to_string()
            }
        }
        Err(_) => iso_time.to_string(),
    }
}

/// 同时展示本地时间与相对时间
pub fn format_datetime_with_ago(iso_time: &str) -> String {
    let absolute = format_datetime(iso_time);
    if absolute == "Unknown" {
        return iso_time.to_string();
    }
    format!("{} ({})", absolute, format_time_ago(iso_time))
}

/// 格式化字节大小为人类可读格式
#[allow(dead_code)]
pub fn format_bytes(size_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size_bytes >= TB {
        format!("{:.2} TB", size_bytes as f64 / TB as f64)
    } else if size_bytes >= GB {
        format!("{:.2} GB", size_bytes as f64 / GB as f64)
    } else if size_bytes >= MB {
        format!("{:.2} MB", size_bytes as f64 / MB as f64)
    } else if size_bytes >= KB {
        format!("{:.2} KB", size_bytes as f64 / KB as f64)
    } else {
        format!("{} B", size_bytes)
    }
}

/// 格式化数字，添加千位分隔符
pub fn format_number(num: u64) -> String {
    let num_str = num.to_string();
    let mut result = String::new();
    let chars: Vec<char> = num_str.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*ch);
    }

    result
}

/// 格式化频率（Hz）
pub fn format_frequency(freq_hz: u64) -> String {
    const KHZ: u64 = 1_000;
    const MHZ: u64 = 1_000_000;
    const GHZ: u64 = 1_000_000_000;

    if freq_hz >= GHZ {
        format!("{:.2} GHz", freq_hz as f64 / GHZ as f64)
    } else if freq_hz >= MHZ {
        format!("{:.2} MHz", freq_hz as f64 / MHZ as f64)
    } else if freq_hz >= KHZ {
        format!("{:.2} KHz", freq_hz as f64 / KHZ as f64)
    } else {
        format!("{} Hz", freq_hz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_datetime_valid() {
        let result = format_datetime("2024-01-15T10:30:00+08:00");
        assert_eq!(result, "2024-01-15 10:30:00");
    }

    #[test]
    fn test_format_datetime_invalid() {
        assert_eq!(format_datetime(""), "Unknown");
        assert_eq!(format_datetime("not-a-date"), "Unknown");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
        assert_eq!(format_bytes(1099511627776), "1.00 TB");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(1), "1");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(1000000000), "1,000,000,000");
    }

    #[test]
    fn test_format_frequency() {
        assert_eq!(format_frequency(0), "0 Hz");
        assert_eq!(format_frequency(500), "500 Hz");
        assert_eq!(format_frequency(1000), "1.00 KHz");
        assert_eq!(format_frequency(1500), "1.50 KHz");
        assert_eq!(format_frequency(1000000), "1.00 MHz");
        assert_eq!(format_frequency(1000000000), "1.00 GHz");
    }
}
