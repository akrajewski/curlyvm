use std::rc::Rc;
use crate::class::Class;
use std::collections::HashMap;
use crate::jvm::types::JTypeValue;




pub struct Heap {
    ref_counter: usize,
    pub objects: HashMap<usize, Object>,
    pub arrays: HashMap<usize, Array>,
}

impl Heap {
    pub fn new() -> Self {
        // Start reference counting from 1, 0 is considered NULL
        Heap {ref_counter: 1, objects: HashMap::new(), arrays: HashMap::new() }
    }

    pub fn allocate_arr(&mut self, arr: Array) -> usize {
        let curr_ref = self.ref_counter;
        self.arrays.insert(curr_ref, arr);
        self.ref_counter += 1;
        curr_ref
    }

    pub fn get_arr(&self, arr_ref: usize) -> &Array {
        match self.arrays.get(&arr_ref) {
            Some(arr) => arr,
            None => panic!("array not found on the heap")
        }
    }

    pub fn get_arr_mut(&mut self, arr_ref: usize) -> &mut Array {
        match self.arrays.get_mut(&arr_ref) {
            Some(arr) => arr,
            None => panic!("array not found on the heap!"),
        }
    }

    pub fn allocate_obj(&mut self, obj: Object) -> usize {
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

pub struct Array {
    arr: Vec<JTypeValue>
}

impl Array {

    pub fn new(count: usize) -> Self {
        Array { arr: vec![JTypeValue::Empty; count] }
    }

    pub fn set(&mut self, index: usize, val: JTypeValue) {
        if index >= self.arr.len() {
            panic!("trying to insert into array with ArrayIndexOutOfBoundsException");
        }

        self.arr[index] = val;
    }

    pub fn get_int(&self, index: usize) -> i32 {
        match self.arr[index] {
            JTypeValue::Int(i) => i,
            _ => panic!("tried to get int from array but it is not an int")
        }
    }



}

