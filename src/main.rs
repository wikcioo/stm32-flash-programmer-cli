use regex::Regex;
use serialport::{available_ports, ClearBuffer, SerialPort};
use std::fs::read;
use std::io::{self, Write};
use std::process::exit;

struct BootloaderCommand {
    code: u8,
    length: u8,
}

const CMD_BL_GET_VER: BootloaderCommand = BootloaderCommand {
    code: 0xA1,
    length: 6,
};
const CMD_BL_GET_HELP: BootloaderCommand = BootloaderCommand {
    code: 0xA2,
    length: 6,
};
const CMD_BL_GET_DEV_ID: BootloaderCommand = BootloaderCommand {
    code: 0xA3,
    length: 6,
};
const CMD_BL_GET_RDP_LEVEL: BootloaderCommand = BootloaderCommand {
    code: 0xA4,
    length: 6,
};
const CMD_BL_JMP_ADDR: BootloaderCommand = BootloaderCommand {
    code: 0xA5,
    length: 10,
};
const CMD_BL_FLASH_ERASE: BootloaderCommand = BootloaderCommand {
    code: 0xA6,
    length: 8,
};
const CMD_BL_MEM_WRITE: BootloaderCommand = BootloaderCommand {
    code: 0xA7,
    length: 11,
};
const CMD_BL_MEM_READ: BootloaderCommand = BootloaderCommand {
    code: 0xA8,
    length: 11,
};
const CMD_BL_SET_RW_PROTECT: BootloaderCommand = BootloaderCommand {
    code: 0xA9,
    length: 8,
};
const CMD_BL_GET_RW_PROTECT: BootloaderCommand = BootloaderCommand {
    code: 0xAA,
    length: 6,
};

fn main() {
    start_program();
}

fn start_program() {
    display_program_name();

    let serial_devices = get_available_serial_ports();

    if serial_devices.is_empty() {
        eprintln!("No available serial devices!");
        exit(0);
    }

    println!("Available serial devices:");
    for (index, name) in serial_devices.iter().enumerate() {
        println!("{index}: {name}");
    }

    let mut port;
    print!("Choose your device from the list: ");
    io::stdout().flush().unwrap();

    loop {
        let mut serial_port_name = String::new();
        io::stdin().read_line(&mut serial_port_name).unwrap();
        let serial_port_name = serial_port_name.trim().to_string();

        if !serial_devices.contains(&serial_port_name) {
            eprintln!("'{serial_port_name}' not found in the list of available ports");
            print!("Try again: ");
            io::stdout().flush().unwrap();
            continue;
        }

        let serial_port_name = serial_port_name.trim().to_string();

        port = match serialport::new(&serial_port_name, 115200).open() {
            Ok(p) => p,
            Err(error) => {
                eprintln!("Failed to open {serial_port_name}: {}", error.description);
                print!("Try again: ");
                io::stdout().flush().unwrap();
                continue;
            }
        };

        break;
    }

    port.set_timeout(std::time::Duration::from_secs(2)).unwrap();
    port.clear(ClearBuffer::Input).unwrap();

    println!();
    display_available_commands();
    loop {
        let cmd = choose_command();
        parse_command(&cmd, port.as_mut());
        port.clear(ClearBuffer::Input).unwrap();
    }
}

fn u32_to_u8(number: u32, index: u32) -> u8 {
    (number >> (8 * (index - 1)) & 0xFF) as u8
}

fn get_crc(buff: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for data in buff {
        crc ^= *data as u32;
        for _ in 0..32 {
            if crc & 0x80000000 != 0 {
                crc = (crc << 1) ^ 0x04C11DB7;
            } else {
                crc <<= 1;
            }
        }
    }

    crc
}

fn calc_checksum_and_send(data: &mut [u8], port: &mut dyn SerialPort) {
    let cmd_len = data[0];
    let mut crc_buffer = [0u8; 4];
    // calculate crc on bytes [0 to CMD_BL_X_LEN - 4)
    // to properly calculate the crc, it expects the first byte to be
    // the length to follow which means we need to subtract one because
    // we don't count the length itself
    data[0] -= 1;
    let crc32 = get_crc(&data[0..(cmd_len - 4) as usize]);
    crc_buffer[0] = u32_to_u8(crc32, 1);
    crc_buffer[1] = u32_to_u8(crc32, 2);
    crc_buffer[2] = u32_to_u8(crc32, 3);
    crc_buffer[3] = u32_to_u8(crc32, 4);

    // append crc_buffer to data
    let data = [&data[0..(cmd_len - 4) as usize], &crc_buffer[..]].concat();

    port.write_all(&data[0..1]).unwrap();
    port.write_all(&data[1..((cmd_len) as usize)]).unwrap();
}

