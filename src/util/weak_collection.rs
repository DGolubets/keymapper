#![allow(dead_code)]
use std::rc::{Rc, Weak};
use std::cell::{Cell, RefCell, Ref};
use std::mem;
use std::ops::Deref;

/// A collection type that holds no strong references to items.
/// An item lives in collection for as long as there is an external reference to it.
pub struct WeakCollection<T> {
    collection: Rc<RefCell<WeakCollectionInternal<T>>>
}

impl<T> WeakCollection<T> {

    pub fn new() -> Self {
        let internal_collection = WeakCollectionInternal {
            items: Vec::new()
        };

        let internal_collection = Rc::new(RefCell::new(internal_collection));

        WeakCollection {
            collection: internal_collection
        }
    }

    pub fn push(&mut self, value: T) -> WeakCollectionItem<T> {
        let mut collection = self.collection.borrow_mut();

        let item = WeakCollectionItemInternal {
            owner: Rc::downgrade(&self.collection),
            index: Cell::new(collection.items.len()),
            value: value
        };

        let item = Rc::new(item);
        collection.items.push(Rc::downgrade(&item));

        WeakCollectionItem {
            item: item
        }
    }

    pub fn len(&self) -> usize {
        self.collection.borrow().items.len()
    }

    pub fn capacity(&self) -> usize {
        self.collection.borrow().items.capacity()
    }

    pub fn shrink_to_fit(&mut self) {
        self.collection.borrow_mut().items.shrink_to_fit();
    }
}

struct WeakCollectionInternal<T> {
    items: Vec<Weak<WeakCollectionItemInternal<T>>>
}

pub struct WeakCollectionItem<T> {
    item: Rc<WeakCollectionItemInternal<T>>
}

impl<T> WeakCollectionItem<T> {
    pub fn get(&self) -> &T {
        &self.item.value
    }
}

impl<T> Deref for WeakCollectionItem<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.item.value
    }
}

struct WeakCollectionItemInternal<T> {
    owner: Weak<RefCell<WeakCollectionInternal<T>>>,
    index: Cell<usize>,
    value: T
}

impl<T> Drop for WeakCollectionItemInternal<T> {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.upgrade() {
            let mut owner = owner.borrow_mut();
            let index = self.index.get();
            owner.items.swap_remove(index);

            if owner.items.len() > 0 {
                // set valid index on swapped item
                if let Some(item) = owner.items[index].upgrade(){
                    item.index.set(index);
                }
            }
        }
    }
}


impl<'a, T: 'a> IntoIterator for &'a WeakCollection<T> {
    type Item = WeakCollectionItem<T>;
    type IntoIter = WeakCollectionIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        WeakCollectionIterator::new(self.collection.borrow())
    }
}

pub struct WeakCollectionIterator<'a, T: 'a> {
    iter: ::std::slice::Iter<'a, Weak<WeakCollectionItemInternal<T>>>,
    _rm: Ref<'a, WeakCollectionInternal<T>> // own Ref here to keep RefCell state valid
}

impl <'a, T: 'a> WeakCollectionIterator<'a, T> {
    fn new(t: Ref<'a, WeakCollectionInternal<T>>) -> WeakCollectionIterator<'a, T>{
        WeakCollectionIterator {
            iter: unsafe { mem::transmute((&t.items).into_iter()) },
            _rm: t,
        }
    }
}

impl<'a, T: 'a> Iterator for WeakCollectionIterator<'a, T> {
    type Item = WeakCollectionItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(wrc) => match wrc.upgrade() {
                    Some(rc) => {
                        let item = WeakCollectionItem {
                            item: rc
                        };
                        return Some(item)
                    },
                    None => ()
                },
                None => return None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use util::weak_collection::*;

    #[test]
    fn drops_dead_elements() {
        let mut collection = WeakCollection::new();
        let a = collection.push("abc");
        let b = collection.push("def");

        drop(a);
        let snapshot: Vec<WeakCollectionItem<&str>> = collection.into_iter().collect();
        assert_eq!(1, snapshot.len());
        assert_eq!(*b, *snapshot[0]);
    }

    #[test]
    fn keep_valid_item_indexes_after_drop() {
        let mut collection = WeakCollection::new();
        let a = collection.push("abc");
        let b = collection.push("def");

        assert_eq!(b.item.index.get(), 1);
        drop(a);
        assert_eq!(b.item.index.get(), 0);
    }

    #[test]
    fn shrink_to_fit() {
        let mut collection = WeakCollection::new();
        let a = collection.push("abc");
        let b = collection.push("def");

        drop(a);
        collection.shrink_to_fit();
        assert_eq!(1, collection.capacity());
    }
}
