use std::collections::HashMap;
use stepflow::prelude::*;
use stepflow::object::{ObjectStore, IdError};
use stepflow::data::{StateData, VarId};
use stepflow::Error;


pub struct StateDataSerde {
  data: HashMap<String, String>,
}

impl StateDataSerde {
  pub fn new(data: HashMap<String, String>) -> Self {
      StateDataSerde { data }
  }

  pub fn to_statedata(self, var_store: &ObjectStore<Box<dyn Var + Send + Sync>, VarId>) -> Result<StateData, Error> {
      let mut state_data = StateData::new();
      for (var_name, val_str) in self.data {
          let var = var_store.get_by_name(&var_name[..]).ok_or_else(|| Error::VarId(IdError::NoSuchName(var_name)))?;
          let val = var.value_from_str(&val_str[..])?;
          state_data.insert(var, val)?;
      }
      Ok(state_data)
  }
}
