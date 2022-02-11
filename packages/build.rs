use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn run_in<T: IntoIterator<Item = &'static str>>(cmd: &str, args: T, dir: PathBuf) {
    assert!(
        dir.exists(),
        "{:?} does not exist. Current directory is {:?}",
        dir,
        env::current_dir().unwrap()
    );
    assert!(dir.is_dir(), "{:?} is not a directory", dir);
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir.clone())
        .status();
    assert!(
        status.is_ok(),
        "failed to run command `{}` in dir {:?}, error: {:?}",
        cmd,
        dir,
        status.err().unwrap()
    );
    assert!(status.unwrap().success());
}

fn main() {
    let create_app = Path::new("./create-chiselstrike-app").to_path_buf();
    let api = Path::new("./chiselstrike-api").to_path_buf();

    // build create-chiselstrike-app so we can use it in tests
    run_in("npm", ["install"], create_app.clone());
    run_in("npm", ["run", "build"], create_app);

    run_in("npm", ["install"], api.clone());
    run_in("npm", ["run", "build"], api);
}