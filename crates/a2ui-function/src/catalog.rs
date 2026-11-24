#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    reason = "simple runtime accessors and standard FunctionError results"
)]

use std::collections::HashMap;

use a2ui_author::{FunctionCall, FunctionError};
use jsonschema::{Registry, Resource, Validator};
use serde_json::{Value, json};

use crate::error::{self, INVALID_CATALOG};

const COMMON_TYPES: &str = include_str!("../common_types.json");
const COMMON_TYPES_URI: &str = "https://a2ui.org/specification/v1_0/common_types.json";
const CATALOG_PLACEHOLDER_URI: &str = "https://a2ui.org/specification/v1_0/catalog.json";
const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

/// The execution boundary declared by a catalog function.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AllowedCallers {
    #[default]
    RendererOnly,
    AgentOnly,
    RendererOrAgent,
}

/// A catalog function's declared JSON result shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Any,
    ValidationResult,
    Void,
}

impl ReturnType {
    fn parse(value: Option<&Value>) -> Result<Self, FunctionError> {
        match value.and_then(Value::as_str) {
            Some("string") => Ok(Self::String),
            Some("number") => Ok(Self::Number),
            Some("boolean") => Ok(Self::Boolean),
            Some("array") => Ok(Self::Array),
            Some("object") => Ok(Self::Object),
            Some("any") => Ok(Self::Any),
            Some("validationResult") => Ok(Self::ValidationResult),
            Some("void") => Ok(Self::Void),
            _ => Err(error::error(
                INVALID_CATALOG,
                "Each function requires a valid returnType",
            )),
        }
    }

    /// Returns whether a JSON value has this declared result shape.
    pub fn accepts(self, value: &Value) -> bool {
        match self {
            Self::String => value.is_string(),
            Self::Number => value.is_number(),
            Self::Boolean => value.is_boolean(),
            Self::Array => value.is_array(),
            Self::Object => value.is_object(),
            Self::Any => true,
            Self::ValidationResult => value
                .as_object()
                .and_then(|object| object.get("valid"))
                .is_some_and(Value::is_boolean),
            Self::Void => value.is_null(),
        }
    }
}

/// Runtime metadata for one declared function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionDefinition {
    pub name: String,
    pub allowed_callers: AllowedCallers,
    pub requires_user_activation: bool,
    pub return_type: ReturnType,
}

impl FunctionDefinition {
    /// Validates a handler or renderer result without including its value in errors.
    pub fn validate_return(&self, value: &Value) -> Result<(), FunctionError> {
        if self.return_type.accepts(value) {
            Ok(())
        } else {
            Err(error::execution_error(format!(
                "Function '{}' returned a value that does not match its declared returnType",
                self.name
            )))
        }
    }
}

/// One immutable v1.0 catalog and its offline `FunctionCall` validator.
#[derive(Clone, Debug)]
pub struct Catalog {
    id: String,
    functions: HashMap<String, FunctionDefinition>,
    validator: Validator,
}

impl Catalog {
    /// Parses a catalog and compiles its `FunctionCall` schema without network or file access.
    pub fn from_json(document: &str) -> Result<Self, FunctionError> {
        let value = serde_json::from_str(document)
            .map_err(|_error| error::error(INVALID_CATALOG, "Catalog is not valid JSON"))?;
        Self::from_value(value)
    }

