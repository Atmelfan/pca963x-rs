extern crate linux_embedded_hal as hal;
extern crate pca963x;

use hal::I2cdev;
use pca963x::{Address, Config, LedOut, PCA9633, PCA9633_ADDR, PCA963X};

fn main() {
    let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();

    let config = Config::default().sleep(false).all_call(false);

    let mut pca9633 = PCA9633::new_config(i2c_bus, Address::_8Pin, config);
    pca9633
        .write_out(PCA9633::Channels::_1, LedOut::FullyOn)
        .expect("Failed")
}
