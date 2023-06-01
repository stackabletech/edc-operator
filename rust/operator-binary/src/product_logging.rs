use crate::controller::MAX_LOG_FILES_SIZE_IN_MIB;

use crate::crd::{Container, EDCCluster, EDC_CONNECTOR_JAVA_LOG_FILE, STACKABLE_LOG_DIR};
use snafu::{OptionExt, ResultExt, Snafu};
use stackable_operator::product_logging::spec::{AutomaticContainerLogConfig, LogLevel};
use stackable_operator::{
    builder::ConfigMapBuilder,
    client::Client,
    k8s_openapi::api::core::v1::ConfigMap,
    kube::ResourceExt,
    product_logging::{
        self,
        spec::{ContainerLogConfig, ContainerLogConfigChoice, Logging},
    },
    role_utils::RoleGroupRef,
};

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("object has no namespace"))]
    ObjectHasNoNamespace,
    #[snafu(display("failed to retrieve the ConfigMap [{cm_name}]"))]
    ConfigMapNotFound {
        source: stackable_operator::error::Error,
        cm_name: String,
    },
    #[snafu(display("failed to retrieve the entry [{entry}] for ConfigMap [{cm_name}]"))]
    MissingConfigMapEntry {
        entry: &'static str,
        cm_name: String,
    },
    #[snafu(display("crd validation failure"))]
    CrdValidationFailure { source: crate::crd::Error },
    #[snafu(display("vectorAggregatorConfigMapName must be set"))]
    MissingVectorAggregatorAddress,
}

type Result<T, E = Error> = std::result::Result<T, E>;

const VECTOR_AGGREGATOR_CM_ENTRY: &str = "ADDRESS";
const CONSOLE_CONVERSION_PATTERN: &str = "%d{ISO8601} %5p [%t] %c{2}: %m%n";

/// Return the address of the Vector aggregator if the corresponding ConfigMap name is given in the
/// cluster spec
pub async fn resolve_vector_aggregator_address(
    edc: &EDCCluster,
    client: &Client,
) -> Result<Option<String>> {
    let vector_aggregator_address = if let Some(vector_aggregator_config_map_name) =
        &edc.spec.cluster_config.vector_aggregator_config_map_name
    {
        let vector_aggregator_address = client
            .get::<ConfigMap>(
                vector_aggregator_config_map_name,
                edc.namespace()
                    .as_deref()
                    .context(ObjectHasNoNamespaceSnafu)?,
            )
            .await
            .context(ConfigMapNotFoundSnafu {
                cm_name: vector_aggregator_config_map_name.to_string(),
            })?
            .data
            .and_then(|mut data| data.remove(VECTOR_AGGREGATOR_CM_ENTRY))
            .context(MissingConfigMapEntrySnafu {
                entry: VECTOR_AGGREGATOR_CM_ENTRY,
                cm_name: vector_aggregator_config_map_name.to_string(),
            })?;
        Some(vector_aggregator_address)
    } else {
        None
    };

    Ok(vector_aggregator_address)
}

/// Extend the role group ConfigMap with logging and Vector configurations
pub fn extend_role_group_config_map(
    rolegroup: &RoleGroupRef<EDCCluster>,
    vector_aggregator_address: Option<&str>,
    logging: &Logging<Container>,
    cm_builder: &mut ConfigMapBuilder,
) -> Result<()> {
    if let Some(ContainerLogConfig {
        choice: Some(ContainerLogConfigChoice::Automatic(log_config)),
    }) = logging.containers.get(&Container::Edc)
    {
        cm_builder.add_data(
            EDC_CONNECTOR_JAVA_LOG_FILE,
            create_java_logging_config(
                &format!("{STACKABLE_LOG_DIR}/edc"),
                EDC_CONNECTOR_JAVA_LOG_FILE,
                MAX_LOG_FILES_SIZE_IN_MIB,
                CONSOLE_CONVERSION_PATTERN,
                log_config,
            ),
        );
    }

    let vector_log_config = if let Some(ContainerLogConfig {
        choice: Some(ContainerLogConfigChoice::Automatic(log_config)),
    }) = logging.containers.get(&Container::Vector)
    {
        Some(log_config)
    } else {
        None
    };

    if logging.enable_vector_agent {
        cm_builder.add_data(
            product_logging::framework::VECTOR_CONFIG_FILE,
            create_vector_config(
                rolegroup,
                vector_aggregator_address.context(MissingVectorAggregatorAddressSnafu)?,
                vector_log_config,
            ),
        );
    }

    Ok(())
}

