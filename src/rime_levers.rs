use anyhow::{anyhow, bail};
use rime::{
    rime_api_call, rime_module_call, rime_struct_new, RimeConfig, RimeLeversApi, RimeTraits,
};
use std::ffi::{CStr, CString};
use std::path::PathBuf;

pub fn setup_engine_traits(workspace: &PathBuf) -> anyhow::Result<()> {
    log::debug!("設置引擎啓動參數. 工作場地: {}", workspace.display());
    std::fs::create_dir_all(workspace)?;
    let c_workspace = CString::new(workspace.to_str().ok_or(anyhow!("路徑編碼轉換錯誤"))?)?;
    let c_distribution = CString::new(env!("CARGO_PKG_NAME"))?;
    let c_version = CString::new(env!("CARGO_PKG_VERSION"))?;
    let mut traits: RimeTraits = rime_struct_new!();
    traits.data_size = std::mem::size_of::<RimeTraits>() as std::ffi::c_int;
    traits.shared_data_dir = c_workspace.as_ptr();
    traits.user_data_dir = c_workspace.as_ptr();
    traits.distribution_name = c_distribution.as_ptr();
    traits.distribution_code_name = c_distribution.as_ptr();
    traits.distribution_version = c_version.as_ptr();
    rime_api_call!(setup, &mut traits);
    Ok(())
}

pub fn build_binaries() -> anyhow::Result<()> {
    log::debug!("製備輸入法固件");
    rime_api_call!(deployer_initialize, std::ptr::null_mut());
    rime_api_call!(deploy);
    rime_api_call!(finalize);
    Ok(())
}

pub fn apply_patch(target_config: &str, key: &str, value: &str) -> anyhow::Result<()> {
    log::debug!("配置補丁: {target_config}:/{key} = {value}");

    let c_target_config = CString::new(target_config)?;
    let c_key = CString::new(key)?;
    let c_value = CString::new(value)?;

    let mut config: RimeConfig = rime_struct_new!();
    if rime_api_call!(config_load_string, &mut config, c_value.as_ptr()) == 0 {
        bail!("無效的 YAML 值: {}", value);
    }

    let c_levers_module_name = CString::new("levers")?;
    let levers = rime_api_call!(find_module, c_levers_module_name.as_ptr());
    if levers.is_null() {
        bail!("沒有 levers 模塊");
    }

    let c_setup_tool_name = CString::new("rime-cli")?;
    let custom_settings = rime_module_call!(
        levers => RimeLeversApi,
        custom_settings_init,
        c_target_config.as_ptr(),
        c_setup_tool_name.as_ptr()
    );

    // 可能已有自定義配置, 先加載
    rime_module_call!(levers => RimeLeversApi, load_settings, custom_settings);
    // 生成補丁
    if rime_module_call!(
        levers => RimeLeversApi,
        customize_item,
        custom_settings,
        c_key.as_ptr(),
        &mut config
    ) != 0
    {
        rime_module_call!(levers => RimeLeversApi, save_settings, custom_settings);
        log::info!("補丁打好了. {target_config}:/{key}");
    }

    rime_module_call!(levers => RimeLeversApi, custom_settings_destroy, custom_settings);
    rime_api_call!(config_close, &mut config);

    Ok(())
}

pub fn add_to_schema_list(schemata: &[String]) -> anyhow::Result<()> {
    log::debug!("加入輸入方案列表: {:#?}", schemata);
    rime_api_call!(deployer_initialize, std::ptr::null_mut());

    let mut default_custom: RimeConfig = rime_struct_new!();
    let c_default_custom_name = CString::new("default.custom")?;
    rime_api_call!(
        user_config_open,
        c_default_custom_name.as_ptr(),
        &mut default_custom
    );
    let mut exists_schemata = vec![];
    let c_schema_list = CString::new("patch/schema_list")?;
    let exists_schema_count = rime_api_call!(config_list_size, &mut default_custom, c_schema_list.as_ptr()) as u64;
    for i in 0..exists_schema_count {
        let c_schema_list_item = CString::new(format!("patch/schema_list/@{}/schema", i))?;
        let schema = rime_api_call!(config_get_cstring, &mut default_custom, c_schema_list_item.as_ptr());
        if !schema.is_null() {
            exists_schemata.push(unsafe { CStr::from_ptr(schema) }.to_str()?.to_owned());
        }
    }
    let new_schemata = schemata.iter().filter(|schema| !exists_schemata.contains(schema));
    let c_new_schema_list_item = CString::new("patch/schema_list/@next/schema")?;
    for schema in new_schemata {
        let c_schema = CString::new(schema.to_owned())?;
        rime_api_call!(
            config_set_string,
            &mut default_custom,
            c_new_schema_list_item.as_ptr(),
            c_schema.as_ptr()
        );
    }
    rime_api_call!(config_close, &mut default_custom);

    rime_api_call!(finalize);
    Ok(())
}

