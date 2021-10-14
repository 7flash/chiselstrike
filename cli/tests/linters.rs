// SPDX-FileCopyrightText: © 2021 ChiselStrike <info@chiselstrike.com>

#[cfg(test)]
mod tests {
    use std::process::Command;
    #[test]
    fn check_formating() {
        let status = Command::new("cargo")
            .args(["fmt", "--all", "--", "--check"])
            .status()
            .unwrap();
        assert!(status.success());
    }
}