fn create_java_logging_config(
    _log_dir: &str,
    _log_file: &str,
    _max_size_in_mib: u32,
    _console_conversion_pattern: &str,
    _config: &AutomaticContainerLogConfig,
) -> String {
    format!(
        r#"handlers=java.util.logging.FileHandler, java.util.logging.ConsoleHandler
.level={root_log_level}
java.util.logging.FileHandler.pattern=/stackable/log/edc/edc.%g.logger
java.util.logging.FileHandler.limit=50000
java.util.logging.FileHandler.count=10
java.util.logging.FileHandler.formatter=java.util.logging.SimpleFormatter
java.util.logging.ConsoleHandler.level={console_log_level}
java.util.logging.ConsoleHandler.formatter = java.util.logging.SimpleFormatter

"#,
        root_log_level = "INFO",
        console_log_level = "INFO"
    )
}

fn create_vector_config(
    role_group: &RoleGroupRef<EDCCluster>,
    vector_aggregator_address: &str,
    config: Option<&AutomaticContainerLogConfig>,
) -> String {
    let vector_log_level = config
        .and_then(|config| config.file.as_ref())
        .and_then(|file| file.level)
        .unwrap_or_default();

    let vector_log_level_filter_expression = match vector_log_level {
        LogLevel::TRACE => "true",
        LogLevel::DEBUG => r#".level != "TRACE""#,
        LogLevel::INFO => r#"!includes(["TRACE", "DEBUG"], .metadata.level)"#,
        LogLevel::WARN => r#"!includes(["TRACE", "DEBUG", "INFO"], .metadata.level)"#,
        LogLevel::ERROR => r#"!includes(["TRACE", "DEBUG", "INFO", "WARN"], .metadata.level)"#,
        LogLevel::FATAL => "false",
        LogLevel::NONE => "false",
    };

    format!(
        r#"data_dir = "/stackable/vector/var"

[log_schema]
host_key = "pod"

[sources.vector]
type = "internal_logs"

[sources.files_stdout]
type = "file"
include = ["{STACKABLE_LOG_DIR}/*/*.stdout.log"]

[sources.files_stderr]
type = "file"
include = ["{STACKABLE_LOG_DIR}/*/*.stderr.log"]

[sources.files_jul]
type = "file"
include = ["{STACKABLE_LOG_DIR}/*/*.logger"]

[transforms.processed_files_stdout]
inputs = ["files_stdout"]
type = "remap"
source = '''
.logger = "ROOT"
.level = "INFO"
'''

[transforms.processed_files_stderr]
inputs = ["files_stderr"]
type = "remap"
source = '''
.logger = "ROOT"
.level = "ERROR"
'''

[transforms.processed_files_jul]
inputs = ["files_jul"]
type = "remap"
source = '''
.logger = "ROOT"
.level = "INFO"
'''

[transforms.parsed_logs_std]
inputs = ["processed_files_std*"]
type = "remap"
source = '''
. |= parse_regex!(.file, r'^{STACKABLE_LOG_DIR}/(?P<container>.*?)/(?P<file>.*?)$')
del(.source_type)
'''

[transforms.parsed_logs_jul]
inputs = ["processed_files_jul"]
type = "remap"
source = '''
. |= parse_regex!(.file, r'^{STACKABLE_LOG_DIR}/(?P<container>.*?)/(?P<file>.*?)$')
del(.source_type)
'''

[transforms.extended_logs_files]
inputs = ["parsed_logs_std"]
type = "remap"
source = '''
parsed_event, err = parse_regex(strip_whitespace(strip_ansi_escape_codes(string!(.message))), r'(?P<level>\w+)+[ ]+(?P<timestamp>[0-9]{{4}}-(0[1-9]|1[0-2])-(0[1-9]|[1-2][0-9]|3[0-1])T(2[0-3]|[01][0-9]):[0-5][0-9]:[0-5][0-9].[0-9]+)+[ ]+(?P<message>.*)')

if err == null {{
  .timestamp = parse_timestamp!(parsed_event.timestamp, "%Y-%m-%dT%H:%M:%S.%f")
  .level = parsed_event.level
  .message = parsed_event.message
}}
'''

[transforms.extended_logs_jul]
inputs = ["parsed_logs_jul"]
type = "remap"
source = '''
.message = "Java Logging: " + string!(.message)
'''

[transforms.filtered_logs_vector]
inputs = ["vector"]
type = "filter"
condition = '{vector_log_level_filter_expression}'

[transforms.extended_logs_vector]
inputs = ["filtered_logs_vector"]
type = "remap"
source = '''
.container = "vector"
.level = .metadata.level
.logger = .metadata.module_path
if exists(.file) {{ .processed_file = del(.file) }}
del(.metadata)
del(.pid)
del(.source_type)
'''

[transforms.extended_logs]
inputs = ["extended_logs_*"]
type = "remap"
source = '''
.namespace = "{namespace}"
.cluster = "{cluster_name}"
.role = "{role_name}"
.roleGroup = "{role_group_name}"
'''

[sinks.aggregator]
inputs = ["extended_logs"]
type = "vector"
address = "{vector_aggregator_address}"
"#,
        namespace = role_group.cluster.namespace.clone().unwrap_or_default(),
        cluster_name = role_group.cluster.name,
        role_name = role_group.role,
        role_group_name = role_group.role_group
    )
}
