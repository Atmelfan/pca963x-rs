#![no_std]
#![allow(non_upper_case_globals)]

extern crate bitflags;
extern crate embedded_hal as hal;

use bitflags::bitflags;
use hal::blocking::i2c;

#[cfg(feature = "embedded-hal-pwm")]
use hal::Pwm;

#[derive(Copy, Clone, Debug)]
pub enum Address {
    /// 8 pin package, fixed address of 0x62
    _8Pin,
    /// 10 pin package with A0 and A1 pins
    _10Pin { a0: bool, a1: bool },
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
            Address::_10Pin { a0, a1 } => 0x60 | (a0 as u8) | (a1 as u8) << 1,
            Address::_16Pin {
                a0,
                a1,
                a2,
                a3,
                a4,
                a5,
                a6,
            } => {
                (a0 as u8)
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

/// Internal trait
pub trait Channels {
    fn get_offs(self) -> u8;
}

/// 4 channels
#[derive(Copy, Clone, Debug)]
pub enum Channels4 {
    /// Channel 1
    _1 = 0,
    /// Channel 2
    _2 = 1,
    /// Channel 3
    _3 = 2,
    /// Channel 4
    _4 = 3,
}

impl Channels for Channels4 {
    fn get_offs(self) -> u8 {
        self as u8
    }
}

/// 8 channels
#[derive(Copy, Clone, Debug)]
pub enum Channels8 {
    /// Channel 1
    _1 = 0,
    /// Channel 2
    _2 = 1,
    /// Channel 3
    _3 = 2,
    /// Channel 4
    _4 = 3,
    /// Channel 5
    _5 = 4,
    /// Channel 6
    _6 = 5,
    /// Channel 7
    _7 = 6,
    /// Channel 8
    _8 = 7,
}

impl Channels for Channels8 {
    fn get_offs(self) -> u8 {
        self as u8
    }
}

/// Output drive mode
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

/// Output change mode
#[derive(Copy, Clone, Debug)]
pub enum Och {
    /// Outputs change on STOP command.
    ChangeOnStop,
    /// Outputs change on ACK.
    ChangeOnAck,
}

// Output change mode
#[derive(Copy, Clone, Debug)]
pub enum OutDrv {
    /// The 4 LED outputs are configured with an open-drain structure.
    OpenDrain,
    /// The 4 LED outputs are configured with a totem pole structure.
    TotemPole,
}

/// Driver configuration registers
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
    /// Default configs but with sleep-mode disabled
    pub fn new() -> Self {
        Config {
            mode1: Mode1::AllCall,
            mode2: Mode2::OutDrv | Mode2::OutNe0,
        }
    }

    /// Enable subaddress 1
    pub fn sub1(mut self, enable: bool) -> Config {
        self.mode1.set(Mode1::Sub1, enable);
        self
    }

    /// Enable subaddress 2
    pub fn sub2(mut self, enable: bool) -> Config {
        self.mode1.set(Mode1::Sub2, enable);
        self
    }

    /// Enable subaddress 3
    pub fn sub3(mut self, enable: bool) -> Config {
        self.mode1.set(Mode1::Sub3, enable);
        self
    }

    /// Enable all call address
    pub fn all_call(mut self, enable: bool) -> Config {
        self.mode1.set(Mode1::AllCall, enable);
        self
    }

    /// Put into sleep mode (default on)
    pub fn sleep(mut self, enable: bool) -> Config {
        self.mode1.set(Mode1::Sleep, enable);
        self
    }

    /// Determines the function of group control registers (see datasheet).
    pub fn blink(mut self, enable: bool) -> Config {
        self.mode2.set(Mode2::DmBlink, enable);
        self
    }

    /// Output logic inverted
    pub fn invert(mut self, enable: bool) -> Config {
        self.mode2.set(Mode2::Invert, enable);
        self
    }

    /// Control when outputs are updated
    pub fn och(mut self, change: Och) -> Config {
        match change {
            Och::ChangeOnStop => self.mode2.set(Mode2::Och, false),
            Och::ChangeOnAck => self.mode2.set(Mode2::Och, true),
        }
        self
    }

    /// Control output driver structure
    pub fn out_drv(mut self, outdrv: OutDrv) -> Config {
        match outdrv {
            OutDrv::OpenDrain => self.mode2.set(Mode2::OutDrv, false),
            OutDrv::TotemPole => self.mode2.set(Mode2::OutDrv, true),
        }
        self
    }

    /// Control output idle behaviour
    pub fn outne(mut self, out: OutputDrive) -> Config {
        match out {
            OutputDrive::OutNe00 => self.mode2.remove(Mode2::OutNe1 | Mode2::OutNe0),
            OutputDrive::OutNe01 => {
                self.mode2.insert(Mode2::OutNe0);
                self.mode2.remove(Mode2::OutNe1);
            }
            OutputDrive::OutNe10 => {
                self.mode2.remove(Mode2::OutNe0);
                self.mode2.insert(Mode2::OutNe1);
            }
        }
        self
    }
}

#[cfg(test)]
mod test_config {
    use super::*;

