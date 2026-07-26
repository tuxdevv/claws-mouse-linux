use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};

const VID: &str = "A8A5";
const PID: &str = "2255";

pub struct Profile {
    pub polling: u8,
    pub stage: u8,
    pub table: [u16; 6],
}

pub fn find_device() -> Result<String, String> {
    let entries = fs::read_dir("/sys/class/hidraw").map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let uevent_path = entry.path().join("device/uevent");
        let Ok(uevent) = fs::read_to_string(&uevent_path) else {
            continue;
        };
        let upper = uevent.to_uppercase();
        if !upper.contains(&format!("V0000{VID}P0000{PID}")) {
            continue;
        }
        if uevent.contains("input2") {
            let name = entry.file_name();
            return Ok(format!("/dev/{}", name.to_string_lossy()));
        }
    }
    Err("vendor HID interface not found (is the receiver plugged in?)".into())
}

fn transact(dev: &str, payload: &[u8]) -> Result<[u8; 64], String> {
    let mut buf = [0u8; 64];
    buf[..payload.len()].copy_from_slice(payload);

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(dev)
        .map_err(|e| e.to_string())?;
    file.write_all(&buf).map_err(|e| e.to_string())?;

    let mut reply = [0u8; 64];
    match file.read_exact(&mut reply) {
        Ok(_) => Ok(reply),
        Err(e) => Err(format!("no reply from device: {e}")),
    }
}

pub fn get_info(dev: &str) -> Result<String, String> {
    let reply = transact(dev, &[0x55, 0x03])?;
    let end = reply[8..].iter().position(|&b| b == 0).unwrap_or(reply.len() - 8);
    Ok(String::from_utf8_lossy(&reply[8..8 + end]).to_string())
}

pub fn get_profile(dev: &str) -> Result<Profile, String> {
    let reply = transact(dev, &[0x55, 0x0e])?;
    let mut table = [0u16; 6];
    for i in 0..6 {
        table[i] = u16::from_le_bytes([reply[13 + i * 2], reply[14 + i * 2]]);
    }
    Ok(Profile { polling: reply[10], stage: reply[12], table })
}

fn set_full(dev: &str, polling: u8, stage: u8, table: &[u16; 6]) -> Result<bool, String> {
    let mut payload = vec![0x55, 0x0f, 0xae, 0x0a, 0x2f, 0x01, 0x01, 0x01, 0x00, 0x03];
    payload.push(polling);
    payload.push(0x06);
    payload.push(stage);
    for v in table {
        payload.extend_from_slice(&v.to_le_bytes());
    }
    payload.extend(std::iter::repeat(0u8).take(24));
    payload.extend_from_slice(&[0xff, 0x01, 0x0a, 0x00, 0xff]);

    let reply = transact(dev, &payload)?;
    Ok(reply[0] == 0xAA)
}

pub fn set_dpi_stage(dev: &str, stage: u8) -> Result<bool, String> {
    if !(1..=6).contains(&stage) {
        return Err("stage must be 1-6".into());
    }
    let p = get_profile(dev)?;
    set_full(dev, p.polling, stage, &p.table)
}

pub fn set_dpi_value(dev: &str, stage: u8, value: u16) -> Result<bool, String> {
    if !(1..=6).contains(&stage) {
        return Err("stage must be 1-6".into());
    }
    if !(50..=12000).contains(&value) {
        return Err("DPI value must be 50-12000".into());
    }
    let p = get_profile(dev)?;
    let mut table = p.table;
    table[(stage - 1) as usize] = value;
    set_full(dev, p.polling, p.stage, &table)
}

pub fn set_polling(dev: &str, level: u8) -> Result<bool, String> {
    if !(1..=4).contains(&level) {
        return Err("polling level must be 1-4".into());
    }
    let p = get_profile(dev)?;
    set_full(dev, level, p.stage, &p.table)
}
