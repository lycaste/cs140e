extern crate serial;
extern crate structopt;
extern crate xmodem;
#[macro_use] extern crate structopt_derive;

use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};
use serial::prelude::*;
use xmodem::Xmodem;
use std::io::{Read, Write};

mod parsers;

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout",
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn xmodem_send<I: Read, O: Write>(mut input: I, mut output: O)
    -> Result<u64, std::io::Error>
{
    Ok(128)
}

use std::io::{BufReader, BufRead};
fn t<I: BufRead>(mut input: I, mut serial: &mut serial::SerialPort, raw: bool)
    -> Result<u64, std::io::Error> {
    match raw {
        true => std::io::copy(&mut input, &mut serial),
        false => xmodem_send(input, serial)
    }
}

fn main() {
    use std::fs::File;

    let opt = Opt::from_args();

    let mut serial = serial::open(&opt.tty_path)
        .expect("path points to invalid TTY");

    serial.reconfigure(&|settings| {
        settings.set_baud_rate(opt.baud_rate)?;
        settings.set_char_size(opt.char_width);
        settings.set_stop_bits(opt.stop_bits);
        settings.set_flow_control(opt.flow_control);
        Ok(())
    }).expect("configure serial failed");
    serial::SerialPort::set_timeout(&mut serial, 
                                    Duration::from_millis(opt.timeout))
        .expect("configure timeout failed");

    let sent = match opt.input {
        Some(f) => {
            t(BufReader::new(File::open(f).expect("file open failed")), &mut serial, opt.raw)
        },
        None => t(BufReader::new(std::io::stdin()), &mut serial, opt.raw)
    };
    if let Ok(n) = sent {
        println!("wrote {} bytes to input", n);
    
    }
}
