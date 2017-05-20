use session::Session;
use rpc::*;
use rmpv::Value;
use neovim_api::NeovimApi;
use std::fmt;
use std::error::Error;

pub struct Neovim {
    pub session: Session,
}

pub struct UiAttachOptions {
    rgb: bool,
    popupmenu_external: bool,
    tabline_external: bool,
}

impl UiAttachOptions {
    pub fn new() -> UiAttachOptions {
        UiAttachOptions {
            rgb: true,
            popupmenu_external: false,
            tabline_external: false,
        }
    }

    pub fn set_rgb(&mut self, rgb: bool) {
        self.rgb = rgb;
    }

    pub fn set_popupmenu_external(&mut self, popupmenu_external: bool) {
        self.popupmenu_external = popupmenu_external;
    }

    pub fn set_tabline_external(&mut self, tabline_external: bool) {
        self.tabline_external = tabline_external;
    }

    fn to_value_map(&self) -> Value {
        Value::Map(vec![(Value::from("rgb"), Value::from(self.rgb)),
                        (Value::from("popupmenu_external"), Value::from(self.popupmenu_external)),
                        (Value::from("ext_tabline"), Value::from(self.tabline_external))])
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CallError {
    GenericError(String),
    NeovimError(u64, String),
}

impl fmt::Display for CallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CallError::GenericError(ref s) => write!(f, "Unknown error type: {}", s),
            CallError::NeovimError(id, ref s) => write!(f, "{} - {}", id, s),
        }
    }
}

impl Error for CallError {
    fn description(&self) -> &str {
        match *self {
            CallError::GenericError(ref s) => s,
            CallError::NeovimError(_, ref s) => s,
        }
    }
}


#[doc(hidden)]
pub fn map_generic_error(err: Value) -> CallError {
    match err {
        Value::String(val) => CallError::GenericError(val.as_str().unwrap().to_owned()),
        Value::Array(arr) => {
            if arr.len() == 2 {
                match (&arr[0], &arr[1]) {
                    (&Value::Integer(ref id), &Value::String(ref val)) => {
                        CallError::NeovimError(id.as_u64().unwrap(),
                                               val.as_str().unwrap().to_owned())
                    }
                    _ => CallError::GenericError(format!("{:?}", arr)),
                }
            } else {
                CallError::GenericError(format!("{:?}", arr))
            }
        }
        val => CallError::GenericError(format!("{:?}", val)),
    }
}

#[doc(hidden)]
pub fn map_result<T: FromVal<Value>>(val: Value) -> T {
    T::from_val(val)
}

impl Neovim {
    pub fn new(session: Session) -> Neovim {
        Neovim { session: session }
    }

    /// Register as a remote UI.
    ///
    /// After this method is called, the client will receive redraw notifications.
    pub fn ui_attach(&mut self,
                     width: u64,
                     height: u64,
                     opts: UiAttachOptions)
                     -> Result<(), CallError> {
        self.session
            .call("nvim_ui_attach",
                  &call_args!(width, height, opts.to_value_map()))
            .map_err(map_generic_error)
            .map(|_| ())
    }

    /// Send a quit command to Nvim.
    /// The quit command is 'qa!' which will make Nvim quit without
    /// saving anything.
    pub fn quit_no_save(&mut self) -> Result<(), CallError> {
        self.command("qa!")
    }
}
