use opendal::{services::Fs, Operator};
use snafu::prelude::*;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Error creating operator: {}", source))]
    OperatorCreation { source: opendal::Error },
}

#[tracing::instrument]
pub fn mount_rmk() -> Result<(), Error> {
    let builder = Fs::default().root(".");
    let op = Operator::new(builder)
        .context(OperatorCreationSnafu)?
        .finish();

    Ok(())
}
