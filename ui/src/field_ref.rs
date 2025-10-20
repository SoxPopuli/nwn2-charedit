use crate::error::Error;
use nwn_lib::files::gff::{field::Field, r#struct::StructField};

#[derive(Debug, Clone)]
pub struct FieldRef<T> {
    pub field: StructField,
    pub value: T,
}
impl<T> FieldRef<T> {
    pub fn new<E>(
        field: StructField,
        expect_fn: impl FnOnce(&Field) -> Result<T, E>,
    ) -> Result<Self, Error>
    where
        E: Into<Error>,
    {
        let lock = field.read()?;
        let value = expect_fn(&lock.field).map_err(|e| e.into())?;
        drop(lock);

        Ok(Self {
            field: field.clone(),
            value,
        })
    }

    pub fn set(&mut self, new_value: T, save_fn: impl FnOnce(&T) -> Field) {
        self.value = new_value;

        let mut lock = self.field.write().unwrap();
        lock.field = save_fn(&self.value);
    }

    pub fn modify(&mut self, modify_fn: impl FnOnce(&mut T), save_fn: impl FnOnce(&T) -> Field) {
        modify_fn(&mut self.value);

        let mut lock = self.field.write().unwrap();
        lock.field = save_fn(&self.value);
    }

    pub fn get(&self) -> &T {
        &self.value
    }
}
