use anyhow::Result;

use starduck::Directive;

pub fn execute_directive(directive: &Directive) -> Result<String> {
    match directive {
        Directive::Addition { .. } => execute_addition(directive),
        Directive::Reconfigure { .. } => execute_reconfig(directive),
    }
}

fn execute_addition(directive: &Directive) -> Result<String> {
    todo!()
}

fn execute_reconfig(directive: &Directive) -> Result<String> {
    todo!()
}
