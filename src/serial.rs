use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        // 0x3f8は最初のシリアルインターフェースの標準のポート番号
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1
        .lock()
        .write_fmt(args)
        .expect("Printing to serial failed");
}

/// serial interfaceを通じてホストに出力する
#[macro_export]
macro_rules! serial_print{
    ($($args:tt)*) => {$crate::serial::_print(format_args!($($args)*));
    };
}

/// serial interfaceを通じてホストに出力し，改行を末尾についかする
#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n")
    };
    ($fmt:expr) => {
        $crate::serial_print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
            concat!($fmt, "\n"), $($arg)*));
}
