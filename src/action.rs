use std::collections::HashMap;
use serde::{Deserialize};
use stepflow::prelude::*;
use stepflow::object::ObjectStore;
use stepflow::data::VarId;
use stepflow::action::{SetDataAction, StringTemplateAction, HtmlEscapedString, UriEscapedString, ActionId, HtmlFormAction, HtmlFormConfig};
use stepflow::Error;
use super::StateDataSerde;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StringEscape {
    Html,
    Uri,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ActionSerde {
    #[serde(rename_all = "camelCase")]
    StringTemplate {
        template: String,
        escape_for: StringEscape,
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
        email_html: Option<String>,
        bool_html: Option<String>,
        prefix_html: Option<String>, // ie. label before each input field
        wrap_tag: Option<String>, // ie. wrap entire element in a <div></div>      
    },
}

impl ActionSerde {
    pub fn to_action(self, action_id: ActionId, var_store: &ObjectStore<Box<dyn Var + Send + Sync>, VarId>) -> Result<Box<dyn Action + Sync + Send>, Error> {
        match self {
            ActionSerde::StringTemplate { template, escape_for } => {
                Ok(
                    match escape_for {
                        StringEscape::Html =>
                            StringTemplateAction::new(action_id, HtmlEscapedString::already_escaped(template)).boxed(),
                        StringEscape::Uri => 
                            StringTemplateAction::new(action_id, UriEscapedString::already_escaped(template)).boxed(),
                    }
                )
            }
            ActionSerde::SetData { data, after_attempt } => {
                let statedata_serde = StateDataSerde::new(data);
                let state_data = statedata_serde.to_statedata(var_store)?;
                Ok(SetDataAction::new(action_id, state_data, after_attempt.into()).boxed())
            }
            ActionSerde::HtmlForm {
                string_html: stringvar_html_template,
                email_html: emailvar_html_template,
                bool_html: boolvar_html_template,
                prefix_html: prefix_html_template,
                wrap_tag
            } => {
                let mut html_config: HtmlFormConfig = Default::default();
                if let Some(stringvar_html_template) = stringvar_html_template {
                    html_config.stringvar_html_template = stringvar_html_template;
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
