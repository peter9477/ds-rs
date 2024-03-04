use log::*;

use crate::ds::state::{DsMode, JoystickSupplier};
use crate::proto::udp::outbound::types::tags::*;
use crate::proto::udp::outbound::types::{Control, Request};
use crate::proto::udp::outbound::*;
use crate::{Alliance, JoystickValue, Mode};
use std::f32;

/// State containing all the data relevant to constructing a UDP control packet to the roboRIO
pub struct SendState {
    /// The mode the robot should be enabled in
    mode: Mode,
    /// The current sequence number
    udp_seqnum: u16,
    /// Whether the robot is enabled
    enabled: bool,
    /// Whether the robot is estopped
    estopped: bool,
    /// The current alliance of the robot
    pub alliance: Alliance,
    /// Any UDP tags that are to be sent with the next UDP control packet
    pending_udp: Vec<UdpTag>,
    /// An optional source for joystick values that will be encoded and sent with the packet
    joystick_provider: Option<Box<JoystickSupplier>>,
    /// Pending reboot or code restart requests
    pending_request: Option<Request>,
    dsmode: DsMode,
}

impl SendState {
    pub fn new(alliance: Alliance) -> SendState {
        SendState {
            mode: Mode::Autonomous,
            udp_seqnum: 0,
            enabled: false,
            estopped: false,
            alliance,
            pending_udp: Vec::new(),
            joystick_provider: None,
            pending_request: None,
            dsmode: DsMode::Normal,
        }
    }

    pub fn request(&mut self, request: Request) {
        self.pending_request = Some(request);
    }

    pub fn queue_udp(&mut self, tag: UdpTag) {
        self.pending_udp.push(tag);
    }

    pub fn pending_udp(&self) -> &Vec<UdpTag> {
        &self.pending_udp
    }

    pub fn set_joystick_supplier(
        &mut self,
        supplier: impl Fn() -> Vec<Vec<JoystickValue>> + Send + Sync + 'static,
    ) {
        self.joystick_provider = Some(Box::new(supplier))
    }

    pub fn set_alliance(&mut self, alliance: Alliance) {
        self.alliance = alliance;
    }

    /// Constructs a control packet from the current state
    ///
    /// if [self.joystick_provider] is Some, it will be used to construct the joysticks tag
    /// if [self.request] is Some, its value will be consumed and sent to the roboRIO
    pub fn control(&mut self) -> UdpControlPacket {
        if let Some(ref supplier) = &self.joystick_provider {
            let joysticks = supplier();

            // Joystick tags come one after another, iterate over the outer Vec and queue with each loop
            for joystick in &joysticks {
                let mut axes = vec![0; 6];
                let mut buttons = vec![false; 10];
                let mut povs = vec![-1i16];

                // This has various flaws including the fixed-size vecs,
                // the use of remove/insert instead of just modifying the entries,
                // the fact it will panic if you try to use ids beyond the
                // entries here, and the fact it will always generate a
                // packet even if there are no changes in the data since last
                // time (that the last one's a problem is just a theory for now...
                // but it really shouldn't be necessary to send this data
                // in every single packet if it has not changed).
                for value in joystick {
                    // If statements bound check to stop it from crashing
                    match value {
                        JoystickValue::Button { id, pressed } => {
                            if *id >= 1 && *id <= 10 {
                                let id = id - 1;
                                buttons.remove(id as usize);
                                buttons.insert(id as usize, *pressed)
                            }
                        }
                        JoystickValue::Axis { id, value } => {
                            if *id <= 5 {
                                let value = if (*value - 1.0).abs() < f32::EPSILON {
                                    127i8
                                } else {
                                    (value * 128f32) as i8
                                };

                                axes.remove(*id as usize);
                                axes.insert(*id as usize, value);
                            }
                        }
                        JoystickValue::POV { id, angle } => {
                            if *id == 0 {
                                povs.remove(*id as usize);
                                povs.insert(*id as usize, *angle);
                            }
                        }
                    }
                }

                let tag = Joysticks::new(axes, buttons, povs);
                // debug!("{}", hex::encode(tag.data()));
                self.queue_udp(UdpTag::Joysticks(tag));
            }
        }

        let mut control = self.mode.to_control();

        if self.enabled {
            control |= Control::ENABLED;
        }

        if self.estopped {
            control |= Control::ESTOP
        }

        let mut tags: Vec<Box<dyn Tag>> = Vec::new();

        for tag in self.pending_udp.clone() {
            match tag {
                UdpTag::Timezone(tz) => tags.push(Box::new(tz)),
                UdpTag::DateTime(dt) => tags.push(Box::new(dt)),
                UdpTag::Joysticks(joy) => tags.push(Box::new(joy)),
                UdpTag::Countdown(cnt) => tags.push(Box::new(cnt)),
            }
        }

        self.pending_udp.clear();

        UdpControlPacket {
            seqnum: self.udp_seqnum,
            control,
            request: self.pending_request.take(),
            alliance: self.alliance,
            tags,
        }
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn ds_mode(&self) -> &DsMode {
        &self.dsmode
    }

    pub fn set_ds_mode(&mut self, mode: DsMode) {
        self.dsmode = mode;
    }

    pub fn increment_seqnum(&mut self) {
        self.udp_seqnum = self.udp_seqnum.wrapping_add(1);
    }

    pub fn reset_seqnum(&mut self) {
        self.udp_seqnum = 0;
    }

    #[allow(unused)]
    pub fn seqnum(&self) -> u16 {
        self.udp_seqnum
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn estop(&mut self) {
        self.disable();
        self.estopped = true;
    }

    pub fn estopped(&self) -> bool {
        self.estopped
    }
}
