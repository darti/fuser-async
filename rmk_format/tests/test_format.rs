use std::fs;

use rmk_format::{
    format::{LayerBuilder, NotebookBuilder},
    notebook,
};

// #[test]
// fn test_version_6() -> anyhow::Result<()> {
//     let bytes = fs::read(
//         "../data/version_6/0d9af7de-39f8-4251-8500-330eec0d00f0/e3c22b43-bd2c-42d8-8b45-d9bdf829b500.rm",
//     )?;

//     let notebook = notebook(&bytes)?;

//     assert_eq!(
//         notebook,
//         NotebookBuilder::default()
//             .version(6)
//             .nb_layers(1)
//             .build()
//             .unwrap()
//     );

//     Ok(())
// }

#[test]
fn test_version_5() -> anyhow::Result<()> {
    let bytes = fs::read(
        "../data/version_5/0d9af7de-39f8-4251-8500-330eec0d00f0/bd99b4ce-4dec-40af-896c-648ecdad5db9.rm",
    )?;

    let notebook = notebook(&bytes)?;

    assert_eq!(
        notebook,
        NotebookBuilder::default()
            .version(5)
            .layer(LayerBuilder::default().build().unwrap())
            .build()
            .unwrap()
    );

    Ok(())
}
