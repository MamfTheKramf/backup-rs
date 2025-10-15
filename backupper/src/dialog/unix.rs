use std::process::Command;

use crate::dialog::DialogResult;

pub fn retry_dialog(title: &str, msg: &str) -> DialogResult {
    let result = Command::new("zenity")
        .arg("--question")
        .arg(format!("--title={}", title))
        .arg(format!("--text={}", msg))
        .arg("--ok-label=Retry")
        .arg("--cancel-label=Cancel")
        .output();

    let choice = result.ok().and_then(|output| output.status.code());
    if let Some(0) = choice {
        DialogResult::Retry
    } else {
        DialogResult::Cancel
    }
}

pub fn info_dialog(title: &str, msg: &str) -> DialogResult {
    let result = Command::new("zenity")
        .arg("--info")
        .arg(format!("--title={}", title))
        .arg(format!("--text={}", msg))
        .output();

    let choice = result.ok().and_then(|output| output.status.code());
    if let Some(0) = choice {
        DialogResult::OK
    } else {
        DialogResult::Unknown
    }
}

#[cfg(test)]
mod unix_dialog_test {
    use super::*;

    #[ignore = "starts interactive window"]
    #[test]
    fn retry_dialog_test() {
        let choice = retry_dialog("CLICK ON CANCEL", "CLICK ON CANCEL");
        assert_eq!(choice, DialogResult::Cancel);

        let choice = retry_dialog("CLICK ON RETRY", "CLICK ON RETRY");
        assert_eq!(choice, DialogResult::Retry);
    }

    #[ignore = "starts interactive window"]
    #[test]
    fn info_dialog_test() {
        let choice = info_dialog("CLICK ON OK", "CLICK ON OK");
        assert_eq!(choice, DialogResult::OK);
    }
}
