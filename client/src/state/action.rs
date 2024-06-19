#[derive(Clone, Debug)]
pub(crate) enum Action {
    ConnectAndLogin { name: String },
    SendMessage { message: String },
    Quit,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectAndLogin { name } => write!(f, "Connect and login @{name}"),
            Self::SendMessage { message } => write!(f, "Send message '{message}'"),
            Self::Quit => write!(f, "Quit"),
        }
    }
}
