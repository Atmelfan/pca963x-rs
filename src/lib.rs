#![no_std]

extern crate bitflags;
extern crate embedded_hal as hal;

use bitflags::bitflags;
use hal::blocking::i2c;

pub const PCA9633_ADDR: u8 = 0x60;

bitflags! {
    struct Mode1: u8 {
        const Sleep     = 0b0001_0000;
        const Sub1      = 0b0000_1000;
        const Sub2      = 0b0000_0100;
        const Sub3      = 0b0000_0010;
        const AllCall   = 0b0000_0001;
    }
}

bitflags! {
    struct Mode2: u8 {
        const DmBlink   = 0b0010_0000;
        const Invert    = 0b0001_0000;
        const Och       = 0b0000_1000;
        const OutDrv    = 0b0000_0100;
        const OutNe1    = 0b0000_0010;
        const OutNe0    = 0b0000_0001;
    }
}

enum LedOut {
    /// LED is fully off
    FullyOff,
    /// LED id fully on
    FullyOn,
    /// LED brightness is controlled through its PWMx
    Pwm,
    /// LED brightness is controlled through its PWMx and group duty/blinking.
    PwmGroup
}

trait Channels {
    fn get_offs(self) -> u8;
}

enum Channels4 {
    _1 = 0,
    _2 = 1,
    _3 = 2,
    _4 = 3
}
impl Channels for Channels4 {
    fn get_offs(self) -> u8 {
        self as u8
    }
}

enum Channels8 {
    _1 = 0,
    _2 = 1,
    _3 = 2,
    _4 = 3,
    _5 = 4,
    _6 = 5,
    _7 = 6,
    _8 = 7
}
impl Channels for Channels8 {
    fn get_offs(self) -> u8 {
        self as u8
    }
}

pub struct Config {
    mode1: Mode1,
    mode2: Mode2
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mode1: Mode1::AllCall | Mode1::Sleep,
            mode2: Mode2::OutDrv | Mode2::OutNe0
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Config {
            mode1: Mode1::AllCall,
            mode2: Mode2::OutDrv | Mode2::OutNe0
        }
    }

    pub fn sub1(&mut self, enable: bool) -> &mut Config {
        self.mode1 |= Mode1::Sub1;
        self
    }

    pub fn sub2(&mut self, enable: bool) -> &mut Config {
        self.mode1 |= Mode1::Sub2;
        self
    }

    pub fn sub3(&mut self, enable: bool) -> &mut Config {
        self.mode1 |= Mode1::Sub3;
        self
    }

    pub fn all_call(&mut self, enable: bool) -> &mut Config {
        self.mode1 |= Mode1::AllCall;
        self
    }

    pub fn sleep(&mut self, enable: bool) -> &mut Config {
        self.mode1 |= Mode1::Sleep;
        self
    }
}

const AUTOINCR_NONE: u8     = 0b0000_0000;
const AUTOINCR_ALL: u8      = 0b1000_0000;
const AUTOINCR_BRIGHT: u8   = 0b1010_0000;
const AUTOINCR_GLOBAL: u8   = 0b1100_0000;
const AUTOINCR_GLBR: u8     = 0b1110_0000;

trait PCA963X<I2C, E>
    where I2C: i2c::Write<Error = E> + i2c::Read<Error = E> {

    const MODE1: u8;
    const MODE2: u8;
    const PWM1: u8;
    const GRPPWM: u8;
    const GRPFREQ: u8;
    const LEDOUT1: u8;
    const SUBADR1: u8;
    const SUBADR2: u8;
    const SUBADR3: u8;
    const ALLCALLADR: u8;

    type Channels: Channels;

    /// Read a register
    fn read(&mut self, register: u8) -> Result<u8, E>;

    /// Write a register
    fn write(&mut self, register: u8, value: u8) -> Result<(), E>;

    /// Write config
    fn write_config(&mut self, conf: Config) -> Result<(), E>;

    /// Write channel pwm
    fn write_duty(&mut self, ch: Self::Channels, value: u8) -> Result<(), E> {
        self.write(AUTOINCR_NONE | Self::PWM1 + ch.get_offs(), value)
    }

    /// Write channel output mode
    fn write_out(&mut self, ch: Self::Channels, out: LedOut) -> Result<(), E> {
        let offs = ch.get_offs();
        let mut ledout = self.read(Self::LEDOUT1 + (offs / 4u8))?;
        ledout &= 0x03 << (offs % 4u8)*2;
        ledout |= (out as u8) << (offs % 4u8)*2;
        self.write(Self::LEDOUT1 + (offs/ 4u8), ledout)
    }

    /// Write group duty cycle
    fn write_group_duty(&mut self, value: u8) -> Result<(), E> {
        self.write(Self::GRPPWM, value)
    }
    /// Write group frequency. Not used if `DmBlink` flag is not set in config.
    fn write_group_freq(&mut self, value: u8) -> Result<(), E> {
        self.write(Self::GRPFREQ, value)
    }

    /// Write sub address 1. Requires `Sub1` flag in config to be set.
    fn write_sub_address1(&mut self, addr: u8) -> Result<(), E> {
        self.write(Self::SUBADR1, addr << 1)
    }

    /// Write sub address 2. Requires `Sub2` flag in config to be set.
    fn write_sub_address2(&mut self, addr: u8) -> Result<(), E> {
        self.write(Self::SUBADR2, addr << 1)
    }

    /// Write sub address 3. Requires `Sub3` flag in config to be set.
    fn write_sub_address3(&mut self, addr: u8) -> Result<(), E> {
        self.write(Self::SUBADR3, addr << 1)
    }

    /// Write all call address. Requires `AllCall` flag in config to be set.
    fn write_all_call_address1(&mut self, addr: u8) -> Result<(), E> {
        self.write(Self::ALLCALLADR, addr << 1)
    }
}

pub struct PCA9633<I2C> {
    i2c: I2C,
    address: u8
}

impl<I2C, E> PCA963X<I2C, E> for PCA9633<I2C>
    where I2C: i2c::Write<Error = E> + i2c::Read<Error = E> {
    const MODE1: u8 = 0x00;
    const MODE2: u8 = 0x01;
    const PWM1: u8  = 0x03;
    const GRPPWM: u8 = 0x06;
    const GRPFREQ: u8 = 0x07;
    const LEDOUT1: u8 = 0x08;
    const SUBADR1: u8 = 0x09;
    const SUBADR2: u8 = 0x0A;
    const SUBADR3: u8 = 0x0B;
    const ALLCALLADR: u8 = 0x0C;

    type Channels = Channels4;

    fn read(&mut self, register: u8) -> Result<u8, E> {
        let mut buf = [0u8];
        self.i2c.write(self.address, &[register])?;
        self.i2c.read(self.address, &mut buf)?;
        Ok(buf[0])
    }

    fn write(&mut self, register: u8, value: u8) -> Result<(), E> {
        self.i2c.write(self.address, &[register, value])
    }

    fn write_config(&mut self, conf: Config) -> Result<(), E> {
        self.i2c.write(self.address, &[AUTOINCR_ALL | Self::MODE1, conf.mode1.bits, conf.mode2.bits])
    }
}

impl<I2C, E> PCA9633<I2C>
    where I2C: i2c::Write<Error = E> + i2c::Read<Error = E>{

    pub fn new(i2c: I2C, address: u8) -> Self {
        PCA9633 {
            i2c,
            address
        }
    }

    pub fn new_config(i2c: I2C, address: u8, conf: Config) -> Result<Self, E> {
        let mut pca = Self::new(i2c, address);
        pca.write_config(conf)?;
        Ok(pca)
    }
}