use std::fmt::Debug;

#[derive(Clone, Debug)]

pub struct Circular<T> {
    vec: Vec<Option<T>>,
    front: usize,
    size: usize,
    cap: usize,
}

impl<T: Clone + Debug> Circular<T> {
    pub fn new(cap: usize) -> Circular<T> {
        Circular {
            vec: vec![None; cap],
            front: 0,
            size: 0,
            cap,
        }
    }

    pub fn push(&mut self, entry: T) {
        if self.size == self.cap {
            self.pop();
        }

        self.size += 1;
        let rear = (self.front + self.size - 1) % self.cap;
        self.vec[rear] = Some(entry);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.size).map(move |i| {
            let index = (self.front + i) % self.cap;
            self.vec[index].as_ref().unwrap()
        })
    }

    pub fn pop(&mut self) -> Result<T, &str> {
        match self.get() {
            None => Err("Queue is empty."),
            Some(v) => {
                self.front = (self.front + 1) % self.cap;
                self.size -= 1;
                Ok(v)
            }
        }
    }

    pub fn get(&self) -> Option<T> {
        return self.vec[self.front].clone();
    }

    pub fn set(&mut self, entry: T) {}
}
