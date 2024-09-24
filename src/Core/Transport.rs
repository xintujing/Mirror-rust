pub trait Transport {
    // 当前平台可以使用此传输？
    fn available(&self) -> bool;

    //

}