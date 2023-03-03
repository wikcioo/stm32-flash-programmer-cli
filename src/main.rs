use regex::Regex;
use serialport::available_ports;
use std::io;
use std::process::exit;

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
