use std::collections::VecDeque;

pub struct Pool<T> {
    capacity: usize,
    objects_stack: VecDeque<T>,
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
            capacity: initial_capacity,
            objects_stack: objects,
            object_generator: Box::new(object_generator),
        }
    }

    #[inline(always)]
    pub fn get(&mut self) -> T {
        self.objects_stack.pop_back().unwrap_or_else(|| (self.object_generator)())
    }

    #[inline(always)]
    pub fn return_(&mut self, item: T) {
        if self.objects_stack.len() >= self.capacity {
            return;
        }
        self.objects_stack.push_back(item);
    }

    pub fn count(&self) -> usize {
        self.objects_stack.len()
    }
}