    #[test]
    fn test_default() {
        let config: Config = Default::default();
        assert_eq!(config.mode1.bits, 0b0001_0001); // Per 7.3.1
        assert_eq!(config.mode2.bits, 0b0000_0101); // Per 7.3.2
    }

    #[test]
    fn test_sub() {
        let config: Config = Config::default().sub1(true).sub2(true).sub3(true);
        assert_eq!(config.mode1.bits, 0b0001_1111); // Per 7.3.1
    }

    #[test]
    fn test_all_call() {
        let config: Config = Config::default().all_call(false);
        assert_eq!(config.mode1.bits, 0b0001_0000); // Per 7.3.1
        let config2: Config = Config::default().all_call(true);
        assert_eq!(config2.mode1.bits, 0b0001_0001); // Per 7.3.1
    }

    #[test]
    fn test_sleep() {
        let config: Config = Config::default().sleep(false);
        assert_eq!(config.mode1.bits, 0b0000_0001); // Per 7.3.1
    }

    #[test]
    fn test_blink() {
        let config: Config = Config::default().blink(true);
        assert_eq!(config.mode2.bits, 0b0010_0101); // Per 7.3.1
    }

    #[test]
    fn test_invert() {
        let config: Config = Config::default().invert(true);
        assert_eq!(config.mode2.bits, 0b0001_0101); // Per 7.3.1
    }

    #[test]
    fn test_och() {
        let config: Config = Config::default().och(Och::ChangeOnAck);
        assert_eq!(config.mode2.bits, 0b0000_1101); // Per 7.3.1
    }

    #[test]
    fn test_outdrv() {
        let config: Config = Config::default().out_drv(OutDrv::OpenDrain);
        assert_eq!(config.mode2.bits, 0b0000_0001); // Per 7.3.1
    }

    #[test]
    fn test_outne() {
        let mut config: Config = Config::default().outne(OutputDrive::OutNe00);
        assert_eq!(config.mode2.bits, 0b0000_0100); // Per 7.3.1
        config = config.outne(OutputDrive::OutNe01);
        assert_eq!(config.mode2.bits, 0b0000_0101); // Per 7.3.1
        config = config.outne(OutputDrive::OutNe10);
        assert_eq!(config.mode2.bits, 0b0000_0110); // Per 7.3.1
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

    fn read_duty(&mut self, ch: Self::Channels) -> Result<u8, E> {
        self.read(Self::PWM0 + ch.get_offs())
    }

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
        ledout &= 0x03 << ((offs % 4u8) * 2);
        ledout |= (out as u8) << ((offs % 4u8) * 2);
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

            /// New LED driver
            ///
            /// *Note: Does not take driver out of __sleep__ mode*
            pub fn new(i2c: I2C, address: Address) -> Self {
                $name {
                    i2c,
                    address: address.address()
                }
            }

            /// New LED driver
            pub fn new_config(i2c: I2C, address: Address, conf: Config) -> Result<Self, E> {
                let mut pca = Self::new(i2c, address);
                pca.write_config(conf)?;
                Ok(pca)
            }
        }

        #[cfg(feature="embedded-hal-pwm")]
        impl<I2C, E, T: PCA963X<I2C, E>> hal::Pwm for T
        where
            I2C: i2c::Write<Error = E> + i2c::Read<Error = E>
        {
            type Channel = T::Channels;
            type Time = ();
            type Duty = u8;

            fn disable(&mut self, channel: Self::Channel) {
                self.write_out(channel, LedOut::FullyOff).unwrap_or_default()
            }

            fn enable(&mut self, channel: Self::Channel) {
                self.write_out(channel, LedOut::Pwm).unwrap_or_default()
            }

            fn get_period(&self) -> Self::Time {
                ()
            }

            fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
                //self.read_duty(channel).unwrap_or(0)
                0
            }

            fn get_max_duty(&self) -> Self::Duty {
                255u8
            }

            fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
                self.write_duty(channel, duty).unwrap_or_default()
            }

            fn set_period<P>(&mut self, _period: P) where
                P: Into<Self::Time> {

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
