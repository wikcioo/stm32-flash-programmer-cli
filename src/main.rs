use regex::Regex;
use serialport::{available_ports, ClearBuffer, SerialPort};
use std::io;
use std::process::exit;

const CMD_BL_GET_VER: u8 = 0xA1;
const CMD_BL_GET_HELP: u8 = 0xA2;
const CMD_BL_GET_DEV_ID: u8 = 0xA3;
const CMD_BL_GET_RDP_LEVEL: u8 = 0xA4;

const CMD_BL_GET_VER_LEN: u8 = 6;
const CMD_BL_GET_HELP_LEN: u8 = 6;
const CMD_BL_GET_DEV_ID_LEN: u8 = 6;
const CMD_BL_GET_RDP_LEVEL_LEN: u8 = 6;

fn main() {
    let serial_devices = get_available_serial_ports();

    if serial_devices.len() == 0 {
        println!("No available serial devices!");
        exit(0);
    }

    println!("Available serial devices:");
    for (index, name) in serial_devices.iter().enumerate() {
        println!("{index}: {name}");
    }

    println!("Enter the serial port name of your device: ");

    let mut serial_port_name = String::new();
    io::stdin().read_line(&mut serial_port_name).unwrap();
    let serial_port_name = serial_port_name.trim().to_string();

    if serial_devices.contains(&serial_port_name) {
        println!("Your device is {serial_port_name}");
    } else {
        println!("Bad device");
    }

    let mut port = serialport::new(serial_port_name, 115200)
        .open()
        .expect("Failed to open {serial_port_name}");
    port.set_timeout(std::time::Duration::from_secs(2)).unwrap();
    port.clear(ClearBuffer::Input).unwrap();

    loop {
        let cmd_number = choose_command();
        if cmd_number == 0 {
            break;
        }
        parse_command_number(cmd_number, port.as_mut());
        port.clear(ClearBuffer::Input).unwrap();
    }
}

fn u32_to_u8(number: u32, index: u32) -> u8 {
    (number >> (8 * (index - 1)) & 0xFF) as u8
}

fn get_crc(buff: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for data in buff {
        crc = crc ^ (*data as u32);
        for _ in 0..32 {
            if crc & 0x80000000 != 0 {
                crc = (crc << 1) ^ 0x04C11DB7;
            } else {
                crc = crc << 1;
            }
        }
    }

    crc
}

fn parse_command_number(number: i32, port: &mut dyn SerialPort) {
    let mut data_buffer = [0u8; 255];

    match number {
        1 => {
            data_buffer[0] = CMD_BL_GET_VER_LEN - 1;
            data_buffer[1] = CMD_BL_GET_VER;

            let data_upper_bound = (CMD_BL_GET_VER_LEN - 4) as usize;
            let crc32 = get_crc(&data_buffer[0..data_upper_bound]);
            data_buffer[2] = u32_to_u8(crc32, 1);
            data_buffer[3] = u32_to_u8(crc32, 2);
            data_buffer[4] = u32_to_u8(crc32, 3);
            data_buffer[5] = u32_to_u8(crc32, 4);
            port.write_all(&data_buffer[0..1]).unwrap();
            port.write_all(&data_buffer[1..(CMD_BL_GET_VER_LEN as usize)])
                .unwrap();

            process_bootloader_reply(data_buffer[1], port);
        }
        2 => {
            data_buffer[0] = CMD_BL_GET_HELP_LEN - 1;
            data_buffer[1] = CMD_BL_GET_HELP;

            let data_upper_bound = (CMD_BL_GET_HELP_LEN - 4) as usize;
            let crc32 = get_crc(&data_buffer[0..data_upper_bound]);
            data_buffer[2] = u32_to_u8(crc32, 1);
            data_buffer[3] = u32_to_u8(crc32, 2);
            data_buffer[4] = u32_to_u8(crc32, 3);
            data_buffer[5] = u32_to_u8(crc32, 4);
            port.write_all(&data_buffer[0..1]).unwrap();
            port.write_all(&data_buffer[1..(CMD_BL_GET_HELP_LEN as usize)])
                .unwrap();

            process_bootloader_reply(data_buffer[1], port);
        }
        3 => {
            data_buffer[0] = CMD_BL_GET_DEV_ID_LEN - 1;
            data_buffer[1] = CMD_BL_GET_DEV_ID;

            let data_upper_bound = (CMD_BL_GET_DEV_ID_LEN - 4) as usize;
            let crc32 = get_crc(&data_buffer[0..data_upper_bound]);
            data_buffer[2] = u32_to_u8(crc32, 1);
            data_buffer[3] = u32_to_u8(crc32, 2);
            data_buffer[4] = u32_to_u8(crc32, 3);
            data_buffer[5] = u32_to_u8(crc32, 4);
            port.write_all(&data_buffer[0..1]).unwrap();
            port.write_all(&data_buffer[1..(CMD_BL_GET_DEV_ID_LEN as usize)])
                .unwrap();

            process_bootloader_reply(data_buffer[1], port);
        }
        4 => {
            data_buffer[0] = CMD_BL_GET_RDP_LEVEL_LEN - 1;
            data_buffer[1] = CMD_BL_GET_RDP_LEVEL;

            let data_upper_bound = (CMD_BL_GET_RDP_LEVEL_LEN - 4) as usize;
            let crc32 = get_crc(&data_buffer[0..data_upper_bound]);
            data_buffer[2] = u32_to_u8(crc32, 1);
            data_buffer[3] = u32_to_u8(crc32, 2);
            data_buffer[4] = u32_to_u8(crc32, 3);
            data_buffer[5] = u32_to_u8(crc32, 4);
            port.write_all(&data_buffer[0..1]).unwrap();
            port.write_all(&data_buffer[1..(CMD_BL_GET_RDP_LEVEL_LEN as usize)])
                .unwrap();

            process_bootloader_reply(data_buffer[1], port);
        }
        _ => println!("Unsupported command number reached!"),
    }
}

