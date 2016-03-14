use std::rc::{Rc, Weak};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::mem;

/// A collection type that holds no strong references to items.
/// An item lives in collection for as long as there is an external reference to it.
pub struct WeakCollection<T> {
    collection: Rc<RefCell<WeakCollectionInternal<T>>>
}

impl<T> WeakCollection<T> {
    pub fn new() -> Self {
        let internal_collection = WeakCollectionInternal {
            last_id: 0,
            table_by_id: HashMap::new()
        };

        let internal_collection = Rc::new(RefCell::new(internal_collection));

        WeakCollection {
            collection: internal_collection
        }
    }

    pub fn push(&mut self, value: T) -> WeakCollectionItem<T> {
        let mut collection = self.collection.borrow_mut();
        collection.last_id += 1;

        let item = WeakCollectionItemInternal {
            owner: Rc::downgrade(&self.collection),
            id: collection.last_id,
            value: value
        };

        let item = Rc::new(item);
        collection.table_by_id.insert(item.id, Rc::downgrade(&item));

        WeakCollectionItem {
            item: item
        }
    }
}

struct WeakCollectionInternal<T> {
    last_id: i32,
    table_by_id: HashMap<i32, Weak<WeakCollectionItemInternal<T>>>
}

pub struct WeakCollectionItem<T> {
    item: Rc<WeakCollectionItemInternal<T>>
}

impl<T> WeakCollectionItem<T> {
    pub fn get(&self) -> &T {
        &self.item.value
    }
}

struct WeakCollectionItemInternal<T> {
    owner: Weak<RefCell<WeakCollectionInternal<T>>>,
    id: i32,
    value: T
}

impl<T> Drop for WeakCollectionItemInternal<T> {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.upgrade() {
            let mut owner = owner.borrow_mut();
            owner.table_by_id.remove(&self.id);
        }
    }
}


impl<'a, T: 'a> IntoIterator for &'a WeakCollection<T> {
    type Item = WeakCollectionItem<T>;
    type IntoIter = WeakCollectionIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        WeakCollectionIterator::new(self.collection.borrow_mut())
    }
}

pub struct WeakCollectionIterator<'a, T: 'a> {
    iter: ::std::collections::hash_map::Values<'a, i32, Weak<WeakCollectionItemInternal<T>>>,
    _rm: RefMut<'a, WeakCollectionInternal<T>>
}

impl <'a, T: 'a> WeakCollectionIterator<'a, T> {
    fn new(t: RefMut<'a, WeakCollectionInternal<T>>) -> WeakCollectionIterator<'a, T>{
        WeakCollectionIterator {
            iter: unsafe { mem::transmute(t.table_by_id.values()) },
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
