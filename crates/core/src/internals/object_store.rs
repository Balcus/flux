use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    objects::{
        blob::Blob,
        commit::Commit,
        object_type::{FluxObject, ObjectType},
        tree::Tree,
    },
    utils,
};

pub struct ObjectStore {
    pub path: PathBuf,
}

impl ObjectStore {
    pub fn new(flux_dir: &Path) -> Self {
        let path = flux_dir.join("objects");
        fs::create_dir(&path).expect("Failed to create object storage directory");
        Self { path }
    }

    pub fn load(flux_dir: &Path) -> Self {
        let path = flux_dir.join("objects");
        Self { path }
    }

    pub fn store(&self, object: &dyn FluxObject) {
        utils::store_object(&self.path, &object.hash(), &object.serialize())
            .expect("Failed to store object");
    }

    pub fn retrieve_object(&self, hash: &str) -> Box<dyn FluxObject> {
        let object = utils::read_object(&self.path, hash).expect("Failed to read object");
        match object.object_type {
            ObjectType::Blob => Box::new(Blob::from_content(object.decompressed_content)),
            ObjectType::Tree => Box::new(Tree::from_content(object.decompressed_content)),
            ObjectType::Commit => Box::new(Commit::from_content(object.decompressed_content)),
            ObjectType::Tag => panic!("Currently not supported"),
        }
    }
}
