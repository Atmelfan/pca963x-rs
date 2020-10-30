#![no_std]
#![allow(non_upper_case_globals)]

extern crate bitflags;
extern crate embedded_hal as hal;

use bitflags::bitflags;
use hal::blocking::i2c;

#[derive(Copy, Clone, Debug)]
pub enum Address {
    /// 8 pin package, fixed address of 0x62
    _8Pin,
    /// 10 pin package with A0 and A1 pins
    _10Pin {
        a0: bool,
        a1: bool,
    },
    /// 16 pin package with A0-A6 pins
    _16Pin {
        a0: bool,
        a1: bool,
        a2: bool,
        a3: bool,
        a4: bool,
        a5: bool,
        a6: bool,
    },
    /// Custom address determined programmatically
    Custom(u8),
}

impl Address {
    pub fn address(self) -> u8 {
        match self {
            Address::_8Pin => 0x62u8,
            Address::_10Pin { a0, a1 } => 0x60 | (a0 as u8) << 0 | (a1 as u8) << 1,
            Address::_16Pin {
                a0,
                a1,
                a2,
                a3,
                a4,
                a5,
                a6,
            } => {
                0x00u8
                    | (a0 as u8) << 0
                    | (a1 as u8) << 1
                    | (a2 as u8) << 2
                    | (a3 as u8) << 3
                    | (a4 as u8) << 4
                    | (a5 as u8) << 5
                    | (a6 as u8) << 6
            }
            Address::Custom(addr) => addr,
        }
    }
}

bitflags! {
    pub struct Mode1: u8 {
        const Sleep     = 0b0001_0000;
        const Sub1      = 0b0000_1000;
        const Sub2      = 0b0000_0100;
        const Sub3      = 0b0000_0010;
        const AllCall   = 0b0000_0001;
    }
}

