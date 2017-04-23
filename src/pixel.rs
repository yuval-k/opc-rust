use std;

const NUM_RGB_BYTES: usize = 3;

#[derive(Debug)]
pub struct OpcPixel<'a, T: AsRef<[u8]> + 'a> {
    x: T,
    pub phantom: std::marker::PhantomData<&'a [u8]>,
}

impl<'a, T: AsRef<[u8]> + 'a> OpcPixel<'a, T> {
    pub fn new(x: T) -> Self {
        OpcPixel {
            x: x,
            phantom: std::marker::PhantomData,
        }
    }
    pub fn r(&self) -> u8 {
        self.x.as_ref()[0]
    }
    pub fn g(&self) -> u8 {
        self.x.as_ref()[1]
    }
    pub fn b(&self) -> u8 {
        self.x.as_ref()[2]
    }
}

impl<'a, T: AsRef<[u8]> + AsMut<[u8]> + 'a> OpcPixel<'a, T> {
    pub fn set_r(&mut self, v: u8) {
        self.x.as_mut()[0] = v;
    }
    pub fn set_g(&mut self, v: u8) {
        self.x.as_mut()[1] = v;
    }
    pub fn set_b(&mut self, v: u8) {
        self.x.as_mut()[2] = v;
    }
}

impl<'a, T: AsRef<[u8]> + 'a> std::convert::From<&'a OpcPixel<'a, T>> for (u8, u8, u8) {
    fn from(t: &'a OpcPixel<'a, T>) -> (u8, u8, u8) {
        (t.r(), t.g(), t.b())
    }
}

#[derive(Clone,Debug)]
pub struct Pixels {
    pixels: Vec<u8>,
}

impl std::convert::From<Vec<u8>> for Pixels {
    fn from(t: Vec<u8>) -> Pixels {
        let mut t = t;
        let blen: usize = t.len() - (t.len() % NUM_RGB_BYTES);
        t.truncate(blen);
        Pixels { pixels: t }
    }
}

impl std::convert::From<Pixels> for Vec<u8> {
    fn from(t: Pixels) -> Vec<u8> {
        t.pixels
    }
}

impl Pixels {
    pub fn new(n: usize) -> Pixels {
        Pixels { pixels: vec![0;n*NUM_RGB_BYTES] }
    }

    pub fn len_bytes(&self) -> usize {
        self.pixels.len()
    }

    pub fn iter(&self) -> PixelIterator {
        PixelIterator::new(self)
    }

    pub fn iter_mut(&mut self) -> PixelIteratorMut {
        PixelIteratorMut::new(self)
    }
}

pub struct PixelIterator<'a> {
    p: std::slice::Chunks<'a, u8>,
}

impl<'a> PixelIterator<'a> {
    fn new(p: &'a Pixels) -> Self {
        PixelIterator { p: p.pixels.chunks(NUM_RGB_BYTES) }
    }
}

impl<'a> std::iter::Iterator for PixelIterator<'a> {
    type Item = OpcPixel<'a, &'a [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        self.p.next().map(|x| OpcPixel::new(x))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.p.nth(n).map(|x| OpcPixel::new(x))
    }

}

impl<'a> ExactSizeIterator for PixelIterator<'a> {
    fn len(&self) -> usize {
        self.p.len()
    }
}

pub struct PixelIteratorMut<'a> {
    p: std::slice::ChunksMut<'a, u8>,
}

impl<'a> PixelIteratorMut<'a> {
    fn new(p: &'a mut Pixels) -> Self {
        PixelIteratorMut { p: p.pixels.chunks_mut(NUM_RGB_BYTES) }
    }
}

impl<'a> std::iter::Iterator for PixelIteratorMut<'a> {
    type Item = OpcPixel<'a, &'a mut [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        self.p.next().map(|x| OpcPixel::new(x))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.p.nth(n).map(|x| OpcPixel::new(x))
    }

}

impl<'a> ExactSizeIterator for PixelIteratorMut<'a> {
    fn len(&self) -> usize {
        self.p.len()
    }
}
