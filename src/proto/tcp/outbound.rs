use byteorder::{BigEndian, WriteBytesExt};

#[derive(Debug, Clone)]
pub enum TcpTag {
    MatchInfo(MatchInfo),
    GameData(GameData),
    JoystickDesc(JoystickDesc),
}

pub(crate) trait OutgoingTcpTag {
    fn id(&self) -> u8;

    fn data(&self) -> Vec<u8>;

    fn construct(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.id());
        buf.extend(self.data());

        let mut out = Vec::new();
        out.write_u16::<BigEndian>(buf.len() as u16).unwrap();
        out.extend(buf);

        out
    }
}

#[derive(Debug, Clone)]
pub struct MatchInfo {
    competition: String,
    match_type: MatchType,
}

impl OutgoingTcpTag for MatchInfo {
    fn id(&self) -> u8 {
        0x07
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.competition.len() as u8);
        buf.extend_from_slice(self.competition.as_bytes());
        buf.push(self.match_type as u8);

        buf
    }
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub gsm: String,
}

impl OutgoingTcpTag for GameData {
    fn id(&self) -> u8 {
        0x0e
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(self.gsm.as_bytes());

        buf
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
#[allow(unused)]
pub enum MatchType {
    None = 0,
    Practice = 1,
    Qualifications = 2,
    Eliminations = 3,
}

#[derive(Debug, Clone)]
pub struct JoystickDesc;

impl OutgoingTcpTag for JoystickDesc {
    fn id(&self) -> u8 {
        0x02
    }

    fn data(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[0, 0, 21, 3, b'P', b'S', b'4', 6, 0, 1, 2, 3, 4, 5, 10, 1]);

        buf
    }
}
