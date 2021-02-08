use serde::{Deserialize};
use stepflow::data::{BoolVar, EmailVar, Var, VarId, StringVar, TrueVar, UriVar};


#[derive(Debug, Deserialize)]
pub enum VarSerde {
    String,
    Email,
    True,
    Uri,
    Bool,
}

impl VarSerde {
    pub fn to_var(self, var_id: VarId) -> Box<dyn Var + Send + Sync> {
        match self {
            VarSerde::String => StringVar::new(var_id).boxed(),
            VarSerde::Email => EmailVar::new(var_id).boxed(),
            VarSerde::True => TrueVar::new(var_id).boxed(),
            VarSerde::Uri => UriVar::new(var_id).boxed(),
            VarSerde::Bool => BoolVar::new(var_id).boxed(),
        }
    }
}