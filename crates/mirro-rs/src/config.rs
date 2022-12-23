use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    cli::SelectionSort,
    tui::dispatch::{
        filter::Filter,
        sort::{ExportSort, ViewSort},
    },
};

pub struct Configuration {
    pub outfile: PathBuf,
    pub export: u16,
    pub filters: Vec<Filter>,
    pub view: ViewSort,
    pub sort: ExportSort,
    pub country: Vec<String>,
    pub ttl: u16,
}

impl Configuration {
    pub fn new(
        outfile: PathBuf,
        export: u16,
        filters: Vec<Filter>,
        view: ViewSort,
        sort: SelectionSort,
        country: Vec<String>,
        ttl: u16,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            outfile,
            export,
            filters,
            view,
            sort: match sort {
                SelectionSort::Percentage => ExportSort::Completion,
                SelectionSort::Delay => ExportSort::MirroringDelay,
                SelectionSort::Duration => ExportSort::Duration,
                SelectionSort::Score => ExportSort::Score,
            },
            country,
            ttl,
        }))
    }
}
