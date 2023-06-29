use smithay::output::Output;

use crate::state::CONFIG;

pub struct Workspaces {
    workspaces: Vec<Workspace>,
}

impl Workspaces {
    fn create_workspaces() -> Self {
        let workspaces: Vec<Workspace> = (0..CONFIG.workspaces).map(|_| Workspace::new()).collect();
        Workspaces { workspaces }
    }
}

pub struct Workspace {
    outputs: Vec<Output>,
}

impl Workspace {
    fn new() -> Self {
        let outputs = Vec::new();
        Workspace { outputs }
    }
}
