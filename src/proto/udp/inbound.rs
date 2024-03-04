use log::*;

pub mod types;

use self::types::*;

use crate::ext::BufExt;
use crate::Result;

use bytes::Buf;

/// Response packet sent by the RIO over UDP every ~20ms.
#[derive(Debug)]
pub struct UdpResponsePacket {
    pub seqnum: u16,
    pub status: Status,
    pub trace: Trace,
    pub battery: f32,
    pub need_date: bool,
}

impl UdpResponsePacket {
    /// Attempts to decode a valid response packet from the given buffer
    /// Will return Err() if any of the reads fail.
    pub fn decode(buf: &mut (impl Buf + Clone)) -> Result<(UdpResponsePacket, usize)> {
        let mut step = 0;
        let before = buf.clone();
        // let text = format!("{}", hex::encode(before.bytes()));
        // debug!("decode: {} {}", text.len() / 2, text);

        let res = (|| {
            let mut len = 0;
            let seqnum = buf.read_u16_be()?;
            step = 1;
            len += 2;

            buf.read_u8()?; // Get rid of comm version
            step = 2;
            len += 1;

            let status = Status::from_bits(buf.read_u8()?).unwrap();
            step = 3;
            let trace = Trace::from_bits(buf.read_u8()?).unwrap();
            step = 4;
            len += 2;

            let battery = {
                let high = buf.read_u8()?;
                step = 5;
                let low = buf.read_u8()?;
                step = 6;
                f32::from(high) + f32::from(low) / 256f32
            };
            len += 2;

            let need_date = buf.read_u8()? == 1;
            step = 7;
            len += 1;

            if let Ok(_tag_len) = buf.read_u8() {
                step += 8;
                // debug!("tag data {}", tag_len);

                use crate::util::InboundTag;
                while let Ok(tag_id) = buf.read_u8() {
                    len += 1;
                    match tag_id {
                        0x01 => {
                            types::tags::JoystickOutput::chomp(buf)?;
                            len += 8;
                        }
                        0x04 => {
                            types::tags::DiskInfo::chomp(buf)?;
                            len += 4;
                        }
                        0x05 => {
                            // debug!("chomp CPU {len} {}", hex::encode(buf.clone().bytes()));
                            types::tags::CPUInfo::chomp(buf)?;
                            // debug!("chomped");
                            len += 1 + 4 * 4 * 2; // cpu count plus 4 32-bit words per cpu
                        }
                        0x06 => {
                            types::tags::RAMInfo::chomp(buf)?;
                            len += 8;
                        }
                        0x08 => {
                            types::tags::PDPLog::chomp(buf)?;
                            len += 25;
                        }
                        0x09 => {
                            types::tags::Unknown::chomp(buf)?;
                            len += 9;
                        }
                        0x0e => {
                            types::tags::CANMetrics::chomp(buf)?;
                            len += 14;
                        }
                        _ => {}
                    }

                    // if !buf.has_remaining() {
                    //     debug!("no more bytes");
                    //     break;
                    // }
                    // debug!("looping");
                }

                // debug!("done while loop");
            }

            Ok((
                UdpResponsePacket {
                    seqnum,
                    status,
                    trace,
                    battery,
                    need_date,
                },
                len,
            ))
        })();

        match res {
            Err(ref err) => {
                // error!("decode: {:?} in {}", err, hex::encode(""));
                // 0177 sequence
                // 01   version (always 1 for now)
                // 02   status
                // 31   trace
                // 0bdc battery 0xb + 0xdc/256 = 11.859V
                // 00   request date = no
                // 22   tag length
                //  05  id, 5=cpu
                //  02  num cpus
                //  41bd6a05 cpu0 time critical %
                //  00000000 cpu0 above normal %
                //  00000000 cpu0 normal %
                //  4070c0d2 cpu0 low %
                //  4150f3d6 cpu1 time critical %
                //  00000000 cpu1 above normal %
                //  00000000 cpu1 normal %
                //  40680005 cpu1 low %

                // 0ac5
                // 0102310bd700
                // 220502 41a96d2b0000000000000000405f728841535a860000000000000000405cef45
                error!("decode: {err:?} at {step} in {}", hex::encode(before.bytes()));
            }
            _ => {}
        }

        res
    }
}
