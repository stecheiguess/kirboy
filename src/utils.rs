pub struct Circular<T> {
    vec: Vec<T>,
    start: usize,
    end: usize,
    pointer: usize,
}

impl<T> Circular<T> {
    pub fn new(array: Vec<T>) -> Circular<T> {
        let length = array.len();
        Circular {
            vec: array,
            start: 0,
            end: length,
            pointer: 0,
        }
    }

    pub fn push(&mut self, entry: T) {}

    /*pub fn pop(&mut self) -> T {

        return self.pointer
    }

    pub fn index(&self) -> usize {
        return  self.pointer % self.end
    }

    pub fn dec(&mut self) {
        self.pointer = (self.pointer % self.end) - 1
    }

    pub fn inc(&mut self) {
        self.pointer = (self.pointer % self.end) + 1
    }

    pub fn get(&self, index: usize) {}

    pub fn set(&mut self, entry: T) {}*/
}
