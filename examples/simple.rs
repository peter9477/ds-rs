use log::*;

extern crate ds;

use ds::*;

use std::thread;
use std::time::Duration;


fn main() {
    env_logger::init();

    let team = 8089;
    let alliance = Alliance::new_red(1); // position 1-3
    let mut ds = DriverStation::new_team(team, alliance);

    info!("Starting: team {team}");
    thread::sleep(Duration::from_millis(1000));
    ds.restart_code();

    ds.set_tcp_consumer(|pkt| {
        match pkt {
            TcpPacket::Stdout(s) => {
                // timestamp: f32,
                // message: String,
                // seqnum: u16,
                println!(">> {}", s.message);
            }

            _ => {}
        }
    });

    ds.set_joystick_supplier(|| {
        vec![vec![
            JoystickValue::Button { id: 1, pressed: false },
            JoystickValue::Button { id: 2, pressed: false },
            JoystickValue::Button { id: 3, pressed: false },
            JoystickValue::Button { id: 4, pressed: false },
            JoystickValue::Button { id: 5, pressed: false },
            JoystickValue::Axis { id: 5, value: 0.100 },
            // JoystickValue::Button { id: 6, pressed: false },
            // pub enum JoystickValue {
            //     /// `value` should range from `-1.0..=1.0`, or `0.0..=1.0` if the axis is a trigger
            //     Axis { id: u8, value: f32 },
            //     /// Represents a button value to be sent to the roboRIO
            //     Button { id: u8, pressed: bool },
            //     /// Represents a POV, or D-pad value to be sent to the roboRIO
            //     POV { id: u8, angle: i16 },
            // }
        ]]
    });

    let mut count = 0;
    let mut battery = 0.0f32;
    let mut started = false;
    let mut mode = ds.mode();
    debug!("mode {:?}", mode);

    const MIN_BATT_DIFF: f32 = 0.050; // volts
    loop {
        if count % (50*5) == 0 {
            let v = ds.trace().is_code_started();
            if started != v {
                started = v;
                debug!("#{} started={}", count, started);
            }

            let v = ds.battery_voltage();
            if (battery - v).abs() >= MIN_BATT_DIFF {
                battery = v;
                info!("battery {v:.3}V");
            }

            if mode != ds.mode() {
                mode = ds.mode();
                info!("MODE {:?}", mode);
            }
        }
        count += 1;

        // crude experiment: at exactly 5 seconds enable robot
        if count == 50*5 {
            ds.enable();
        }
        if count == 50*15 {
            ds.disable();
        }

        thread::sleep(Duration::from_millis(20));
    }
}
