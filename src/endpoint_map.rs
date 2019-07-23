use usb_device::endpoint::EndpointAddress;

pub struct EndpointMap
{
    map: [u8; 16],
    mask: u32,
}

fn addr2index(ep_addr: EndpointAddress) -> usize {
    (u8::from(ep_addr).rotate_left(1) & 0x1f) as usize
}

impl EndpointMap {
    pub fn new() -> Self {
        Self {
            map: [0; 16],
            mask: 0
        }
    }

    pub fn set(&mut self, ep_addr: EndpointAddress, index: u8) {
        assert!(index < 16);

        let bit_index = addr2index(ep_addr);
        self.mask |= 1 << bit_index;

        let mask = 0xf << ((bit_index & 1) * 4);
        let value = index << ((bit_index & 1) * 4);
        let b = &mut self.map[bit_index >> 1];
        *b = (*b & !mask) | value;
    }

    pub fn get(&self, ep_addr: EndpointAddress) -> Option<u8> {
        let bit_index = addr2index(ep_addr);

        if self.mask & (1 << bit_index) == 0 {
            return None;
        }

        Some((self.map[bit_index >> 1] >> ((bit_index & 1) * 4)) & 0xf)
    }
}

impl<'a> IntoIterator for &'a EndpointMap {
    type Item = (EndpointAddress, u8);
    type IntoIter = EndpointMapIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        EndpointMapIterator {
            map: self,
            bit_index: 0
        }
    }
}

pub struct EndpointMapIterator<'a> {
    map: &'a EndpointMap,
    bit_index: u8,
}

impl Iterator for EndpointMapIterator<'_> {
    type Item = (EndpointAddress, u8);

    fn next(&mut self) -> Option<Self::Item> {
        while self.bit_index < 32 {
            let ep_addr = self.bit_index.rotate_right(1).into();
            if let Some(index) = self.map.get(ep_addr) {
                return Some((ep_addr, index));
            }
            self.bit_index += 1;
        }
        None
    }
}
