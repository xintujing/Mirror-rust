use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct Pool<T> {
    objects: RefCell<VecDeque<T>>,
    object_generator: Arc<dyn Fn() -> T>, // 使用 Arc 以便共享闭包
}

impl<T> Pool<T> {
    // 创建一个新的对象池，传入对象生成器和初始容量
    pub fn new(object_generator: Arc<dyn Fn() -> T>, initial_capacity: usize) -> Self {
        let mut objects = VecDeque::with_capacity(initial_capacity);

        // 预先生成指定容量的对象
        for _ in 0..initial_capacity {
            objects.push_back(object_generator());
        }

        Pool {
            objects: RefCell::new(objects),
            object_generator,
        }
    }

    // 从池中获取一个对象，如果池为空则生成一个新对象
    #[inline(always)]
    pub fn get(&self) -> T {
        let mut objects = self.objects.borrow_mut();
        objects.pop_front().unwrap_or_else(|| (self.object_generator)())
    }

    // 将一个对象返回到池中
    #[inline(always)]
    pub fn return_object(&self, item: T) {
        let mut objects = self.objects.borrow_mut();
        objects.push_back(item);
    }

    // 获取池中对象的数量，用于测试
    pub fn count(&self) -> usize {
        self.objects.borrow().len()
    }
}
