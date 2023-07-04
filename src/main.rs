use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use thiserror::Error;
use openapiv3::OpenAPI;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("source read error")]
    SourceReadError(#[from] io::Error),
    #[error("YAML schema parse error")]
    SchemaParseError(#[from] serde_yaml::Error),
    #[error("Missing operationId for path {path}, verb {verb}")]
    MissingOperationID {
        path: String,
        verb: String,
    },
}

fn load_api<P: AsRef<Path>>(p: P) -> Result<OpenAPI, GeneratorError> {
    let mut f = File::open(p)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    let api = serde_yaml::from_str::<OpenAPI>(&content)?;
    Ok(api)
}

fn generate_service_trait(api: &OpenAPI) -> Result<(), GeneratorError> {
    let mut scope = codegen::Scope::new();
    let mut new_trait = codegen::Trait::new("ServiceHandler");

    // TODO: Requires multiple passes i guess due to how scope lends out objects.
    for (path, path_item) in api.paths.iter() {
        if let Some(path_item) = path_item.as_item() {
            for (verb, operation) in path_item.iter() {
                let Some(operation_id) = &operation.operation_id else {
                    return Err(GeneratorError::MissingOperationID {
                        path: path.to_owned(),
                        verb: verb.to_owned(),
                    });
                };

                // TODO: Operation id isn't good enough, also need content type.
                let request_type = codegen::Struct::new(&format!("{}Request", operation_id));
                let response_type = codegen::Enum::new(&format!("{}Response", operation_id));
                let mut return_type = codegen::Type::new("Result");
                return_type.generic(request_type.ty())
                    .generic(response_type.ty());

               scope.push_struct(request_type); 
               scope.push_enum(response_type); 

                let mut new_fn = new_trait.new_fn(operation_id)
                    .arg_mut_self()
                    .ret(return_type);
            }
        }
    }

    scope.push_trait(new_trait);
    println!("Service trait code:\n{}", scope.to_string());

    Ok(())
}

fn main() -> Result<(), GeneratorError> {
    let api = load_api("./spec.yaml")?;

    generate_service_trait(&api)
}
