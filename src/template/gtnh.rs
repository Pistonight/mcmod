use std::{collections::BTreeMap, io};

use async_trait::async_trait;

use crate::util::{IoResult, Project};
use crate::gradle;

use super::TemplateHandler;

pub struct Gtnh1710Handler;
#[async_trait(?Send)]
impl TemplateHandler for Gtnh1710Handler {
    async fn setup_project(&self, _project: &Project) -> IoResult<()> {
        Ok(())
    }

    async fn run_gradlew(&self, project: &Project, args: &[&str]) -> IoResult<()> {
        let mut java_version = 8;
        if let Some(arg) = args.first() {
            if arg.ends_with("17") {
                java_version = 17;
            }
        }
        gradle::run_gradlew(&project.target_root(), java_version, args).await
    }

    async fn make_gradle_properties(&self, project: &Project) -> IoResult<BTreeMap<String, String>> {
        let mcmod = project.mcmod().await?;

        if !mcmod.version.is_empty() || !mcmod.artifact_version.is_empty() {
            Err(io::Error::new(io::ErrorKind::Other, "Version is automatically determined from git for this template. Remove the versions in mcmod.yaml"))?;
        }

        let mut map = BTreeMap::new();
        map.insert("modName".to_owned(), mcmod.name.clone());
        map.insert("modId".to_owned(), mcmod.modid.clone());
        map.insert("modGroup".to_owned(), mcmod.group.clone());
        map.insert("customArchivesBaseName".to_owned(), mcmod.archives_base_name.clone());
        map.insert("generateGradleTokenClass".to_owned(), format!("{}.Tags", mcmod.group));

        let group_prefix = format!("{}.", mcmod.group);

        if mcmod.api.is_empty() {
            map.insert("apiPackage".to_owned(), "".to_owned());
        } else {
            match mcmod.api.strip_prefix(&group_prefix) {
                Some(x) => {
                    map.insert("apiPackage".to_owned(), x.to_owned());
                },
                None => {
                    Err(io::Error::new(io::ErrorKind::Other, format!("api package must be in the same group as the mod ('{}')", mcmod.group)))?
                }
            }
        }

        let ats = mcmod.access_transformers.join(" ");
        map.insert("accessTransformersFile".to_owned(), ats);

        if mcmod.mixins.is_empty() {
            map.insert("useMixins".to_owned(), "false".to_owned());
            map.insert("mixinsPackage".to_owned(), "".to_owned());
        } else {
            map.insert("useMixins".to_owned(), "true".to_owned());
            match mcmod.mixins.strip_prefix(&group_prefix) {
                Some(x) => {
                    map.insert("mixinsPackage".to_owned(), x.to_owned());
                },
                None => {
                    Err(io::Error::new(io::ErrorKind::Other, format!("mixins package must be in the same group as the mod ('{}')", mcmod.group)))?
                }
            }
        }

        if mcmod.coremod.is_empty() {
            map.insert("coreModClass".to_owned(), "".to_owned());
        } else {
            match mcmod.coremod.strip_prefix(&group_prefix) {
                Some(x) => {
                    map.insert("coreModClass".to_owned(), x.to_owned());
                },
                None => {
                    Err(io::Error::new(io::ErrorKind::Other, format!("coremod class must be in the same group as the mod ('{}')", mcmod.group)))?
                }
            }
        }

        Ok(map)
    }
}
