use serde::{Deserialize};
use stepflow::data::{BoolVar, EmailVar, Var, VarId, StringVar, TrueVar};


#[derive(Debug, Deserialize)]
pub enum VarSerde {
    String,
    Email,
    True,
    Bool,
}

impl VarSerde {
    pub fn to_var(self, var_id: VarId) -> Box<dyn Var + Send + Sync> {
        match self {
            VarSerde::String => StringVar::new(var_id).boxed(),
            VarSerde::Email => EmailVar::new(var_id).boxed(),
            VarSerde::True => TrueVar::new(var_id).boxed(),
            VarSerde::Bool => BoolVar::new(var_id).boxed(),
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use stepflow_test_util::test_id;
    use super::{VarSerde, BoolVar, VarId};

    #[test]
    fn basic() {
        let json = json!("Bool");
        let var_serde: VarSerde = serde_json::from_value(json).unwrap();
        let var = var_serde.to_var(test_id!(VarId));
        assert!(var.is::<BoolVar>());
    }
}