use std::hash::Hash;
use serde::Deserialize;
use stepflow::prelude::*;
use stepflow::data::VarId;
use stepflow::object::{ObjectStore, IdError};
use stepflow::step::{Step, StepId};
use stepflow::Error;


#[derive(Debug, Deserialize)]
pub struct StepSerde {
    name: Option<String>,
    #[serde(rename(deserialize = "substeps"))]
    substep_names: Option<Vec<String>>,
    #[serde(rename(deserialize = "inputVars"))]
    input_vars: Option<Vec<String>>,
    #[serde(rename(deserialize = "outputVars"))]
    output_vars: Vec<String>,
}

fn names_to_ids<T, TID>(store: &ObjectStore<T, TID>, names: &Vec<String>) -> Result<Vec<TID>, IdError<TID>>
        where T:ObjectStoreContent + ObjectStoreContent<IdType = TID>,
        TID: Eq + Hash + Clone,
{
    names.into_iter()
        .map(|name| {
            store.id_from_name(&name[..])
                .map(|id| id.clone())
                .ok_or_else(|| IdError::NoSuchName(name.clone()))
        })
        .collect()
}

impl StepSerde {
    pub fn input_var_ids(&self, varstore: &ObjectStore<Box<dyn Var + Send + Sync>, VarId>) -> Result<Option<Vec<VarId>>, IdError<VarId>> {
        match &self.input_vars {
            Some(input_vars) => Ok(Some(names_to_ids(varstore, &input_vars)?)),
            None => Ok(None),
        }
    }

    pub fn output_var_ids(&self, varstore: &ObjectStore<Box<dyn Var + Send + Sync>, VarId>) -> Result<Vec<VarId>, IdError<VarId>> {
        names_to_ids(varstore, &self.output_vars)
    }

    pub fn to_step(self, step_id: StepId, input_var_ids: Option<Vec<VarId>>, output_var_ids: Vec<VarId>) -> Result<(Step, Option<Vec<String>>), IdError<StepId>> {
        let step = Step::new(step_id, input_var_ids, output_var_ids);
        Ok((step, self.substep_names ))
    }

    pub fn add_substeps(step_id: StepId, substep_names: Vec<String>, step_store: &mut ObjectStore<Step, StepId>) -> Result<(), Error> {
        let substep_step_ids = names_to_ids(step_store, &substep_names).map_err(|e| Error::StepId(e))?;
        let step = step_store.get_mut(&step_id).ok_or_else(|| Error::StepId(IdError::IdMissing(step_id)))?;
        for substep_step_id in substep_step_ids {
            step.push_substep(substep_step_id);
        }
        Ok(())
    }

}