bitflags! {
    pub struct Mode2: u8 {
        const DmBlink   = 0b0010_0000;
        const Invert    = 0b0001_0000;
        const Och       = 0b0000_1000;
        const OutDrv    = 0b0000_0100;
        const OutNe1    = 0b0000_0010;
        const OutNe0    = 0b0000_0001;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LedOut {
    /// LED is fully off
    FullyOff,
    /// LED id fully on
    FullyOn,
    /// LED brightness is controlled through its PWMx
    Pwm,
    /// LED brightness is controlled through its PWMx and group duty/blinking.
    PwmGroup,
}

pub trait Channels {
    fn get_offs(self) -> u8;
}

#[derive(Copy, Clone, Debug)]
pub enum Channels4 {
    _1 = 0,
    _2 = 1,
    _3 = 2,
    _4 = 3,
}

impl Channels for Channels4 {
    fn get_offs(self) -> u8 {
        self as u8
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Channels8 {
    _1 = 0,
    _2 = 1,
    _3 = 2,
    _4 = 3,
    _5 = 4,
    _6 = 5,
    _7 = 6,
    _8 = 7,
}

impl Channels for Channels8 {
    fn get_offs(self) -> u8 {
        self as u8
    }
}

#[derive(Copy, Clone, Debug)]
pub enum OutputDrive {
    /// When OE = 1 (output drivers not enabled), LEDn = 0
    OutNe00 = 0x00,
    /// When OE = 1 (output drivers not enabled):
    /// * LEDn = 1 when OUTDRV = 1
    /// * LEDn = high-impedance when OUTDRV = 0 (same as OUTNE[1:0] = 10)
    OutNe01 = 0x01,
    /// When OE = 1 (output drivers not enabled), LEDn = high-impedance
    OutNe10 = 0x02,
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
    mode1: Mode1,
    mode2: Mode2,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mode1: Mode1::AllCall | Mode1::Sleep,
            mode2: Mode2::OutDrv | Mode2::OutNe0,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Config {
            mode1: Mode1::AllCall,
            mode2: Mode2::OutDrv | Mode2::OutNe0,
        }
    }

    pub fn sub1(&mut self, enable: bool) -> &mut Config {
        self.mode1.set(Mode1::Sub1, enable);
        self
    }

    pub fn sub2(&mut self, enable: bool) -> &mut Config {
        self.mode1.set(Mode1::Sub2, enable);
        self
    }

    pub fn sub3(&mut self, enable: bool) -> &mut Config {
        self.mode1.set(Mode1::Sub3, enable);
        self
    }

    pub fn all_call(&mut self, enable: bool) -> &mut Config {
        self.mode1.set(Mode1::AllCall, enable);
        self
    }

    pub fn sleep(&mut self, enable: bool) -> &mut Config {
        self.mode1.set(Mode1::Sleep, enable);
        self
    }

    pub fn blink(&mut self, enable: bool) -> &mut Config {
        self.mode2.set(Mode2::DmBlink, enable);
        self
    }

    pub fn invert(&mut self, enable: bool) -> &mut Config {
        self.mode2.set(Mode2::Invert, enable);
        self
    }

    pub fn out_drv(&mut self, enable: bool) -> &mut Config {
        self.mode2.set(Mode2::OutDrv, enable);
        self
    }

    pub fn outne(&mut self, out: OutputDrive) -> &mut Config {
        match out {
            OutputDrive::OutNe00 => self.mode2.remove(Mode2::OutNe1 | Mode2::OutNe0),
            OutputDrive::OutNe01 => {
                self.mode2.insert(Mode2::OutNe0);
                self.mode2.remove(Mode2::OutNe0);
            }
            OutputDrive::OutNe10 => {
                self.mode2.remove(Mode2::OutNe0);
                self.mode2.insert(Mode2::OutNe0);
            }
        }
        self
    }
}

//const AUTOINCR_NONE: u8     = 0b0000_0000;
const AUTOINCR_ALL: u8 = 0b1000_0000;
//const AUTOINCR_BRIGHT: u8 = 0b1010_0000;
//const AUTOINCR_GLOBAL: u8 = 0b1100_0000;
//const AUTOINCR_GLBR: u8 = 0b1110_0000;

pub trait PCA963X<I2C, E>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
    const MODE1: u8;
    const MODE2: u8;
    const PWM0: u8;
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
        self.write(Self::PWM0 + ch.get_offs(), value)
    }

    /// Write channel output mode
    fn write_out(&mut self, ch: Self::Channels, out: LedOut) -> Result<(), E> {
        let offs = ch.get_offs();
        let mut ledout = self.read(Self::LEDOUT1 + (offs / 4u8))?;
        ledout &= 0x03 << (offs % 4u8) * 2;
        ledout |= (out as u8) << (offs % 4u8) * 2;
        self.write(Self::LEDOUT1 + (offs / 4u8), ledout)
    }

    ///// Write channel output mode to all outputs
    //fn write_all_out(&mut self, out: LedOut) -> Result<(), E>;

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

macro_rules! device {
    ($name:ident, $channels:ident => $($reg:ident = $val:expr);*) => {
        pub struct $name<I2C> {
            i2c: I2C,
            address: u8
        }

        impl<I2C, E> PCA963X<I2C, E> for $name<I2C>
            where I2C: i2c::Write<Error = E> + i2c::Read<Error = E> {

            $(
                const $reg : u8 = $val;
            )*

            type Channels = $channels;

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

        impl<I2C, E> $name<I2C>
            where I2C: i2c::Write<Error = E> + i2c::Read<Error = E>{

            pub fn new(i2c: I2C, address: Address) -> Self {
                $name {
                    i2c,
                    address: address.address()
                }
            }

            pub fn new_config(i2c: I2C, address: Address, conf: Config) -> Result<Self, E> {
                let mut pca = Self::new(i2c, address);
                pca.write_config(conf)?;
                Ok(pca)
            }
        }
    };
}

device!(PCA9633, Channels4 =>
    MODE1 = 0x00;
    MODE2 = 0x01;
    PWM0  = 0x02;
    //PWM1 = 0x03;
    //PWM2 = 0x04;
    //PWM3 = 0x05;
    GRPPWM = 0x06;
    GRPFREQ = 0x07;
    LEDOUT1 = 0x08;
    SUBADR1 = 0x09;
    SUBADR2 = 0x0A;
    SUBADR3 = 0x0B;
    ALLCALLADR = 0x0C
);

device!(PCA9634, Channels8 =>
    MODE1 = 0x00;
    MODE2 = 0x01;
    PWM0 = 0x02;
    //PWM1 = 0x03;
    //PWM2 = 0x04;
    //PWM3 = 0x05;
    //PWM4 = 0x06;
    //PWM5 = 0x07;
    //PWM6 = 0x08;
    //PWM7 = 0x09;
    GRPPWM = 0x0A;
    GRPFREQ = 0x0B;
    LEDOUT1 = 0x0C;
    //LEDOUT2 = 0x0D;
    SUBADR1 = 0x0E;
    SUBADR2 = 0x0F;
    SUBADR3 = 0x10;
    ALLCALLADR = 0x11
);
