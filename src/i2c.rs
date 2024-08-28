use crate::config::Config;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait I2CInput: Send + Sync {
    fn read(&self);
    fn state(&self) -> bool;
}

pub trait I2COutput: Send + Sync {
    fn write(&mut self);
    fn set_at(&mut self, pos: usize, value: bool);
    fn is_dirty(&self) -> bool;
}

struct InputPin {
    input: Arc<Mutex<dyn I2CInput>>,
    pos: usize,
}

struct OutputPin {
    output: Arc<Mutex<dyn I2COutput>>,
    pos: usize,
}

pub struct I2CDeviceTree {
    pub inputs: Vec<Arc<Mutex<dyn I2CInput>>>,
    pub outputs: Vec<Arc<Mutex<dyn I2COutput>>>,
    in_pin_mapping: HashMap<String, InputPin>,
    out_pin_mapping: HashMap<String, OutputPin>,
}

impl I2CDeviceTree {
    pub fn new(config: &Config) -> Self {
        let mut result = I2CDeviceTree {
            inputs: Vec::new(),
            outputs: Vec::new(),
            in_pin_mapping: HashMap::new(),
            out_pin_mapping: HashMap::new(),
        };
        for input in &config.ios.inputs {
            let device: Arc<Mutex<dyn I2CInput>> = match input.chip.as_str() {
                "PCF8574" => Arc::new(Mutex::new(PCF8574::new(input.address))),
                _ => {
                    println!("Unknown chip '{}'", input.chip);
                    continue;
                }
            };
            for (p, pin) in input.pins.iter().enumerate() {
                result.in_pin_mapping.insert(
                    pin.clone(),
                    InputPin {
                        input: Arc::clone(&device),
                        pos: p,
                    },
                );
            }
            result.inputs.push(device);
        }
        for output in &config.ios.outputs {
            let device: Arc<Mutex<dyn I2COutput>> = match output.chip.as_str() {
                "PCF8574" => Arc::new(Mutex::new(PCF8574::new(output.address))),
                _ => continue,
            };
            for (p, pin) in output.pins.iter().enumerate() {
                result.out_pin_mapping.insert(
                    pin.clone(),
                    OutputPin {
                        output: Arc::clone(&device),
                        pos: p,
                    },
                );
            }
            result.outputs.push(device);
        }
        result
    }

    pub fn update(&self, output_name: &str, value: bool) {
        if let Some(output_pin) = self.out_pin_mapping.get(output_name) {
            let mut output_guard = output_pin.output.lock().unwrap();
            output_guard.set_at(output_pin.pos, value);
        } else {
            eprintln!("Output pin '{}' not found", output_name);
        }
    }
}

pub struct PCF8574 {
    address: u8,
    states: u8,
    dirty: Mutex<bool>,
}

impl PCF8574 {
    pub fn new(addr: u8) -> Self {
        PCF8574 {
            address: addr,
            states: 0,
            dirty: Mutex::new(false),
        }
    }
}

impl I2CInput for PCF8574 {
    fn read(&self) {}
    fn state(&self) -> bool {
        true
    }
}

impl I2COutput for PCF8574 {
    fn write(&mut self) {
        println!(
            "Writing value '0x{:02X}' to {:02X}",
            self.states, self.address
        );
        let mut dirty = self.dirty.lock().unwrap();
        *dirty = false;
    }

    fn set_at(&mut self, pos: usize, value: bool) {
        println!("Setting pin {} to {}.", pos, value);
        if pos > 7 {
            panic!("Position {} out of range. Must be between 0 and 7.", pos);
        }
        if value {
            self.states |= 1 << pos;
        } else {
            self.states &= !(1 << pos);
        }

        let mut dirty = self.dirty.lock().unwrap();
        *dirty = true;
    }

    fn is_dirty(&self) -> bool {
        let dirty = self.dirty.lock().unwrap();
        *dirty
    }
}
