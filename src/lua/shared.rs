use std::{
    cell::{Ref, RefCell, RefMut},
    fmt,
    ops::Deref,
    rc::Rc,
};

/// Used to hold rust struct references in lua
/// Allow for field nesting modification, like foo.pos.x = 10
pub struct Shared<T: fmt::Debug>(Rc<RefCell<T>>);

impl<T: fmt::Debug> Shared<T> {
    pub fn new(data: T) -> Self {
        Self(Rc::new(RefCell::new(data)))
    }

    pub fn borrow(&self) -> Ref<T> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.0.deref().borrow_mut()
    }
}

impl<T: fmt::Debug> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}

impl<T: fmt::Debug> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Shared({:?})", self.0.borrow())
    }
}