fn process_bootloader_reply(command: u8, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; 2];
    port.read_exact(&mut rcv_buffer).unwrap();

    if rcv_buffer[0] == 0xBB {
        let reply_length = rcv_buffer[1] as usize;
        match command {
            CMD_BL_GET_VER => {
                process_cmd_bl_get_ver(reply_length, port);
            }
            CMD_BL_GET_HELP => {
                process_cmd_bl_get_help(reply_length, port);
            }
            CMD_BL_GET_DEV_ID => {
                process_cmd_bl_get_dev_id(reply_length, port);
            }
            CMD_BL_GET_RDP_LEVEL => {
                process_cmd_bl_get_rdp_level(reply_length, port);
            }
            _ => println!("Unknown bootloader command"),
        }
    } else if rcv_buffer[0] == 0xEE {
        println!("CRC verification failed!");
    } else {
        println!("Unknown reply!");
    }
}

fn process_cmd_bl_get_ver(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();
    println!("Bootloader version: 0x{:02X}", rcv_buffer[0]);
}

fn process_cmd_bl_get_help(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();
    print!("Bootloader available commands: ");
    for cmd in rcv_buffer {
        print!("0x{:02X} ", cmd);
    }
    println!();
}

fn process_cmd_bl_get_dev_id(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();
    let dev_id: u16 = (rcv_buffer[1] as u16) << 8 | rcv_buffer[0] as u16;
    println!("Bootloader device id: 0x{:04X}", dev_id);
}

fn process_cmd_bl_get_rdp_level(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();
    println!("Bootloader rdp level: 0x{:02X}", rcv_buffer[0]);
}

fn choose_command() -> i32 {
    display_menu();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let cmd_number: i32 = input.trim().parse().expect("Invalid input");
    cmd_number
}

fn display_menu() {
    println!("Choose a bootloader action");
    println!("GET VERSION => 1");
    println!("GET HELP    => 2");
    println!("GET DEV ID  => 3");
    println!("GET RDP LVL => 4");
    println!("QUIT        => 0");
}

fn get_available_serial_ports() -> Vec<String> {
    let pattern = Regex::new("/dev/tty[A-Za-z]*").unwrap();

    let ports = available_ports().unwrap();
    let mut available = Vec::new();
    for port in ports {
        let port_name = port.port_name;
        if pattern.is_match(&port_name) {
            available.push(port_name);
        }
    }

    available
}
