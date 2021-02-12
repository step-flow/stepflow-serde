use std::collections::HashMap;
use std::convert::TryFrom;
use serde::{Deserialize};
use stepflow::{data::InvalidValue, prelude::*};
use stepflow::object::ObjectStore;
use stepflow::data::VarId;
use stepflow::action::{SetDataAction, ActionId, UriAction, Uri, HtmlFormAction, HtmlFormConfig};
use stepflow::Error;
use super::StateDataSerde;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ActionSerde {
    #[serde(rename_all = "camelCase")]
    Uri {
        base_uri: String,
    },
    #[serde(rename_all = "camelCase")]
    SetData {
        #[serde(rename = "stateData")]
        data: HashMap<String, String>,    // varname -> value
        after_attempt: u32,
    },
    #[serde(rename_all = "camelCase")]
    HtmlForm {
        string_html: Option<String>,
        uri_html: Option<String>,
        email_html: Option<String>,
        bool_html: Option<String>,
        prefix_html: Option<String>, // ie. label before each input field
        wrap_tag: Option<String>, // ie. wrap entire element in a <div></div>      
    },
}

impl ActionSerde {
    pub fn to_action(self, action_id: ActionId, var_store: &ObjectStore<Box<dyn Var + Send + Sync>, VarId>) -> Result<Box<dyn Action + Sync + Send>, Error> {
        match self {
            ActionSerde::Uri { base_uri } => {
                let base_uri = Uri::try_from(base_uri).map_err(|_e| InvalidValue::BadFormat)?;
                Ok(UriAction::new(action_id, base_uri).boxed())
            }
            ActionSerde::SetData { data, after_attempt } => {
                let statedata_serde = StateDataSerde::new(data);
                let state_data = statedata_serde.to_statedata(var_store)?;
                Ok(SetDataAction::new(action_id, state_data, after_attempt.into()).boxed())
            }
            ActionSerde::HtmlForm {
                string_html: stringvar_html_template,
                uri_html: urivar_html_template,
                email_html: emailvar_html_template,
                bool_html: boolvar_html_template,
                prefix_html: prefix_html_template,
                wrap_tag
            } => {
                let mut html_config: HtmlFormConfig = Default::default();
                if let Some(stringvar_html_template) = stringvar_html_template {
                    html_config.stringvar_html_template = stringvar_html_template;
                }
                if let Some(urivar_html_template) = urivar_html_template {
                    html_config.urivar_html_template = urivar_html_template;
                }
                if let Some(emailvar_html_template) = emailvar_html_template {
                    html_config.emailvar_html_template = emailvar_html_template;
                }
                if let Some(boolvar_html_template) = boolvar_html_template {
                    html_config.boolvar_html_template = boolvar_html_template;
                }
                html_config.prefix_html_template = prefix_html_template;
                html_config.wrap_tag = wrap_tag;

                Ok(HtmlFormAction::new(action_id, html_config).boxed())
            }
        }
    }
}
