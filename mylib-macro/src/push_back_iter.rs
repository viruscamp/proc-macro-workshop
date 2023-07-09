use std::{collections::VecDeque, ops::{Deref, DerefMut}};

pub struct PushBackIterator<T, ITER: Iterator<Item = T>> {
    vec_deque: VecDeque<T>,
    iter: ITER,
}
impl<T, ITER: Iterator<Item = T>> PushBackIterator<T, ITER> {
    pub fn new(iter: ITER) -> Self {
        Self {
            vec_deque: VecDeque::new(),
            iter
        }
    }
}
impl<T, ITER: Iterator<Item = T>> Deref for PushBackIterator<T, ITER> {
    type Target = VecDeque<T>;
    fn deref(&self) -> &Self::Target {
        &self.vec_deque
    }
}
impl<T, ITER: Iterator<Item = T>> DerefMut for PushBackIterator<T, ITER> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec_deque
    }
}
impl<T, ITER: Iterator<Item = T>> Iterator for PushBackIterator<T, ITER> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tt) = self.vec_deque.pop_front() {
            Some(tt)
        } else if let Some(tt) = self.iter.next() {
            Some(tt)
        } else {
            None
        }
    }
}
