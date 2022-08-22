use std::path::{PathBuf};

pub struct VersionConstruct {
    pub id: VersionId,
    pub at: PathBuf,
}

pub enum VersionId {
    V1_19_2,
}

impl VersionId {

    pub fn create(self, base_folder: &PathBuf) -> VersionConstruct {
        match self {
            VersionId::V1_19_2 => {
                let id = self.to_string();

                let at = base_folder.join(id);

                VersionConstruct {
                    id: self,
                    at,
                }
            }
        }
    }

}

impl ToString for VersionId {
    fn to_string(&self) -> String {
        match self {
            VersionId::V1_19_2 => "1.19.2".to_string(),
        }
    }
}