fn parse_command(cmd: &str, port: &mut dyn SerialPort) {
    let mut data_buffer = vec![0u8; 255];

    match cmd {
        "menu" => {
            display_available_commands();
            return;
        }
        "version" => {
            data_buffer[0] = CMD_BL_GET_VER.length;
            data_buffer[1] = CMD_BL_GET_VER.code;
        }
        "commands" => {
            data_buffer[0] = CMD_BL_GET_HELP.length;
            data_buffer[1] = CMD_BL_GET_HELP.code;
        }
        "dev_id" => {
            data_buffer[0] = CMD_BL_GET_DEV_ID.length;
            data_buffer[1] = CMD_BL_GET_DEV_ID.code;
        }
        "rdp" => {
            data_buffer[0] = CMD_BL_GET_RDP_LEVEL.length;
            data_buffer[1] = CMD_BL_GET_RDP_LEVEL.code;
        }
        "jmp" => {
            data_buffer[0] = CMD_BL_JMP_ADDR.length;
            data_buffer[1] = CMD_BL_JMP_ADDR.code;

            print!("Enter memory address to jump to in hex: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");
            let lowercase_input = input.to_lowercase();
            let input = lowercase_input.trim().trim_start_matches("0x");

            if let Ok(address_decimal) = u32::from_str_radix(input, 16) {
                data_buffer[2] = u32_to_u8(address_decimal, 1);
                data_buffer[3] = u32_to_u8(address_decimal, 2);
                data_buffer[4] = u32_to_u8(address_decimal, 3);
                data_buffer[5] = u32_to_u8(address_decimal, 4);
            } else {
                eprintln!("Invalid hex address!");
                return;
            }
        }
        "erase" => {
            data_buffer[0] = CMD_BL_FLASH_ERASE.length;
            data_buffer[1] = CMD_BL_FLASH_ERASE.code;

            let mut input = String::new();
            print!("Enter the sector number you want to start erasing from (0 to 7): ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let base_sector_number = input.trim().parse().expect("Invalid input");

            if base_sector_number > 7 {
                println!("Invalid sector number!");
                return;
            }

            const NUM_OF_FLASH_SECTORS: u8 = 8;

            let mut input = String::new();
            print!(
                "Enter the amount of sectors to erase starting from {base_sector_number} sector: "
            );
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let num_of_sectors_to_erase = input.trim().parse().expect("Invalid input");

            if num_of_sectors_to_erase > NUM_OF_FLASH_SECTORS - base_sector_number {
                println!(
                    "Can't erase {num_of_sectors_to_erase} sectors starting at {base_sector_number} sector!"
                );
                return;
            }

            data_buffer[2] = base_sector_number;
            data_buffer[3] = num_of_sectors_to_erase;
        }
        "write" => {
            data_buffer[1] = CMD_BL_MEM_WRITE.code;

            let mut input = String::new();
            print!("Enter filename: ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let filename = input.trim();

            let mut input = String::new();
            print!("Enter memory address at which to start writing: ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let input = input.trim().trim_start_matches("0x");
            let base_address = u32::from_str_radix(input, 16).expect("Invalid hex number");

            let mut bytes: Vec<u8> = read(filename).unwrap();

            let mut no_bytes_left_to_read = bytes.len();
            let single_byte_write_no: u8 = 128;
            let mut no_bytes_sent = 0;

            while no_bytes_left_to_read > 0 {
                let no_bytes_to_be_send = if no_bytes_left_to_read >= single_byte_write_no as usize
                {
                    single_byte_write_no
                } else {
                    no_bytes_left_to_read as u8
                };

                data_buffer[0] = CMD_BL_MEM_WRITE.length + no_bytes_to_be_send;
                data_buffer[2] = u32_to_u8(base_address + no_bytes_sent, 1);
                data_buffer[3] = u32_to_u8(base_address + no_bytes_sent, 2);
                data_buffer[4] = u32_to_u8(base_address + no_bytes_sent, 3);
                data_buffer[5] = u32_to_u8(base_address + no_bytes_sent, 4);

                data_buffer[6] = no_bytes_to_be_send;

                data_buffer[7..(no_bytes_to_be_send as usize + 7)]
                    .copy_from_slice(&bytes[..(no_bytes_to_be_send as usize)]);

                bytes = bytes[no_bytes_to_be_send as usize..].to_vec();

                calc_checksum_and_send(&mut data_buffer, port);
                process_bootloader_reply(data_buffer[1], port);

                no_bytes_sent += no_bytes_to_be_send as u32;
                no_bytes_left_to_read -= no_bytes_to_be_send as usize;
            }
            return;
        }
        "read" => {
            data_buffer[0] = CMD_BL_MEM_READ.length;
            data_buffer[1] = CMD_BL_MEM_READ.code;

            let mut input = String::new();
            print!("Enter memory address to start reading from (in hex): ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let input = input.trim().trim_start_matches("0x");
            let base_address = u32::from_str_radix(input, 16).expect("Invalid hex number");

            let mut input = String::new();

            print!("Enter how many bytes to read: ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            // TODO: Allow for bigger memory reads than u8
            let num_of_bytes_to_read: u8 = input.trim().parse().expect("Invalid input");

            data_buffer[2] = u32_to_u8(base_address, 1);
            data_buffer[3] = u32_to_u8(base_address, 2);
            data_buffer[4] = u32_to_u8(base_address, 3);
            data_buffer[5] = u32_to_u8(base_address, 4);

            data_buffer[6] = num_of_bytes_to_read;
        }
        "set_prot" => {
            data_buffer[0] = CMD_BL_SET_RW_PROTECT.length;
            data_buffer[1] = CMD_BL_SET_RW_PROTECT.code;

            let mut input = String::new();
            println!(
                "Enter which sectors you want to set protection (0 to 7) separated by space: "
            );
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let sector_numbers: Vec<&str> = input.trim().split(' ').collect();
            let sector_numbers: Vec<u8> =
                sector_numbers.iter().map(|x| x.parse().unwrap()).collect();
            let mut sectors = 0u8;

            for num in sector_numbers {
                if num > 7 {
                    println!("Incorrect sector values!");
                    return;
                }
                sectors |= 1 << (num);
            }

            let mut input = String::new();
            println!("Enter 1 for write or 2 for read/write: ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let prot_level = input.trim().parse().expect("Invalid input");

            if !(1..=2).contains(&prot_level) {
                println!("Incorrect protection level value!");
                return;
            }

            data_buffer[2] = sectors;
            data_buffer[3] = prot_level;
        }
        "get_prot" => {
            data_buffer[0] = CMD_BL_GET_RW_PROTECT.length;
            data_buffer[1] = CMD_BL_GET_RW_PROTECT.code;
        }
        "quit" => {
            exit(0);
        }
        "" => {
            return;
        }
        _ => {
            println!("Command '{cmd}' is not supported!");
            return;
        }
    }

    calc_checksum_and_send(&mut data_buffer, port);
    process_bootloader_reply(data_buffer[1], port);
}

fn process_bootloader_reply(command: u8, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; 2];
    port.read_exact(&mut rcv_buffer).unwrap();

    if rcv_buffer[0] == 0xBB {
        let reply_length = rcv_buffer[1] as usize;
        if command == CMD_BL_GET_VER.code {
            process_cmd_bl_get_ver(reply_length, port);
        } else if command == CMD_BL_GET_HELP.code {
            process_cmd_bl_get_help(reply_length, port);
        } else if command == CMD_BL_GET_DEV_ID.code {
            process_cmd_bl_get_dev_id(reply_length, port);
        } else if command == CMD_BL_GET_RDP_LEVEL.code {
            process_cmd_bl_get_rdp_level(reply_length, port);
        } else if command == CMD_BL_JMP_ADDR.code {
            process_cmd_bl_jmp_addr(reply_length, port);
        } else if command == CMD_BL_FLASH_ERASE.code {
            process_cmd_bl_flash_erase(reply_length, port);
        } else if command == CMD_BL_MEM_WRITE.code {
            process_cmd_bl_mem_write(reply_length, port);
        } else if command == CMD_BL_MEM_READ.code {
            process_cmd_bl_mem_read(reply_length, port);
        } else if command == CMD_BL_SET_RW_PROTECT.code {
            process_cmd_bl_set_rw_protect(reply_length, port);
        } else if command == CMD_BL_GET_RW_PROTECT.code {
            process_cmd_bl_get_rw_protect(reply_length, port);
        } else {
            println!("Unknown bootloader command");
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
        print!("0x{cmd:02X} ");
    }
    println!();
}

fn process_cmd_bl_get_dev_id(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();
    let dev_id: u16 = (rcv_buffer[1] as u16) << 8 | rcv_buffer[0] as u16;
    println!("Bootloader device id: 0x{dev_id:04X}");
}

fn process_cmd_bl_get_rdp_level(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();
    println!("Bootloader rdp level: 0x{:02X}", rcv_buffer[0]);
}

fn process_cmd_bl_jmp_addr(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();

    let result;
    if rcv_buffer[0] == 0 {
        result = "SUCCESS".to_string();
    } else if rcv_buffer[0] == 1 {
        result = "FAILURE".to_string();
    } else {
        result = "INVALID RESPONSE".to_string();
    }

    println!("Bootloader jump to address: {result}");
    if result == "SUCCESS" {
        exit(0);
    }
}

fn process_cmd_bl_flash_erase(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();

    let result;
    if rcv_buffer[0] == 0 {
        result = "SUCCESS".to_string();
    } else if rcv_buffer[0] == 1 {
        result = "FAILURE".to_string();
    } else {
        result = "INVALID RESPONSE".to_string();
    }

    println!("Bootloader flash erase: {result}");
}

fn process_cmd_bl_mem_write(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();

    if rcv_buffer[0] == 1 {
        println!("Bootloader memory write: SUCCESS");
    } else if rcv_buffer[0] == 0 {
        println!("Bootloader memory write: FAILURE");
    } else {
        println!("Bootloader memory write: INVALID RESPONSE");
    }
}

fn process_cmd_bl_mem_read(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();

    if rcv_buffer[0] == 1 {
        println!("Bootloader memory read: SUCCESS");
        println!("Memory content: ");
        for byte in rcv_buffer[1..].iter() {
            print!("0x{byte:02X} ");
        }
        println!();
    } else if rcv_buffer[0] == 0 {
        println!("Bootloader memory read: FAILURE");
    } else {
        println!("Bootloader memory read: INVALID RESPONSE");
    }
}

fn process_cmd_bl_set_rw_protect(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();

    let result;
    if rcv_buffer[0] == 1 {
        result = "SUCCESS".to_string();
    } else if rcv_buffer[0] == 0 {
        result = "FAILURE".to_string();
    } else {
        result = "INVALID RESPONSE".to_string();
    }

    println!("Bootloader set r/w protection: {result}");
}

fn process_cmd_bl_get_rw_protect(length: usize, port: &mut dyn SerialPort) {
    let mut rcv_buffer = vec![0u8; length];
    port.read_exact(&mut rcv_buffer).unwrap();

    println!("Bootloader get r/w protection: ");
    for (index, prot_level) in rcv_buffer.iter().enumerate() {
        let protection;
        if *prot_level == 0 {
            protection = "No protection";
        } else if *prot_level == 1 {
            protection = "Write protection";
        } else if *prot_level == 2 {
            protection = "Read and Write protection";
        } else {
            protection = "Unknown";
        }

        println!("sector nr {index}: {protection}");
    }
}

fn choose_command() -> String {
    let mut input = String::new();
    print!(">>> ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

fn display_program_name() {
    println!("#######################################");
    println!("#  STM32 FLASH PROGRAMMER CLI V0.1.0  #");
    println!("#######################################\n");
}

fn display_available_commands() {
    println!("Available commands:");
    println!("menu");
    println!("version");
    println!("commands");
    println!("dev_id");
    println!("rdp");
    println!("jmp");
    println!("erase");
    println!("write");
    println!("read");
    println!("set_prot");
    println!("get_prot");
    println!("quit");
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
