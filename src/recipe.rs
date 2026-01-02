use std::fmt;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct RecipeInfo {
    pub author: String,
    pub name: String,
    pub version: Option<String>,
}

impl fmt::Display for RecipeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.author, self.name)?;
        if let Some(version) = &self.version {
            write!(f, "@{}", version)?;
        }
        Ok(())
    }
}

impl From<&str> for RecipeInfo {
    fn from(source: &str) -> Self {
        // declare version or not? 有冇版本?
        let (full_name, version) = source
            .split_once('@')
            .map(|(full_name, version)| (full_name, Some(version.to_owned())))
            .unwrap_or((source, None));
        // who's the author? 哪位方家?
        let (author, name) = full_name
            .split_once('/')
            .map(|(author, name)| (author.to_owned(), name.to_owned()))
            .unwrap_or(("rime".to_owned(), normalize_recipe_name(full_name)));
        Self {
            author, name, version
        }
    }
}

fn normalize_recipe_name(name: &str) -> String {
    // normalize to contains "rime-" prefix 規範規範, 要包含 rime 數據倉庫前綴
    if name.starts_with("rime-") {
        name.to_owned()
    } else {
        format!("rime-{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_info_full_name_without_version() {
        let recipe = RecipeInfo::from("lotem/rime-zhengma");
        assert_eq!(recipe.author, "lotem");
        assert_eq!(recipe.name, "rime-zhengma");
        assert_eq!(recipe.version, None);
    }

    #[test]
    fn test_recipe_info_full_name_with_version() {
        let recipe = RecipeInfo::from("lotem/rime-octagram-data@hant");
        assert_eq!(recipe.author, "lotem");
        assert_eq!(recipe.name, "rime-octagram-data");
        assert_eq!(recipe.version, Some("hant".to_owned()));
    }

    #[test]
    fn test_recipe_info_name_only() {
        let recipe = RecipeInfo::from("luna-pinyin");
        assert_eq!(recipe.author, "rime");
        assert_eq!(recipe.name, "rime-luna-pinyin");
        assert_eq!(recipe.version, None);
    }

    #[test]
    fn test_recipe_info_normalized_name() {
        let recipe = RecipeInfo::from("rime-luna-pinyin");
        assert_eq!(recipe.author, "rime");
        assert_eq!(recipe.name, "rime-luna-pinyin");
        assert_eq!(recipe.version, None);
    }

    #[test]
    fn test_recipe_info_name_and_version_only() {
        let recipe = RecipeInfo::from("bopomofo@master");
        assert_eq!(recipe.author, "rime");
        assert_eq!(recipe.name, "rime-bopomofo");
        assert_eq!(recipe.version, Some("master".to_owned()));
    }
}
