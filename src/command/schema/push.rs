use crate::client::get_rover_client;
use anyhow::Result;
use rover_client::query::schema::push;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Push {
    /// Path to a .graphql SDL file
    #[structopt(name = "SCHEMA_PATH", parse(from_os_str))]
    schema_path: PathBuf,

    /// The variant of the request graph from Apollo Studio
    #[structopt(long, default_value = "current")]
    variant: String,

    /// The unique graph name that this schema is being pushed to
    #[structopt(long)]
    graph_name: String,

    /// Name of the configuration profile (default: "default")
    #[structopt(long = "profile", default_value = "default")]
    profile_name: String,
}

impl Push {
    pub fn run(&self) -> Result<()> {
        let client = get_rover_client(&self.profile_name)?;
        log::info!(
            "Let's push this schema, {}@{}, mx. {}!",
            &self.graph_name,
            &self.variant,
            &self.profile_name
        );

        let schema_document = get_schema_from_file_path(&self.schema_path)?;

        let push_response = push::run(
            push::push_schema_mutation::Variables {
                graph_id: self.graph_name.clone(),
                variant: self.variant.clone(),
                schema_document: Some(schema_document),
            },
            client,
        )?;

        handle_response(push_response);
        Ok(())
    }
}

fn get_schema_from_file_path(path: &PathBuf) -> Result<String> {
    if Path::exists(path) {
        let contents = std::fs::read_to_string(path)?;
        Ok(contents)
    } else {
        Err(anyhow::anyhow!(
            "Invalid path. No file found at {}",
            path.display()
        ))
    }
}

/// handle all output logging from operation
fn handle_response(response: push::PushResponse){
    log::info!(
        "{}\nSchema Hash: {}",
        response.message, // the message will say if successful, and details
        response.schema_hash
    );
}

#[cfg(test)]
mod tests {
    use super::{get_schema_from_file_path, handle_response, push};
    use assert_fs::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn get_schema_from_file_path_loads() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("schema.graphql");
        let mut temp_file = File::create(file_path.clone()).unwrap();
        write!(temp_file, "type Query {{ hello: String! }}").unwrap();

        let schema = get_schema_from_file_path(&file_path).unwrap();
        assert_eq!(schema, "type Query { hello: String! }".to_string());
    }

    #[test]
    fn get_schema_from_file_path_errs_on_bad_path() {
        let empty_path = std::path::PathBuf::new().join("wow.graphql");
        let schema = get_schema_from_file_path(&empty_path);
        assert_eq!(schema.is_err(), true);
    }

    #[test]
    fn handle_response_doesnt_err() {
        handle_response(push::PushResponse {
            message: "oooh wowo it pushed successfully!".to_string(),
            schema_hash: "123456".to_string()
        })
    }
}
