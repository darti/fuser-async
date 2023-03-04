use datafusion::arrow::{array::UInt64Array, record_batch::RecordBatch};

use fuser_async::fuser::FileAttr;
use itertools::izip;

use crate::errors::DatafusionFsError;

pub fn to_file_attr(batches: Vec<RecordBatch>) -> Result<FileAttr, DatafusionFsError> {
    for batch in batches {
        let inos = batch.column(0).as_any().downcast_ref::<UInt64Array>();

        if let Some(inos) = inos {
            for (ino) in izip!(inos) {
                if let Some(ino) = (ino) {}
            }
        }
    }

    todo!()
}
