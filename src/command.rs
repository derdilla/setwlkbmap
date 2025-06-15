use std::process::{Command, Stdio};

/// Helpers for making [Command]s quicker to use.
pub(crate) trait ExecutableWithError {
    /// Executes a command as a child process and outputs user friendly error messages.
    fn execute_with_err(&mut self) -> Result<(), String>;

    /// Returns the shell command
    fn as_shell_cmd(&self) -> String;
}

impl ExecutableWithError for Command {
    fn execute_with_err(&mut self) -> Result<(), String> {
        self.stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to execute `{}`:\n {e}", self.as_shell_cmd()))
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    let tool_name = self.get_program().to_str().unwrap().to_string();
                    Err(format!("{tool_name} returned exit code {}", status.code().unwrap_or(-1)))
                }
            })
    }

    fn as_shell_cmd(&self) -> String {
        let tool_name = self.get_program().to_str().unwrap().to_string();
        let args = self.get_args()
            .map(|arg| arg.to_str().unwrap())
            .map(|arg| if arg.contains(" ") { format!("\"{arg}\"") } else { arg.to_string() })
            .collect::<Vec<String>>()
            .join(" ");
        if args.is_empty() {
            tool_name
        } else {
            format!("{tool_name} {args}")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::ExecutableWithError;


    #[test]
    fn creates_expected_commands() {
        assert_eq!(Command::new("neofetch").as_shell_cmd(), "neofetch".to_string());
        assert_eq!(Command::new("This")
            .arg("was")
            .arg("a Triumph")
            .arg("!").as_shell_cmd(), "This was \"a Triumph\" !")
    }
}