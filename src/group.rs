use fnv::FnvHashMap;
use {UndoCmd, UndoStack};

/// A unique id for an `UndoStack`.
pub struct Uid(u64);

/// A collection of `UndoStack`s.
///
/// A `UndoGroup` is useful when working with multiple `UndoStack`s and only one of them should
/// be active at a given time, like a text editor with multiple documents opened.
pub struct UndoGroup<'a> {
    group: FnvHashMap<u64, UndoStack<'a>>,
    active: Option<u64>,
    id: u64,
}

impl<'a> UndoGroup<'a> {
    /// Creates a new `UndoGroup`.
    pub fn new() -> Self {
        UndoGroup {
            group: FnvHashMap::default(),
            active: None,
            id: 0,
        }
    }

    /// Creates a new `UndoGroup` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        UndoGroup {
            group: FnvHashMap::with_capacity_and_hasher(capacity, Default::default()),
            active: None,
            id: 0,
        }
    }

    /// Returns the capacity of the `UndoGroup`.
    pub fn capacity(&self) -> usize {
        self.group.capacity()
    }

    /// Shrinks the capacity of the `UndoGroup` as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.group.shrink_to_fit()
    }

    /// Adds an `UndoStack` to the group and returns an unique id for this stack.
    pub fn add_stack(&mut self, stack: UndoStack<'a>) -> Uid {
        let id = self.id;
        self.id += 1;
        self.group.insert(id, stack);
        Uid(id)
    }

    /// Removes the `UndoStack` with the specified id.
    pub fn remove_stack(&mut self, Uid(id): Uid) -> UndoStack<'a> {
        let stack = self.group.remove(&id).unwrap();
        // Check if it was the active stack that was removed.
        if let Some(active) = self.active {
            if active == id {
                // If it was, we set the active stack as None.
                self.active = None;
            }
        }
        stack
    }

    /// Sets the `UndoStack` with the specified id as the current active one.
    pub fn set_active_stack(&mut self, &Uid(id): &Uid) {
        self.active = Some(id);
    }

    /// Clears the current active `UndoStack`.
    pub fn clear_active_stack(&mut self) {
        self.active = None;
    }

    /// Calls [`is_clean`] on the active `UndoStack`, if there is one.
    /// Returns `None` if there is no active stack.
    ///
    /// [`is_clean`]: struct.UndoStack.html#method.is_clean
    pub fn is_clean(&self) -> Option<bool> {
        self.active.as_ref().and_then(|i| self.group.get(i).map(|t| t.is_clean()))
    }

    /// Calls [`is_dirty`] on the active `UndoStack`, if there is one.
    /// Returns `None` if there is no active stack.
    ///
    /// [`is_dirty`]: struct.UndoStack.html#method.is_dirty
    pub fn is_dirty(&self) -> Option<bool> {
        self.is_clean().map(|t| !t)
    }

    /// Calls [`push`] on the active `UndoStack`, if there is one.
    /// Does nothing if there is no active stack.
    ///
    /// [`push`]: struct.UndoStack.html#method.push
    pub fn push<T>(&mut self, cmd: T)
        where T: UndoCmd + 'a,
    {
        if let Some(ref active) = self.active {
            let ref mut stack = self.group.get_mut(active).unwrap();
            stack.push(cmd);
        }
    }

    /// Calls [`redo`] on the active `UndoStack`, if there is one.
    /// Does nothing if there is no active stack.
    ///
    /// [`redo`]: struct.UndoStack.html#method.redo
    pub fn redo(&mut self) {
        if let Some(ref active) = self.active {
            let ref mut stack = self.group.get_mut(active).unwrap();
            stack.redo();
        }
    }

    /// Calls [`undo`] on the active `UndoStack`, if there is one.
    /// Does nothing if there is no active stack.
    ///
    /// [`undo`]: struct.UndoStack.html#method.undo
    pub fn undo(&mut self) {
        if let Some(ref active) = self.active {
            let ref mut stack = self.group.get_mut(active).unwrap();
            stack.undo();
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn pop() {
        use std::rc::Rc;
        use std::cell::RefCell;
        use {UndoCmd, UndoStack, UndoGroup};

        /// Pops an element from a vector.
        #[derive(Clone)]
        struct PopCmd {
            vec: Rc<RefCell<Vec<i32>>>,
            e: Option<i32>,
        }

        impl UndoCmd for PopCmd {
            fn redo(&mut self) {
                self.e = self.vec.borrow_mut().pop();
            }

            fn undo(&mut self) {
                self.vec.borrow_mut().push(self.e.unwrap());
                self.e = None;
            }
        }

        // We need to use Rc<RefCell> since all commands are going to mutate the vec.
        let vec1 = Rc::new(RefCell::new(vec![1, 2, 3]));
        let vec2 = Rc::new(RefCell::new(vec![1, 2, 3]));

        let mut group = UndoGroup::new();

        let a = group.add_stack(UndoStack::new());
        let b = group.add_stack(UndoStack::new());

        group.set_active_stack(&a);
        group.push(PopCmd { vec: vec1.clone(), e: None });

        assert_eq!(vec1.borrow().len(), 2);

        group.set_active_stack(&b);
        group.push(PopCmd { vec: vec2.clone(), e: None });

        assert_eq!(vec2.borrow().len(), 2);

        group.set_active_stack(&a);
        group.undo();

        assert_eq!(vec1.borrow().len(), 3);

        group.set_active_stack(&b);
        group.undo();

        assert_eq!(vec2.borrow().len(), 3);

        let _ = group.remove_stack(b);
        group.redo();

        assert_eq!(group.group.len(), 1);
        assert_eq!(vec2.borrow().len(), 3);
    }
}
