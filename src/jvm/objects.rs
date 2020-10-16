use std::rc::Rc;
use crate::class::Class;
use std::collections::HashMap;
use crate::jvm::types::JTypeValue;

pub struct Heap {
    ref_counter: usize,
    pub objects: HashMap<usize, Object>,
}

impl Heap {
    pub fn new() -> Self {
        Heap {ref_counter: 0, objects: HashMap::new()}
    }

    pub fn allocate(&mut self, obj: Object) -> usize {
        let curr_ref = self.ref_counter;
        self.objects.insert(curr_ref, obj);
        self.ref_counter += 1;
        curr_ref
    }

    pub fn get_obj(&self, obj_ref: usize) -> &Object {
        match self.objects.get(&obj_ref) {
            Some(o) => o,
            None => panic!("object not found on the heap")
        }
    }

    pub fn get_obj_mut(&mut self, obj_ref: usize) -> &mut Object {
        match self.objects.get_mut(&obj_ref) {
            Some(o) => o,
            None => panic!("object not found on the heap")
        }
    }

}

#[derive(Debug)]
pub struct Object {
    // TODO how can we hide those fields?
    pub class: Rc<Class>,
    pub fields: HashMap<usize, JTypeValue>,
}

impl Object {
    pub fn new(class: Rc<Class>) -> Self {
        Object {class, fields: HashMap::new()}
    }

    pub fn field_value(&self, index: usize) -> JTypeValue {

        match self.fields.get(&index) {
            Some(v) => *v,
            None => panic!("field value not found")
        }
    }
}

