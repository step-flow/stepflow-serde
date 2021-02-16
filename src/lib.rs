//! Provides structures for [`Serde`](::serde) to simplify deserialization of a [`Session`](stepflow::Session)
//!
//! The main object to use with `Serde` is [`SessionSerde`].
//!
//! # Examples
//! ```
//! # use stepflow::SessionId;
//! # use stepflow_serde::SessionSerde;
//! const JSON: &str = r#"
//! {
//!     "vars": {
//!         "name": "String",
//!         "email": "Email"
//!     },
//!     "steps": {
//!        "$root": {
//!            "substeps": ["nameStep", "emailStep"],
//!            "outputs": ["name","email"]
//!        },
//!        "nameStep": {
//!            "outputs": ["name"]
//!        },
//!        "emailStep": {
//!            "outputs": ["email"]
//!        }
//!     },
//!     "actions": {
//!         "$all": { "type": "htmlForm" }
//!     }
//! }"#;
//!
//! // Parse JSON to a Session
//! let session_serde: SessionSerde = serde_json::from_str(JSON).unwrap();
//! let session = session_serde.into_session::<serde_json::Error>(SessionId::new(0), false).unwrap();
//!
//! ```

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
    use serde_json::json;
    use stepflow_test_util::test_id;
    use stepflow::data::{StringVar, UriValue};
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
               "outputs": ["first_name","last_name","email", "email_waited"]
           },
           "name": {
               "outputs": ["first_name","last_name"]
           },
           "email": {
               "outputs": ["email", "email_waited"]
           }
        },
        "actions": {
            "$all": {
                "type": "uri",
                "baseUri": "/base-path"
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

    pub fn create_session(json: &str, allow_implicit_var: bool) -> Result<Session, SerdeError<serde_json::Error>> {
        let session_serde: SessionSerde = serde_json::from_str(json).map_err(|e| SerdeError::InvalidFormat(e))?;
        let session = session_serde.into_session(test_id!(SessionId), allow_implicit_var)?;
        Ok(session)
    }

    #[test]
    fn derserialize() {
        let mut session = create_session(JSON, false).unwrap();
        let name_stepid = session.step_store().get_by_name("name").unwrap().id().clone();
        let email_stepid = session.step_store().get_by_name("email").unwrap().id().clone();
        let _firstname_var_id = session.var_store().get_by_name("first_name").unwrap().id().clone();
        let _email_waited_varid = session.var_store().get_by_name("email_waited").unwrap().id().clone();
        let uri_action_id = session.action_store().id_from_name("$all").unwrap();

        // advance to first step (name)
        let name_advance = session.advance(None).unwrap();
        assert_eq!(name_advance, AdvanceBlockedOn::ActionStartWith(uri_action_id, "/base-path/name".parse::<UriValue>().unwrap().boxed()));

        // try advancing without setting name and fail
        let name_advance_fail = session.advance(None).unwrap();
        assert_eq!(
            name_advance_fail, 
            AdvanceBlockedOn::ActionStartWith(uri_action_id, "/base-path/name".parse::<UriValue>().unwrap().boxed()));

        // advance to next step (email) - fail setdata (attempt #1) so get URI action result
        let mut data_name = HashMap::new();
        data_name.insert("first_name".to_owned(), "billy".to_owned());
        data_name.insert("last_name".to_owned(), "bob".to_owned());
        let statedata_name = StateDataSerde::new(data_name).to_statedata(session.var_store()).unwrap();
        let name_advance_success = session.advance(Some((&name_stepid,  statedata_name))).unwrap();
        assert_eq!(name_advance_success, AdvanceBlockedOn::ActionStartWith(uri_action_id, "/base-path/email".parse::<UriValue>().unwrap().boxed()));

        // put in email and try advancing -- fail setdata (attempt #2) because email waited setdata action hasn't fired so get URI action result
        let mut data_email = HashMap::new();
        data_email.insert("email".to_owned(), "a@b.com".to_owned());
        let statedata_email = StateDataSerde::new(data_email).to_statedata(session.var_store()).unwrap();
        let name_advance_success = session.advance(Some((&email_stepid,  statedata_email))).unwrap();
        assert_eq!(name_advance_success, AdvanceBlockedOn::ActionStartWith(uri_action_id, "/base-path/email".parse::<UriValue>().unwrap().boxed()));

        // try advancing again -- success with setdata firing and we're finished
        let name_advance_success = session.advance(None).unwrap();
        assert_eq!(name_advance_success, AdvanceBlockedOn::FinishedAdvancing);
    }

    #[test]
    fn session_ids() {
        let session1 = create_session(JSON, false).unwrap();
        let session2 = create_session(JSON, false).unwrap();
        assert_ne!(session1.id(), session2.id());
    }

    #[test]
    fn implicit_vars() {
        let json = json!({
            "steps": {
                "$root": {
                    "substeps": ["step1"],
                    "outputs": ["test_output"]
                },
                "step1": { "inputs": ["test_input"], "outputs": ["test_output"] }
            },
            "actions": {
                "$all": { "type": "htmlForm" }
            }
        });
        let json = json.to_string();

        // expect error when we don't allow implicit var
        assert!(matches!(create_session(&json[..], false), Err(_)));

        // create session
        let session = create_session(&json[..], true).unwrap();

        assert_eq!(session.var_store().iter_names().count(), 2);
        let input_var = session.var_store().get_by_name("test_input").unwrap();
        assert!(input_var.is::<StringVar>());
        let output_var = session.var_store().get_by_name("test_output").unwrap();
        assert!(output_var.is::<StringVar>());
    }
}