use std::borrow::BorrowFrom;
use std::collections::HashMap;
use std::collections::hash_map::Hasher;
use std::hash::Hash;


pub trait ItemGetter<'obj, Key, Value> 
    where Key: Sized + Eq + BorrowFrom<Key>, Value: ToString + 'obj
{
    fn get_value(&'obj self, key: &Key) -> Option<&Value>;
}


impl <'obj, Key, Value> ItemGetter<'obj, Key, Value> + 'obj
    where Key: Sized + Eq + BorrowFrom<Key>, Value: ToString + 'obj
{
    pub fn get_string(&'obj self, key: &Key) -> String {
        match self.get_value(key) {
            Some(s) => s.to_string(),
            None => "".to_string(),
        }
    }
}


impl<'obj, Key, Value> ItemGetter<'obj, Key, Value> for HashMap<Key, Value>
    where Key: Hash<Hasher> + Eq + BorrowFrom<Key>, Value: ToString + 'obj
{
    fn get_value(&'obj self, key: &Key) -> Option<&Value> {
        self.get(key)
    }
} 


impl<'obj, Value: ToString + 'obj> ItemGetter<'obj, usize, Value> for Vec<Value> {
    fn get_value(&'obj self, key: &usize) -> Option<&Value> {
        self.get(*key)
    }
}
