pub mod prelude {
    pub use std::convert::TryFrom;
}

mod session;
pub use session::SessionSerde;

mod data;
pub use data::StateDataSerde;

mod errors;
pub use errors::SerdeError;

mod var;
use var::VarSerde;

mod step;
use step::StepSerde;

mod action;
use action::ActionSerde;


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::TryFrom;
    use stepflow_test_util::test_id;
    use stepflow::data::{UriValue};
    use stepflow::{Session, SessionId, AdvanceBlockedOn};
    use stepflow::prelude::*;
    use super::{ SessionSerde, StateDataSerde, SerdeError};

    const JSON: &str = r#"
    {
        "vars": {
            "first_name": "String",
            "last_name": "String",
            "email": "Email",
            "email_waited": "True",
            "nothing": "Bool"
        },
        "steps": {
           "$root": {
               "substeps": ["name", "email"],
               "outputVars": ["first_name","last_name","email", "email_waited"]
           },
           "name": {
               "outputVars": ["first_name","last_name"]
           },
           "email": {
               "outputVars": ["email", "email_waited"]
           }
        },
        "stepActions": {
            "$all": {
                "type": "url",
                "baseUrl": "/base-path"
            },
            "email": {
                "type": "setData",
                "stateData": {
                    "email_waited": "true"
                },
                "afterAttempt": 2
            }
        }
    }"#;

    pub fn create_session(json: &str) -> Result<Session, SerdeError> {
        let mut session_serde: SessionSerde = serde_json::from_str(json).map_err(|_e| SerdeError::Other)?;
        session_serde.session_id = test_id!(SessionId);
        let session = Session::try_from(session_serde)?;
        Ok(session)
    }

    #[test]
    fn derserialize() {
        let mut session = create_session(JSON).unwrap();
        let name_stepid = session.step_store().get_by_name("name").unwrap().id().clone();
        let email_stepid = session.step_store().get_by_name("email").unwrap().id().clone();
        let _firstname_var_id = session.varstore().get_by_name("first_name").unwrap().id().clone();
        let _email_waited_varid = session.varstore().get_by_name("email_waited").unwrap().id().clone();
        let url_action_id = session.action_store().id_from_name("$all").unwrap();

        // advance to first step (name)
        let name_advance = session.advance(None).unwrap();
        assert_eq!(name_advance, AdvanceBlockedOn::ActionStartWith(url_action_id, "/base-path/name".parse::<UriValue>().unwrap().boxed()));

        // try advancing without setting name and fail
        let name_advance_fail = session.advance(None).unwrap();
        assert_eq!(
            name_advance_fail, 
            AdvanceBlockedOn::ActionStartWith(url_action_id, "/base-path/name".parse::<UriValue>().unwrap().boxed()));

        // advance to next step (email) - fail setdata (attempt #1) so get URL action result
        let mut data_name = HashMap::new();
        data_name.insert("first_name".to_owned(), "billy".to_owned());
        data_name.insert("last_name".to_owned(), "bob".to_owned());
        let statedata_name = StateDataSerde::new(data_name).to_statedata(session.varstore()).unwrap();
        let name_advance_success = session.advance(Some((&name_stepid,  statedata_name))).unwrap();
        assert_eq!(name_advance_success, AdvanceBlockedOn::ActionStartWith(url_action_id, "/base-path/email".parse::<UriValue>().unwrap().boxed()));

        // put in email and try advancing -- fail setdata (attempt #2) because email waited setdata action hasn't fired so get URL action result
        let mut data_email = HashMap::new();
        data_email.insert("email".to_owned(), "a@b.com".to_owned());
        let statedata_email = StateDataSerde::new(data_email).to_statedata(session.varstore()).unwrap();
        let name_advance_success = session.advance(Some((&email_stepid,  statedata_email))).unwrap();
        assert_eq!(name_advance_success, AdvanceBlockedOn::ActionStartWith(url_action_id, "/base-path/email".parse::<UriValue>().unwrap().boxed()));

        // try advancing again -- success with setdata firing and we're finished
        let name_advance_success = session.advance(None).unwrap();
        assert_eq!(name_advance_success, AdvanceBlockedOn::FinishedAdvancing);
    }

    #[test]
    fn session_ids() {
        let session1 = create_session(JSON).unwrap();
        let session2 = create_session(JSON).unwrap();
        assert_ne!(session1.id(), session2.id());
    }
}