    /// Builds a catalog from an owned JSON document.
    pub fn from_value(document: Value) -> Result<Self, FunctionError> {
        let object = document
            .as_object()
            .ok_or_else(|| error::error(INVALID_CATALOG, "Catalog must be a JSON object"))?;
        if object.get("protocolVersion").and_then(Value::as_str) != Some("1.0") {
            return Err(error::error(
                INVALID_CATALOG,
                "Catalog protocolVersion must be '1.0'",
            ));
        }
        let id = object
            .get("catalogId")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| error::error(INVALID_CATALOG, "Catalog requires a non-empty catalogId"))?
            .to_owned();
        let functions = parse_functions(object.get("functions"))?;
        let validator = compile_validator(document)?;
        Ok(Self {
            id,
            functions,
            validator,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn function(&self, name: &str) -> Option<&FunctionDefinition> {
        self.functions.get(name)
    }

    fn validate(&self, call: &FunctionCall) -> Result<(), FunctionError> {
        let instance = serde_json::to_value(call)
            .map_err(|_error| error::invalid_call("Function call could not be serialized"))?;
        self.validator.validate(&instance).map_err(|_error| {
            error::invalid_call(format!(
                "Function '{}' has invalid arguments or call shape",
                call.name
            ))
        })
    }
}

/// An immutable set of catalogs keyed by their unique catalog IDs.
#[derive(Clone, Debug)]
pub struct CatalogSet {
    catalogs: HashMap<String, Catalog>,
}

impl CatalogSet {
    pub fn new(catalogs: impl IntoIterator<Item = Catalog>) -> Result<Self, FunctionError> {
        let mut by_id = HashMap::new();
        for catalog in catalogs {
            if by_id.insert(catalog.id.clone(), catalog).is_some() {
                return Err(error::error(INVALID_CATALOG, "Catalog IDs must be unique"));
            }
        }
        if by_id.is_empty() {
            return Err(error::error(
                INVALID_CATALOG,
                "CatalogSet requires at least one catalog",
            ));
        }
        Ok(Self { catalogs: by_id })
    }

    /// Parses a set containing one catalog. Add more catalogs through `CatalogSet::new`.
    pub fn from_json(document: &str) -> Result<Self, FunctionError> {
        Self::new([Catalog::from_json(document)?])
    }

    pub fn get(&self, catalog_id: &str) -> Option<&Catalog> {
        self.catalogs.get(catalog_id)
    }

    pub fn len(&self) -> usize {
        self.catalogs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.catalogs.is_empty()
    }

    /// Resolves an explicit `catalogId` and validates the complete `FunctionCall` JSON.
    pub fn validate_call(&self, call: &FunctionCall) -> Result<FunctionDefinition, FunctionError> {
        let catalog_id = call
            .catalog_id
            .as_deref()
            .ok_or_else(|| error::invalid_call("Function call requires an explicit catalogId"))?;
        let catalog = self
            .get(catalog_id)
            .ok_or_else(|| error::unknown_function(&call.name))?;
        let definition = catalog
            .function(&call.name)
            .cloned()
            .ok_or_else(|| error::unknown_function(&call.name))?;
        catalog.validate(call)?;
        Ok(definition)
    }

    /// Validates a renderer-originated callAgentFunction invocation.
    pub fn validate_agent_call(
        &self,
        call: &FunctionCall,
    ) -> Result<FunctionDefinition, FunctionError> {
        let definition = self.validate_call(call)?;
        if definition.allowed_callers == AllowedCallers::AgentOnly {
            return Err(error::invalid_call(format!(
                "Function '{}' cannot be called by the renderer",
                call.name
            )));
        }
        Ok(definition)
    }

    /// Validates an agent-originated callRendererFunction invocation.
    pub fn validate_renderer_call(
        &self,
        call: &FunctionCall,
    ) -> Result<FunctionDefinition, FunctionError> {
        let definition = self.validate_call(call)?;
        if definition.allowed_callers == AllowedCallers::RendererOnly {
            return Err(error::invalid_call(format!(
                "Function '{}' cannot be called by the agent",
                call.name
            )));
        }
        if definition.requires_user_activation {
            return Err(error::invalid_call(format!(
                "Function '{}' requires user activation",
                call.name
            )));
        }
        Ok(definition)
    }
}

fn parse_functions(
    value: Option<&Value>,
) -> Result<HashMap<String, FunctionDefinition>, FunctionError> {
    let Some(value) = value else {
        return Ok(HashMap::new());
    };
    let functions = value
        .as_object()
        .ok_or_else(|| error::error(INVALID_CATALOG, "Catalog functions must be an object"))?;
    functions
        .iter()
        .map(|(name, value)| {
            let allowed_callers = match value.get("allowedCallers") {
                None => AllowedCallers::RendererOnly,
                Some(Value::String(value)) if value == "rendererOnly" => {
                    AllowedCallers::RendererOnly
                }
                Some(Value::String(value)) if value == "agentOnly" => AllowedCallers::AgentOnly,
                Some(Value::String(value)) if value == "rendererOrAgent" => {
                    AllowedCallers::RendererOrAgent
                }
                _ => {
                    return Err(error::error(
                        INVALID_CATALOG,
                        format!("Function '{name}' has an invalid allowedCallers value"),
                    ));
                }
            };
            let requires_user_activation = match value.get("requiresUserActivation") {
                None => false,
                Some(Value::Bool(value)) => *value,
                Some(_) => {
                    return Err(error::error(
                        INVALID_CATALOG,
                        format!("Function '{name}' has invalid requiresUserActivation metadata"),
                    ));
                }
            };
            if requires_user_activation && allowed_callers != AllowedCallers::RendererOnly {
                return Err(error::error(
                    INVALID_CATALOG,
                    format!(
                        "Function '{name}' requires rendererOnly when user activation is required"
                    ),
                ));
            }
            Ok((
                name.clone(),
                FunctionDefinition {
                    name: name.clone(),
                    allowed_callers,
                    requires_user_activation,
                    return_type: ReturnType::parse(value.get("returnType"))?,
                },
            ))
        })
        .collect()
}

fn compile_validator(mut document: Value) -> Result<Validator, FunctionError> {
    let common_types: Value = serde_json::from_str(COMMON_TYPES)
        .map_err(|_error| error::error(INVALID_CATALOG, "Embedded common types are invalid"))?;
    document["$id"] = Value::String(CATALOG_PLACEHOLDER_URI.to_owned());
    let registry = Registry::new()
        .add(COMMON_TYPES_URI, Resource::from_contents(common_types))
        .map_err(|_error| error::error(INVALID_CATALOG, "Could not register common types"))?
        .add(CATALOG_PLACEHOLDER_URI, Resource::from_contents(document))
        .map_err(|_error| error::error(INVALID_CATALOG, "Could not register the catalog"))?
        .prepare()
        .map_err(|_error| error::error(INVALID_CATALOG, "Could not resolve catalog references"))?;
    let schema = json!({
        "$schema": DRAFT_2020_12,
        "$ref": format!("{COMMON_TYPES_URI}#/$defs/FunctionCall")
    });
    jsonschema::options()
        .with_registry(&registry)
        .offline()
        .build(&schema)
        .map_err(|error| {
            error::error(
                INVALID_CATALOG,
                format!("Invalid catalog function schema: {}", error.masked()),
            )
        })
}
