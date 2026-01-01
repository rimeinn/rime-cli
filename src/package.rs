use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use crate::recipe::RecipeInfo;

#[derive(Clone)]
pub struct RecipePackage<'a> {
    pub recipe: RecipeInfo,
    pub host: Option<&'a str>,
    // pub 內容文件 Vec<PathBuf>,
}

impl RecipePackage<'_> {
    pub fn repository_url(&self) -> String {
        format!(
            "https://{}/{}/{}.git",
            self.host.unwrap_or("github.com"),
            self.recipe.author,
            self.recipe.name
        )
    }

    pub fn repository_branch(&self) -> Option<&str> {
        self.recipe.version.as_deref()
    }

    pub fn local_path(&self) -> PathBuf {
        ["pkg", self.recipe.author.as_str(), self.recipe.name.as_str()]
            .iter()
            .collect()
    }

    pub fn group_by_repository<'a>(
        recipes: &[RecipeInfo],
        host: Option<&'a str>,
    ) -> HashMap<RecipeInfo, Vec<RecipePackage<'a>>> {
        let mut group = HashMap::new();
        recipes.iter().for_each(|recipe| {
            let package_name = RecipeInfo {
                version: None,
                ..recipe.clone()
            };
            group
                .entry(package_name)
                .or_insert_with(Vec::new)
                .push(RecipePackage {
                    recipe: recipe.clone(),
                    host,
                });
        });
        group
    }
}

impl fmt::Display for RecipePackage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.repository_branch() {
            Some(branch) => write!(f, "{}@{}", self.repository_url(), branch),
            None => write!(f, "{}", self.repository_url()),
        }
    }
}
