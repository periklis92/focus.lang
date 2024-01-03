use std::{any::Any, hash::Hash, marker::PhantomData};

pub trait GcObject {
    fn mark(&self, gc: &mut Gc);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct GcRef<T: GcObject> {
    index: usize,
    _marker: PhantomData<T>,
}

impl<T: GcObject> Hash for GcRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

pub struct GcObjectHeader {
    is_marked: bool,
    object: Box<dyn GcObject>,
}

pub struct Gc {
    objects: Vec<Option<GcObjectHeader>>,
    free_slots: Vec<usize>,
}

impl Gc {
    pub fn alloc<T: GcObject + 'static>(&mut self, object: T) -> GcRef<T> {
        let header = GcObjectHeader {
            is_marked: false,
            object: Box::new(object),
        };
        let index = if let Some(index) = self.free_slots.pop() {
            self.objects[index] = Some(header);
            index
        } else {
            self.objects.push(Some(header));
            self.objects.len() - 1
        };
        GcRef {
            index,
            _marker: Default::default(),
        }
    }

    pub fn free<T: GcObject + 'static>(&mut self, gc_ref: GcRef<T>) {
        if let Some(_object) = self.objects[gc_ref.index].take() {
            self.free_slots.push(gc_ref.index);
        } else {
            panic!("Free called on freed object {}", gc_ref.index);
        }
    }

    pub fn get_ref<T: GcObject + 'static>(&self, gc_ref: GcRef<T>) -> &T {
        self.objects[gc_ref.index]
            .as_ref()
            .unwrap()
            .object
            .as_any()
            .downcast_ref()
            .expect(&format!("Reference {} not found", gc_ref.index))
    }

    pub fn get_mut<T: GcObject + 'static>(&mut self, gc_ref: GcRef<T>) -> &mut T {
        self.objects[gc_ref.index]
            .as_mut()
            .unwrap()
            .object
            .as_any_mut()
            .downcast_mut()
            .expect(&format!("Reference {} not found", gc_ref.index))
    }
}
