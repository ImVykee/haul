use directories::ProjectDirs;
use std::fs::read_to_string;

pub struct Config {
    pub ui: Ui,
    pub date: Date,
}

pub struct Ui {
    pub colors: bool,
    pub compact: bool,
}

pub struct Date {
    pub format: String,
    pub warn_days_before: i32,
}

pub fn parse_config() -> Result<Config, Box<dyn std::error::Error>> {
    let raw_config = read_to_string(config_path()?)?;
    let mut current_block = String::from("_NONE_");
    let mut line_counter = 0;
    let mut config = Config {
        ui: Ui {
            colors: true,
            compact: false,
        },
        date: Date {
            format: String::from("%d/%m/%y"),
            warn_days_before: 1,
        },
    };
    for line in raw_config.lines() {
        line_counter += 1;
        let line = line.to_lowercase();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if &current_block == "_NONE_" && line.ends_with("{") {
            current_block = line.replace("{", "").trim().to_string();
            continue;
        }
        if line == "}" {
            current_block = String::from("_NONE_");
            continue;
        }
        let mut parts = line.split("=");
        let param = parts
            .next()
            .ok_or(format!("invalid config line -| line {}", line_counter))?
            .trim();
        let value = parts
            .next()
            .ok_or(format!("invalid config line -| line {}", line_counter))?
            .trim();
        match current_block.as_str() {
            "ui" => match param {
                "colors" => config.ui.colors = parse_bool(value)?,
                "compact" => config.ui.compact = parse_bool(value)?,
                _ => (),
            },
            "date" => match param {
                "format" => config.date.format = parse_date(value)?,
                "warn_days_before" => config.date.warn_days_before = value.parse()?,
                _ => (),
            },
            _ => (),
        };
    }
    Ok(config)
}

fn parse_date(raw_date: &str) -> Result<String, Box<dyn std::error::Error>> {
    let date = raw_date.split("/");
    let mut result = String::new();
    for pos in date {
        if !["d", "m", "y"].contains(&pos) {
            return Err(format!("Invalid date format \"{}\"", raw_date).into());
        }
        if pos == "y" {
            result += "%Y/";
        } else {
            result += &format!("%{}/", pos);
        }
    }
    result.pop();
    Ok(result)
}

fn parse_bool(raw_bool: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match raw_bool {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("can't parse {} into boolean", raw_bool).into()),
    }
}

pub fn config_path() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let dirs = ProjectDirs::from("", "", "haul").unwrap();
    Ok(dirs.config_dir().join("config"))
}
