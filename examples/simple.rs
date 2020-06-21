extern crate pca963x;
extern crate linux_embedded_hal as hal;

use hal::I2cdev;
use pca963x::{PCA9633_ADDR, PCA9633, Config};

fn main() {
    let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();

    let config = Config::default().sleep(false).all_call(false);


}