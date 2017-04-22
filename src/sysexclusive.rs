use std;

#[derive(Clone,Debug)]
pub struct SystemExclusiveData {
    sys_exclusive: Vec<u8>,
}


impl SystemExclusiveData {
    pub fn newo<T: Into<Vec<u8>>>(systemid: u16, data: T) -> Self {
        let mut data = data.into();
        let syshigh: u8 = (systemid >> 8) as u8;
        let syslow: u8 = (systemid & 0xff) as u8;
        let mut header = vec![syshigh, syslow];
        header.append(&mut data);
        SystemExclusiveData { sys_exclusive: header }
    }

    pub fn get_system_id(&self) -> u16 {
        if self.sys_exclusive.len() < 2 {
            0
        } else {
            ((self.sys_exclusive[1] as u16) << 8) + (self.sys_exclusive[0] as u16)
        }
    }

    pub fn get_data(&self) -> &[u8] {
        if self.sys_exclusive.len() < 2 {
            &self.sys_exclusive[0..0]
        } else {
            &self.sys_exclusive[2..]
        }
    }
}


impl std::convert::From<Vec<u8>> for SystemExclusiveData {
    fn from(t: Vec<u8>) -> SystemExclusiveData {
        SystemExclusiveData { sys_exclusive: t }
    }
}

impl std::convert::From<SystemExclusiveData> for Vec<u8> {
    fn from(t: SystemExclusiveData) -> Vec<u8> {
        t.sys_exclusive
    }
}
