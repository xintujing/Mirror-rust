struct NetworkReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> NetworkReader<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        NetworkReader { buffer, position: 0 }
    }
}

struct NetworkReaderPool;

impl NetworkReaderPool {
    fn get_reader<'a>(data: &'a [u8]) -> NetworkReaderPooled<'a> {
        // Simulate getting a NetworkReader from a pool
        NetworkReaderPooled::new(data)
    }

    fn return_reader<'a>(_reader: NetworkReaderPooled<'a>) {
        // Simulate returning a NetworkReader to a pool
        println!("Reader returned to pool");
    }
}

struct NetworkReaderPooled<'a> {
    inner: NetworkReader<'a>,
}

impl<'a> NetworkReaderPooled<'a> {
    fn new(data: &'a [u8]) -> Self {
        NetworkReaderPooled {
            inner: NetworkReader::new(data),
        }
    }
}

impl<'a> Drop for NetworkReaderPooled<'a> {
    fn drop(&mut self) {
        // Automatically return the reader to the pool when it goes out of scope
        NetworkReaderPool::return_reader(self.clone());
    }
}

impl<'a> Clone for NetworkReaderPooled<'a> {
    fn clone(&self) -> Self {
        NetworkReaderPooled {
            inner: NetworkReader::new(self.inner.buffer),
        }
    }
}

fn main() {
    let data = [0, 1, 2, 3, 4, 5];
    {
        let reader = NetworkReaderPool::get_reader(&data);
        // use `reader` within this scope
    } // `reader` is automatically returned to the pool here due to `Drop`
}
