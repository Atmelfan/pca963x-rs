extern crate linux_embedded_hal as hal;
extern crate pca963x;

use hal::i2cdev::linux::LinuxI2CError;
use hal::I2cdev;
use pca963x::{Address, Channels4, Config, LedOut, PCA9633, PCA963X};

fn main() -> Result<(), LinuxI2CError> {
    let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();

    // Run mode, no all call address, blinking/grou freq enabled
    let config = Config::default().sleep(false).all_call(false).blink(true);

    let mut pca9633 = PCA9633::new_config(i2c_bus, Address::_8Pin, config)?;

    // Configure outputs
    pca9633.write_out(Channels4::_1, LedOut::FullyOn)?;
    pca9633.write_out(Channels4::_2, LedOut::FullyOff)?;
    pca9633.write_out(Channels4::_3, LedOut::Pwm)?;
    pca9633.write_out(Channels4::_4, LedOut::PwmGroup)?;

    // Set a blinking pattern
    pca9633.write_group_duty(128)?;
    pca9633.write_group_freq(128)?;

    // Set PWM registers
    pca9633.write_duty(Channels4::_3, 64)?;
    pca9633.write_duty(Channels4::_4, 192)?;

    Ok(())
}