pub fn select_schema(schema: &str) -> anyhow::Result<()> {
    log::debug!("選擇輸入方案: {schema}");
    rime_api_call!(deployer_initialize, std::ptr::null_mut());

    let mut user_config: RimeConfig = rime_struct_new!();
    let c_user_config = CString::new("user")?;
    rime_api_call!(user_config_open, c_user_config.as_ptr(), &mut user_config);
    let c_selected_schema = CString::new("var/previously_selected_schema")?;
    let c_schema = CString::new(schema.to_owned())?;
    rime_api_call!(
        config_set_string,
        &mut user_config,
        c_selected_schema.as_ptr(),
        c_schema.as_ptr()
    );
    rime_api_call!(config_close, &mut user_config);

    rime_api_call!(finalize);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use claims::assert_ok;
    use lazy_static::lazy_static;
    use std::fs::{read_to_string, write};
    use std::sync::{Once, RwLock};

    lazy_static! {
        static ref PUBLIC_TEST_SPACE: PathBuf = std::env::temp_dir().join("rime_levers_tests");
    }
    // 公共測試場地只需在各項測試開始之前清理一次.
    static PREPARED_PUBLIC_TEST_SPACE: Once = Once::new();
    // rime::Deployer 是個單例, 同一時刻只能服務一片場地.
    // 公共場地中的測試可以並發執行, 持讀鎖. 專用場地的測試持寫鎖.
    static ENGINE_LOCK: RwLock<()> = RwLock::new(());

    fn prepare() {
        PREPARED_PUBLIC_TEST_SPACE.call_once(|| {
            if PUBLIC_TEST_SPACE.exists() {
                assert_ok!(std::fs::remove_dir_all(&*PUBLIC_TEST_SPACE));
            }
        });
        assert_ok!(setup_engine_traits(&PUBLIC_TEST_SPACE));
    }

    #[test]
    fn test_apply_patch_for_default() {
        let _lock = ENGINE_LOCK.read().unwrap();
        prepare();
        assert_ok!(apply_patch("default", "menu/page_size", "5"));

        let result_file = PUBLIC_TEST_SPACE.join("default.custom.yaml");
        let patched_file_content = assert_ok!(read_to_string(&result_file));
        assert!(patched_file_content.contains(
            r#"
patch:
  "menu/page_size": 5"#
        ));
    }

    #[test]
    fn test_apply_patch_for_schema() {
        let _lock = ENGINE_LOCK.read().unwrap();
        prepare();
        assert_ok!(apply_patch("ohmyrime.schema", "menu/page_size", "9"));

        let result_file = PUBLIC_TEST_SPACE.join("ohmyrime.custom.yaml");
        let patched_file_content = assert_ok!(read_to_string(&result_file));
        assert!(patched_file_content.contains(
            r#"
patch:
  "menu/page_size": 9"#
        ));
    }

    #[test]
    fn test_apply_patch_for_list_value() {
        let _lock = ENGINE_LOCK.read().unwrap();
        prepare();
        assert_ok!(apply_patch(
            "patch_list",
            "starcraft/races",
            r#"[protoss, terran, zerg]"#
        ));

        let result_file = PUBLIC_TEST_SPACE.join("patch_list.custom.yaml");
        let patched_file_content = assert_ok!(read_to_string(&result_file));
        println!("補丁文件內容: {}", patched_file_content);
        assert!(patched_file_content.contains(
            r#"
patch:
  "starcraft/races":
    - protoss
    - terran
    - zerg"#
        ));
    }

    #[test]
    fn test_apply_patch_for_map_value() {
        let _lock = ENGINE_LOCK.read().unwrap();
        prepare();
        assert_ok!(apply_patch(
            "patch_map",
            "starcraft/workers",
            r#"{protoss: probe, terran: scv, zerg: drone}"#
        ));

        let result_file = PUBLIC_TEST_SPACE.join("patch_map.custom.yaml");
        let patched_file_content = assert_ok!(read_to_string(&result_file));
        assert!(patched_file_content.contains(
            r#"
patch:
  "starcraft/workers":
    protoss: probe
    terran: scv
    zerg: drone"#
        ));
    }

    #[test]
    fn test_build_binary() {
        let _lock = ENGINE_LOCK.write().unwrap();
        let specified_test_place = std::env::temp_dir().join("rime_levers_tests_build");
        if specified_test_place.exists() {
            assert_ok!(std::fs::remove_dir_all(&specified_test_place));
        }
        assert_ok!(setup_engine_traits(&specified_test_place));
        assert_ok!(write(
            specified_test_place.join("default.yaml"),
            r#"
schema_list:
  - schema: ohmyrime
"#,
        ));
        assert_ok!(write(
            specified_test_place.join("ohmyrime.schema.yaml"),
            r#"
schema:
  schema_id: ohmyrime
"#,
        ));

        assert_ok!(build_binaries());

        assert!(specified_test_place.join("installation.yaml").exists());
        assert!(specified_test_place.join("user.yaml").exists());
        let staging_dir = specified_test_place.join("build");
        let default_config_file = staging_dir.join("default.yaml");
        let default_config_content = assert_ok!(read_to_string(&default_config_file));
        assert!(default_config_content.contains(
            r#"
schema_list:
  - schema: ohmyrime"#
        ));
        let schema_file = staging_dir.join("ohmyrime.schema.yaml");
        let schema_file_content = assert_ok!(read_to_string(&schema_file));
        assert!(schema_file_content.contains(
            r#"
schema:
  schema_id: ohmyrime"#
        ));
    }

    #[test]
    fn test_add_to_schema_list() {
        let _lock = ENGINE_LOCK.write().unwrap();
        let specified_test_place = std::env::temp_dir().join("rime_levers_tests_add");
        if specified_test_place.exists() {
            assert_ok!(std::fs::remove_dir_all(&specified_test_place));
        }
        assert_ok!(setup_engine_traits(&specified_test_place));

        let new_schemata = vec!["protoss".to_owned(), "terran".to_owned()];
        assert_ok!(add_to_schema_list(&new_schemata));

        let default_custom = specified_test_place.join("default.custom.yaml");
        assert!(default_custom.exists());
        let default_custom_content = assert_ok!(read_to_string(&default_custom));
        assert!(default_custom_content.contains(
            r#"patch:
  schema_list:
    - {schema: protoss}
    - {schema: terran}"#
        ));

        let new_schemata = vec!["terran".to_owned(), "zerg".to_owned()];
        assert_ok!(add_to_schema_list(&new_schemata));
        let default_custom_content = assert_ok!(read_to_string(&default_custom));
        assert!(default_custom_content.contains(
            r#"patch:
  schema_list:
    - {schema: protoss}
    - {schema: terran}
    - {schema: zerg}"#
        ));
    }

    #[test]
    fn test_select_schema() {
        let _lock = ENGINE_LOCK.write().unwrap();
        let specified_test_space = std::env::temp_dir().join("rime_levers_tests_select");
        if specified_test_space.exists() {
            assert_ok!(std::fs::remove_dir_all(&specified_test_space));
        }
        assert_ok!(setup_engine_traits(&specified_test_space));

        let grrrr_selection = "protoss";
        assert_ok!(select_schema(grrrr_selection));

        let user_config = specified_test_space.join("user.yaml");
        assert!(user_config.exists());
        let user_config_content = assert_ok!(read_to_string(&user_config));
        assert!(user_config_content.contains(
            r#"var:
  previously_selected_schema: protoss"#
        ));

        let boxer_selection = "terran";
        assert_ok!(select_schema(boxer_selection));

        let user_config_content = assert_ok!(read_to_string(&user_config));
        assert!(user_config_content.contains(
            r#"var:
  previously_selected_schema: terran"#
        ));
    }
}
