use std::path::PathBuf;

use async_process::Command;

use anyhow::anyhow;
use serde_json::json;
use uuid::Uuid;

const PROJECTS_BASE: &str = "/home/iobt/rust/meltano-projects/";
const PORTALS_PROJECT: &str = "/home/iobt/rust/meltano-projects/portals";
const MELTANO_BIN: &str = "/home/iobt/rust/meltano-projects/.venv/bin";

pub enum ExtractorType {
  GoogleSheetsExtractor,
  SpreadsheetsExtractor,
}

pub enum ExtractorConfigData {
  GoogleSheets(GoogleSheetsConfigData),
  Spreadsheets(SpreadsheetsConfigData),
}

pub enum LoaderType {
  PostgresLoader,
}

pub struct GoogleSheetsConfigData {
  pub refresh_token: String,
  pub spreadsheet_id: String,
  pub sheet_name: String,
}

pub enum SpreadsheetsType {
  CSV,
  Excel,
}

pub struct SpreadsheetsTableData {
  // local server? need to store "downloaded" excel file somewhere
  pub file_path: PathBuf,
  pub key_properties: Option<String>,

  // For Excel only
  pub worksheet_name: Option<String>,
}

pub struct SpreadsheetsConfigData {
  pub file_type: SpreadsheetsType,
  pub table_data: SpreadsheetsTableData,
}

// Holds services for authorization?
pub struct MeltanoService {}

impl MeltanoService {
  pub fn new() -> Self {
    MeltanoService {}
  }

  pub async fn extract_load(
    &self,
    // extractor_type: ExtractorType,
    extractor_config_data: ExtractorConfigData,
  ) -> Result<bool, anyhow::Error> {
    match extractor_config_data {
      ExtractorConfigData::GoogleSheets(config_data) => self.google_sheets_to_db(config_data).await,
      ExtractorConfigData::Spreadsheets(config_data) => self.spreadsheets_to_db(config_data).await,
    }
  }

  pub async fn spreadsheets_to_db(
    &self,
    config_data: SpreadsheetsConfigData,
  ) -> Result<bool, anyhow::Error> {
    let table_data = config_data.table_data;
    let file_path = table_data.file_path;
    let root_dir = file_path
      .parent()
      .and_then(|p| p.to_str())
      .ok_or_else(|| anyhow!("Current path is invalid!"))?;
    let file_name = file_path
      .file_name()
      .and_then(|n| n.to_str())
      .ok_or_else(|| anyhow!("CSV file is invalid!"))?;

    let key_properties = if let Some(key) = table_data.key_properties {
      vec![key]
    } else {
      vec![]
    };

    let table_string = match config_data.file_type {
          SpreadsheetsType::CSV => json!([{
            "path": format!("file://{}", root_dir).as_str(),
            "name": format!("csv_{}", file_name.split(".").next().unwrap_or_else(|| "").to_lowercase().replace(" ", "")).as_str(),
            // Need to have regex '^' to filter out possible lock file
            "pattern": format!("^{}", file_name).as_str(),
            "start_date": "2020-01-01T00:00:00Z",
            "key_properties": key_properties,
            "format": "csv",
          }]).to_string(),
          SpreadsheetsType::Excel => {
            let worksheet_name = table_data.worksheet_name.ok_or(anyhow!("No worksheet name provided!"))?;
            json!([{
            "path": format!("file://{}", root_dir).as_str(),
            "name": format!("{}_{}", file_name.split(".").next().unwrap_or_else(|| ""), worksheet_name.as_str()).to_lowercase().as_str(),
            // Need to have regex '^' to filter out possible lock file
            "pattern": format!("^{}", file_name).as_str(),
            "start_date": "2020-01-01T00:00:00Z",
            "key_properties": key_properties,
            "format": "excel",
            "worksheet_name": worksheet_name.as_str()
          }]).to_string()
        }
        };

    std::env::set_current_dir(PORTALS_PROJECT)?;
    let output = Command::new("meltano")
      .envs(vec![
        ("PATH", MELTANO_BIN),
        ("MELTANO_CLI_LOG_LEVEL", "debug"),
        ("TAP_SPREADSHEETS_ANYWHERE_TABLES", table_string.as_str()),
      ])
      .args(&[
        "elt",
        "tap-spreadsheets-anywhere",
        "target-postgres",
        &format!(
          "--job_id={}-spreadsheets-anywhere-to-postgres",
          Uuid::new_v4()
        ),
      ])
      .output()
      .await?;

    println!("{}", &std::str::from_utf8(&output.stdout)?);
    println!("{}", &std::str::from_utf8(&output.stderr)?);

    Ok(true)
  }

  pub async fn google_sheets_to_db(
    &self,
    config_data: GoogleSheetsConfigData,
  ) -> Result<bool, anyhow::Error> {
    // TODO: set_current_dir implications?
    std::env::set_current_dir(PORTALS_PROJECT)?;
    // Injects refresh_token and spreadsheet ID dynamically
    let refresh_token = config_data.refresh_token;
    let spreadsheet_id = config_data.spreadsheet_id;
    let sheet_name = config_data.sheet_name;

    let output2 = Command::new("meltano")
      .envs(vec![
        ("PATH", MELTANO_BIN),
        ("TAP_GOOGLE_SHEETS_REFRESH_TOKEN", refresh_token.as_str()),
        ("TAP_GOOGLE_SHEETS_SPREADSHEET_ID", spreadsheet_id.as_str()),
        ("TAP_GOOGLE_SHEETS_SELECT", sheet_name.as_str()),
      ])
      .args(&[
        "elt",
        "tap-google-sheets",
        "target-postgres",
        // TODO: need to modify job ID else may overwrite to existing table for same user
        &format!(
          "--job_id={}-sheets-{}-to-postgres",
          Uuid::new_v4(),
          spreadsheet_id.as_str()
        ),
      ])
      .output()
      .await?;

    dbg!(&std::str::from_utf8(&output2.stdout));
    dbg!(&std::str::from_utf8(&output2.stderr));

    Ok(true)
  }
}
