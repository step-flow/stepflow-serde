use std::collections::HashMap;
use serde::{Deserialize};
use stepflow::prelude::*;
use stepflow::object::IdError;
use stepflow::{Session, SessionId, Error};

use super::{VarSerde, StepSerde, ActionSerde, SerdeError};


// we reserve anything with prefixed with a $.. maybe someday we'll enforce it.
const NAME_GLOBAL_ACTION: &str = "$all";
const NAME_ROOT_STEP: &str = "$root";


#[derive(Debug, Deserialize)]
#[serde(rename = "Session")]
pub struct SessionSerde {
    vars: Option<HashMap<String, VarSerde>>,
    steps: HashMap<String, StepSerde>,
    actions: HashMap<String, ActionSerde>,
}

impl SessionSerde {
    /// Converts into a [`Session`]
    ///
    /// if `allow_implicit_var` is `true`, when a [`Step`](stepflow::step::Step) is parsed and uses a variable that has
    /// not been declared in the `vars` section, a [`StringVar`](stepflow::data::StringVar) will be created.
    pub fn into_session<T>(self, session_id: SessionId, allow_implicit_var: bool) -> Result<Session, SerdeError<T>> {
        let mut session = Session::with_capacity(
            session_id,
            if let Some(vars) = &self.vars { vars.len() } else { 0 },
            self.steps.len(),
            self.actions.len()
        );

        // Create Vars
        if let Some(vars) = self.vars {
            for (var_name, var_serde) in vars {
                session.var_store_mut().insert_new_named(var_name, |var_id| {
                    Ok(var_serde.to_var(var_id))
                })?;
            }
        }

        // Create implicit vars from Steps
        if allow_implicit_var {
            for (_step_name, step_serde) in &self.steps {
                step_serde.ensure_all_vars(&mut session)?;
            }
        }

        // Create Steps
        // steps in 2 passes.
        // 1. register just the steps, no sub-steps since it's possible they'll be registered later
        // 2. once all the steps are registered, assign the child sub-steps
        let mut stepid_to_substep_names = HashMap::with_capacity(self.steps.len());
        for (step_name, step_serde) in self.steps {
            let var_store = session.var_store();
            
            let input_var_ids = step_serde.input_var_ids(var_store)?;
            let output_var_ids = step_serde.output_var_ids(var_store)?;

            session.step_store_mut().insert_new_named(step_name, |step_id| {
                let (step, substep_names) = step_serde.to_step(step_id, input_var_ids, output_var_ids)?;
                stepid_to_substep_names.insert(step.id().clone(), substep_names);
                Ok(step)
            })?;
        }
        for (step_id, substep_names) in stepid_to_substep_names {
            if let Some(substep_names) = substep_names {
                StepSerde::add_substeps(step_id, substep_names, session.step_store_mut())?;
            }
        }

        // Set Root Step
        let root_step_id = session.step_store()
            .id_from_name(NAME_ROOT_STEP)
            .ok_or_else(|| Error::StepId(IdError::NoSuchName(NAME_ROOT_STEP.to_owned())))?.clone();
        session.push_root_substep(root_step_id);

        // Set actions
        for (step_name, action_serde) in self.actions {
            let action_id = session.action_store_mut().reserve_id();
            let action = action_serde.to_action(action_id, session.var_store())?;
            session.action_store_mut().register_named::<String>(step_name.clone(), action)?;
            if step_name.eq(NAME_GLOBAL_ACTION) {
                session.set_action_for_step(action_id, None)?;
            } else {
                let step_id = session.step_store().id_from_name(&step_name[..]).ok_or_else(|| Error::StepId(IdError::NoSuchName(step_name)))?.clone();
                session.set_action_for_step(action_id, Some(&step_id))?;
            }
        }

        // Return session
        Ok(session)
    }
}