use std::collections::VecDeque;

pub struct Pool<T> {
    objects: VecDeque<T>,
    object_generator: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T> Pool<T> {
    pub fn new<F>(object_generator: F, initial_capacity: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let mut objects = VecDeque::with_capacity(initial_capacity);
        for _ in 0..initial_capacity {
            objects.push_back(object_generator());
        }
        Self {
            objects,
            object_generator: Box::new(object_generator),
        }
    }

    #[inline(always)]
    pub fn get(&mut self) -> T {
        self.objects.pop_back().unwrap_or_else(|| (self.object_generator)())
    }

    #[inline(always)]
    pub fn return_(&mut self, item: T) {
        self.objects.push_back(item);
    }

    pub fn count(&self) -> usize {
        self.objects.len()
    }
}