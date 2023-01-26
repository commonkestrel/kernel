#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use kernel::serial_println;
use kernel::graphics::{ vga, colors };
use core::panic::PanicInfo;

// Panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}

//Various

// Main
#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("hello");

    let screen = vga::init();
    screen.clear(colors::rgb(30, 30, 30));
    
    screen.debug(6);
    screen.line(5, 1, 100, 100, 200, 110);
    screen.circle(9, 200, 40, 30);
    serial_println!("finished");
    //screen.dilation(colors::rgb(70, 220, 160), 100, 100, 5);

    #[cfg(test)]
    test_main();

    loop {}
}
