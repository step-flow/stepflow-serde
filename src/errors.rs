use stepflow::data::VarId;
use stepflow::step::StepId;
use stepflow::action::ActionId;
use stepflow::SessionId;
use stepflow::object::IdError;
use stepflow::Error;

#[derive(Debug)]
pub enum SerdeError {
  Error(Error),
  MissingRootStep,
  Other,
}

impl From<Error> for SerdeError {
  fn from(err: Error) -> Self {
    SerdeError::Error(err)
  }
}

macro_rules! from_id_error {
  ($id_type:ident) => {
    impl From<IdError<$id_type>> for SerdeError {
      fn from(err: IdError<$id_type>) -> Self {
        SerdeError::Error(Error::$id_type(err))
      }
    }
  };
}

from_id_error!(VarId);
from_id_error!(StepId);
from_id_error!(ActionId);
from_id_error!(SessionId);