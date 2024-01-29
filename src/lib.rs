use datagram_transport::{DatagramRead, DatagramTransport, DatagramWrite};
use rand::Rng;

pub struct HazyDirection {
    pub drop_chance: u32,
}

pub struct HazyTransport<T: DatagramTransport> {
    pub outgoing: HazyDirection,
    pub incoming: HazyDirection,
    pub inner: T,
}

impl<T: DatagramTransport> HazyTransport<T> {
    pub fn new(inner: T) -> Self {
        Self {
            outgoing: HazyDirection { drop_chance: 0 },
            incoming: HazyDirection { drop_chance: 0 },
            inner,
        }
    }
}

impl<T: DatagramTransport> DatagramWrite for HazyTransport<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<()> {
        let mut rng = rand::thread_rng();
        let drop_chance = rng.gen_range(0..100);
        if drop_chance < self.outgoing.drop_chance {
            return Ok(());
        }
        self.inner.write(buf)
    }
}

impl<T: DatagramTransport> DatagramRead for HazyTransport<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut rng = rand::thread_rng();
        let size_to_return = self.inner.read(buf)?;
        let drop_chance = rng.gen_range(0..100);
        if drop_chance < self.incoming.drop_chance {
            return Ok(0);
        }
        Ok(size_to_return)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTransport {}

    impl DatagramWrite for TestTransport {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl DatagramRead for TestTransport {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            buf[0] = 0;
            Ok(1)
        }
    }

    impl DatagramTransport for TestTransport {}

    #[test]
    fn it_works() {
        let test_transport = TestTransport {};
        let mut hazy = HazyTransport::<TestTransport>::new(test_transport);
        let data: &[u8] = &[1, 2, 3, 4, 5];
        assert_eq!(hazy.write(data).unwrap(), ());
    }
}
