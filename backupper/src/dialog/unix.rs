use std::process::{Command, Output};

use crate::dialog::DialogResult;

pub fn retry_dialog(title: &str, msg: &str) -> DialogResult {
    let result = Command::new("zenity")
        .arg("--question")
        .arg(format!("--title={}", title))
        .arg(format!("--text={}", msg))
        .arg("--ok-label=Retry")
        .arg("--cancel-label=Cancel")
        .output();

    if let Ok(Output { status, .. }) = result {
        if status.code().unwrap_or(1) == 0 {
            return DialogResult::Retry;
        }
    }
    DialogResult::Cancel
}

#[cfg(test)]
mod unix_dialog_test {
    use super::*;

    #[ignore = "starts interactive window; doesn't actually test anything"]
    #[test]
    fn retry_dialog_test() {
        let choice = retry_dialog("CLICK ON CANCEL", "CLICK ON CANCEL");
        assert_eq!(choice, DialogResult::Cancel);

        let choice = retry_dialog("CLICK ON RETRY", "CLICK ON RETRY");
        assert_eq!(choice, DialogResult::Retry);
    }
}
