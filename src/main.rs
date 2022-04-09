mod client;
mod context;
mod errors;
mod transaction;

use context::Context;

fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("Expected file name as argument");

    let path = std::path::Path::new(&filename);
    let mut context = match Context::from_csv(path) {
        Ok(context) => context,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let _ = context.batch();
    context.write_as_csv(std::io::stdout());
}
