//! Ensures that `Pod`s are configured and running for each [`EDCCluster`]
use crate::product_logging::{extend_role_group_config_map, resolve_vector_aggregator_address};
use crate::OPERATOR_NAME;

use crate::crd::{
    ConnectorConfig, Container, EDCCluster, EDCClusterStatus, EDCRole, APP_NAME, CONFIG_PROPERTIES,
    CONTROL_PORT, CONTROL_PORT_NAME, EDC_FS_CONFIG, EDC_IONOS_ACCESS_KEY, EDC_IONOS_ENDPOINT,
    EDC_IONOS_SECRET_KEY, HTTP_PORT, HTTP_PORT_NAME, JVM_SECURITY_PROPERTIES, LOGGING_PROPERTIES,
    MANAGEMENT_PORT, MANAGEMENT_PORT_NAME, PROTOCOL_PORT, PROTOCOL_PORT_NAME, PUBLIC_PORT,
    PUBLIC_PORT_NAME, SECRET_KEY_S3_ACCESS_KEY, SECRET_KEY_S3_SECRET_KEY, STACKABLE_CERTS_DIR,
    STACKABLE_CERT_MOUNT_DIR, STACKABLE_CERT_MOUNT_DIR_NAME, STACKABLE_CONFIG_DIR,
    STACKABLE_CONFIG_DIR_NAME, STACKABLE_LOG_CONFIG_MOUNT_DIR, STACKABLE_LOG_CONFIG_MOUNT_DIR_NAME,
    STACKABLE_LOG_DIR, STACKABLE_LOG_DIR_NAME, STACKABLE_SECRETS_DIR,
};
use product_config::{
    types::PropertyNameKind,
    writer::{to_java_properties_string, PropertiesWriterError},
    ProductConfigManager,
};
use snafu::{OptionExt, ResultExt, Snafu};
use stackable_operator::{
    builder::{
        configmap::ConfigMapBuilder,
        meta::ObjectMetaBuilder,
        pod::{
            container::ContainerBuilder,
            resources::ResourceRequirementsBuilder,
            security::PodSecurityContextBuilder,
            volume::{
                SecretOperatorVolumeSourceBuilder, SecretOperatorVolumeSourceBuilderError,
                VolumeBuilder,
            },
            PodBuilder,
        },
    },
    client::GetApi,
    cluster_resources::{ClusterResourceApplyStrategy, ClusterResources},
    commons::{
        authentication::tls::{CaCert, TlsVerification},
        product_image_selection::ResolvedProductImage,
        rbac::build_rbac_resources,
        s3::S3ConnectionSpec,
        secret_class::SecretClassVolumeError,
    },
    k8s_openapi::{
        api::core::v1::SecretVolumeSource,
        api::{
            apps::v1::{StatefulSet, StatefulSetSpec},
            core::v1::{
                ConfigMap, ConfigMapVolumeSource, EmptyDirVolumeSource, Probe, Service,
                ServicePort, ServiceSpec, TCPSocketAction, Volume,
            },
        },
        apimachinery::pkg::{apis::meta::v1::LabelSelector, util::intstr::IntOrString},
    },
    kube::{runtime::controller::Action, Resource, ResourceExt},
    kvp::{LabelError, Labels, ObjectLabels},
    logging::controller::ReconcilerError,
    memory::{BinaryMultiple, MemoryQuantity},
    product_config_utils::{transform_all_roles_to_config, validate_all_roles_and_groups_config},
    product_logging::{
        self,
        spec::{
            ConfigMapLogConfig, ContainerLogConfig, ContainerLogConfigChoice,
            CustomContainerLogConfig,
        },
    },
    role_utils::RoleGroupRef,
    status::condition::{
        compute_conditions, operations::ClusterOperationsConditionBuilder,
        statefulset::StatefulSetConditionBuilder,
    },
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};
use strum::EnumDiscriminants;
use tracing::warn;

pub const EDC_CONTROLLER_NAME: &str = "edccluster";
const DOCKER_IMAGE_BASE_NAME: &str = "edc";

pub const MAX_LOG_FILES_SIZE: MemoryQuantity = MemoryQuantity {
    value: 10.0,
    unit: BinaryMultiple::Mebi,
};

pub struct Ctx {
    pub client: stackable_operator::client::Client,
    pub product_config: ProductConfigManager,
}

#[derive(Snafu, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(strum::IntoStaticStr))]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[snafu(display("object defines no namespace"))]
    ObjectHasNoNamespace,
    #[snafu(display("object defines no metastore role"))]
    NoServerRole,
    #[snafu(display("failed to calculate global service name"))]
    GlobalServiceNameNotFound,
    #[snafu(display("failed to calculate service name for role {rolegroup}"))]
    RoleGroupServiceNameNotFound { rolegroup: RoleGroupRef<EDCCluster> },
    #[snafu(display("failed to apply global Service"))]
    ApplyRoleService {
        source: stackable_operator::cluster_resources::Error,
    },
    #[snafu(display("failed to apply Service for {rolegroup}"))]
    ApplyRoleGroupService {
        source: stackable_operator::cluster_resources::Error,
        rolegroup: RoleGroupRef<EDCCluster>,
    },
    #[snafu(display("failed to build ConfigMap for {rolegroup}"))]
    BuildRoleGroupConfig {
        source: stackable_operator::builder::configmap::Error,
        rolegroup: RoleGroupRef<EDCCluster>,
    },
    #[snafu(display("failed to apply ConfigMap for {rolegroup}"))]
    ApplyRoleGroupConfig {
        source: stackable_operator::cluster_resources::Error,
        rolegroup: RoleGroupRef<EDCCluster>,
    },
    #[snafu(display("failed to apply StatefulSet for {rolegroup}"))]
    ApplyRoleGroupStatefulSet {
        source: stackable_operator::cluster_resources::Error,
        rolegroup: RoleGroupRef<EDCCluster>,
    },
    #[snafu(display("failed to generate product config"))]
    GenerateProductConfig {
        source: stackable_operator::product_config_utils::Error,
    },
    #[snafu(display("invalid product config"))]
    InvalidProductConfig {
        source: stackable_operator::product_config_utils::Error,
    },
    #[snafu(display("object is missing metadata to build owner reference"))]
    ObjectMissingMetadataForOwnerRef {
        source: stackable_operator::builder::meta::Error,
    },
    #[snafu(display("failed to apply discovery ConfigMap"))]
    ApplyDiscoveryConfig {
        source: stackable_operator::client::Error,
    },
    #[snafu(display("failed to update status"))]
    ApplyStatus {
        source: stackable_operator::client::Error,
    },
    #[snafu(display("failed to format runtime properties"))]
    PropertiesWriteError { source: PropertiesWriterError },
    #[snafu(display("failed to parse db type {db_type}"))]
    InvalidDbType {
        source: strum::ParseError,
        db_type: String,
    },
    #[snafu(display("failed to resolve S3 connection"))]
    ResolveS3Connection {
        source: stackable_operator::commons::s3::Error,
    },
    #[snafu(display("failed to resolve and merge resource config for role and role group"))]
    FailedToResolveResourceConfig { source: crate::crd::Error },
    #[snafu(display("failed to create EDC container [{name}]"))]
    FailedToCreateEdcContainer {
        source: stackable_operator::builder::pod::container::Error,
        name: String,
    },
    #[snafu(display("failed to create cluster resources"))]
    CreateClusterResources {
        source: stackable_operator::cluster_resources::Error,
    },
    #[snafu(display("failed to delete orphaned resources"))]
    DeleteOrphanedResources {
        source: stackable_operator::cluster_resources::Error,
    },
    #[snafu(display("failed to resolve the Vector aggregator address"))]
    ResolveVectorAggregatorAddress {
        source: crate::product_logging::Error,
    },
    #[snafu(display("failed to add the logging configuration to the ConfigMap [{cm_name}]"))]
    InvalidLoggingConfig {
        source: crate::product_logging::Error,
        cm_name: String,
    },
    #[snafu(display("failed to patch service account"))]
    ApplyServiceAccount {
        source: stackable_operator::cluster_resources::Error,
    },
    #[snafu(display("failed to patch role binding"))]
    ApplyRoleBinding {
        source: stackable_operator::cluster_resources::Error,
    },
    #[snafu(display("failed to build RBAC resources"))]
    BuildRbacResources {
        source: stackable_operator::commons::rbac::Error,
    },
    #[snafu(display(
        "Druid does not support skipping the verification of the tls enabled S3 server"
    ))]
    S3TlsNoVerificationNotSupported,
    #[snafu(display(
        "failed to serialize [{JVM_SECURITY_PROPERTIES}] for group {}",
        rolegroup
    ))]
    JvmSecurityProperties {
        source: PropertiesWriterError,
        rolegroup: String,
    },

    #[snafu(display("failed to build label"))]
    BuildLabel { source: LabelError },

    #[snafu(display("failed to build object meta data"))]
    ObjectMeta {
        source: stackable_operator::builder::meta::Error,
    },

    #[snafu(display("failed to build TLS volume for {volume_name:?}"))]
    BuildTlsVolume {
        source: SecretOperatorVolumeSourceBuilderError,
        volume_name: String,
    },

    #[snafu(display("failed to convert credentials to volume for {volume_name:?}"))]
    CredentialsToVolume {
        source: SecretClassVolumeError,
        volume_name: String,
    },
}
type Result<T, E = Error> = std::result::Result<T, E>;

impl ReconcilerError for Error {
    fn category(&self) -> &'static str {
        ErrorDiscriminants::from(self).into()
    }
}

pub async fn reconcile_edc(edc: Arc<EDCCluster>, ctx: Arc<Ctx>) -> Result<Action> {
    tracing::info!("Starting reconcile");
    let client = &ctx.client;
    let resolved_product_image: ResolvedProductImage = edc
        .spec
        .image
        .resolve(DOCKER_IMAGE_BASE_NAME, crate::built_info::PKG_VERSION);

    let s3_bucket_spec = edc
        .spec
        .cluster_config
        .ionos
        .s3
        .resolve(&ctx.client, edc.get_namespace())
        .await
        .context(ResolveS3ConnectionSnafu)?;

    let validated_config = validate_all_roles_and_groups_config(
        &resolved_product_image.product_version,
        &transform_all_roles_to_config(
            edc.as_ref(),
            [(
                EDCRole::Connector.to_string(),
                (
                    vec![
                        PropertyNameKind::Env,
                        PropertyNameKind::Cli,
                        PropertyNameKind::File(CONFIG_PROPERTIES.to_string()),
                        PropertyNameKind::File(JVM_SECURITY_PROPERTIES.to_string()),
                    ],
                    edc.spec.connectors.clone().context(NoServerRoleSnafu)?,
                ),
            )]
            .into(),
        )
        .context(GenerateProductConfigSnafu)?,
        &ctx.product_config,
        false,
        false,
    )
    .context(InvalidProductConfigSnafu)?;

    let server_config = validated_config
        .get(&EDCRole::Connector.to_string())
        .map(Cow::Borrowed)
        .unwrap_or_default();

    let mut cluster_resources = ClusterResources::new(
        APP_NAME,
        OPERATOR_NAME,
        EDC_CONTROLLER_NAME,
        &edc.object_ref(&()),
        ClusterResourceApplyStrategy::from(&edc.spec.cluster_operation),
    )
    .context(CreateClusterResourcesSnafu)?;

    let (rbac_sa, rbac_rolebinding) = build_rbac_resources(
        edc.as_ref(),
        APP_NAME,
        cluster_resources
            .get_required_labels()
            .context(BuildLabelSnafu)?,
    )
    .context(BuildRbacResourcesSnafu)?;

    let rbac_sa = cluster_resources
        .add(client, rbac_sa)
        .await
        .context(ApplyServiceAccountSnafu)?;
    cluster_resources
        .add(client, rbac_rolebinding)
        .await
        .context(ApplyRoleBindingSnafu)?;

    let server_role_service = build_server_role_service(&edc, &resolved_product_image)?;

    // we have to get the assigned ports
    cluster_resources
        .add(client, server_role_service)
        .await
        .context(ApplyRoleServiceSnafu)?;

    let vector_aggregator_address = resolve_vector_aggregator_address(&edc, client)
        .await
        .context(ResolveVectorAggregatorAddressSnafu)?;

    let mut ss_cond_builder = StatefulSetConditionBuilder::default();

    for (rolegroup_name, rolegroup_config) in server_config.iter() {
        let rolegroup = edc.server_rolegroup_ref(rolegroup_name);

        let config = edc
            .merged_config(&EDCRole::Connector, &rolegroup.role_group)
            .context(FailedToResolveResourceConfigSnafu)?;

        let rg_service = build_rolegroup_service(&edc, &resolved_product_image, &rolegroup)?;
        let rg_configmap = build_connector_rolegroup_config_map(
            &edc,
            &resolved_product_image,
            &rolegroup,
            rolegroup_config,
            &config,
            s3_bucket_spec.connection.as_ref(),
            vector_aggregator_address.as_deref(),
        )?;
        let rg_statefulset = build_server_rolegroup_statefulset(
            &edc,
            &resolved_product_image,
            &rolegroup,
            rolegroup_config,
            &config,
            s3_bucket_spec.connection.as_ref(),
            &rbac_sa.name_any(),
        )?;

        cluster_resources
            .add(client, rg_service)
            .await
            .context(ApplyRoleGroupServiceSnafu {
                rolegroup: rolegroup.clone(),
            })?;

        cluster_resources
            .add(client, rg_configmap)
            .await
            .context(ApplyRoleGroupConfigSnafu {
                rolegroup: rolegroup.clone(),
            })?;

        ss_cond_builder.add(
            cluster_resources
                .add(client, rg_statefulset)
                .await
                .context(ApplyRoleGroupStatefulSetSnafu {
                    rolegroup: rolegroup.clone(),
                })?,
        );
    }

    let cluster_operation_cond_builder =
        ClusterOperationsConditionBuilder::new(&edc.spec.cluster_operation);

    let status = EDCClusterStatus {
        conditions: compute_conditions(
            edc.as_ref(),
            &[&ss_cond_builder, &cluster_operation_cond_builder],
        ),
    };

    client
        .apply_patch_status(OPERATOR_NAME, &*edc, &status)
        .await
        .context(ApplyStatusSnafu)?;

    cluster_resources
        .delete_orphaned_resources(client)
        .await
        .context(DeleteOrphanedResourcesSnafu)?;

    Ok(Action::await_change())
}

pub fn build_server_role_service(
    edc: &EDCCluster,
    resolved_product_image: &ResolvedProductImage,
) -> Result<Service> {
    let role_name = EDCRole::Connector.to_string();

    let role_svc_name = edc
        .server_role_service_name()
        .context(GlobalServiceNameNotFoundSnafu)?;

    let metadata = ObjectMetaBuilder::new()
        .name_and_namespace(edc)
        .name(role_svc_name)
        .ownerreference_from_resource(edc, None, Some(true))
        .context(ObjectMissingMetadataForOwnerRefSnafu)?
        .with_recommended_labels(build_recommended_labels(
            edc,
            &resolved_product_image.app_version_label,
            &role_name,
            "global",
        ))
        .context(ObjectMetaSnafu)?
        .build();

    let service_selector_labels =
        Labels::role_selector(edc, APP_NAME, &role_name).context(BuildLabelSnafu)?;

    let service_spec = ServiceSpec {
        type_: Some(edc.spec.cluster_config.listener_class.k8s_service_type()),
        ports: Some(service_ports()),
        selector: Some(service_selector_labels.into()),
        ..ServiceSpec::default()
    };

    Ok(Service {
        metadata,
        spec: Some(service_spec),
        status: None,
    })
}

/// The rolegroup [`ConfigMap`] configures the rolegroup based on the configuration given by the administrator
fn build_connector_rolegroup_config_map(
    edc: &EDCCluster,
    resolved_product_image: &ResolvedProductImage,
    rolegroup: &RoleGroupRef<EDCCluster>,
    role_group_config: &HashMap<PropertyNameKind, BTreeMap<String, String>>,
    merged_config: &ConnectorConfig,
    s3_conn: Option<&S3ConnectionSpec>,
    vector_aggregator_address: Option<&str>,
) -> Result<ConfigMap> {
    let mut config_properties = String::new();

    for (property_name_kind, config) in role_group_config {
        let mut conf: BTreeMap<String, Option<String>> = Default::default();
        match property_name_kind {
            PropertyNameKind::File(file_name) if file_name == CONFIG_PROPERTIES => {
                if let Some(conn) = s3_conn {
                    if let Some(endpoint) = conn.endpoint() {
                        conf.insert(EDC_IONOS_ENDPOINT.to_string(), Some(endpoint));
                    }
                }

                let transformed_config: BTreeMap<String, Option<String>> = config
                    .iter()
                    .map(|(k, v)| (k.clone(), Some(v.clone())))
                    .collect();
                conf.extend(transformed_config);

                config_properties =
                    to_java_properties_string(conf.iter()).context(PropertiesWriteSnafu)?;
            }
            _ => {}
        }
    }

    // build JVM security properties from configOverrides.
    let jvm_sec_props: BTreeMap<String, Option<String>> = role_group_config
        .get(&PropertyNameKind::File(JVM_SECURITY_PROPERTIES.to_string()))
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|(k, v)| (k, Some(v)))
        .collect();

    let cm_metadata = ObjectMetaBuilder::new()
        .name_and_namespace(edc)
        .name(rolegroup.object_name())
        .ownerreference_from_resource(edc, None, Some(true))
        .context(ObjectMissingMetadataForOwnerRefSnafu)?
        .with_recommended_labels(build_recommended_labels(
            edc,
            &resolved_product_image.app_version_label,
            &rolegroup.role,
            &rolegroup.role_group,
        ))
        .context(ObjectMetaSnafu)?
        .build();

    let mut cm_builder = ConfigMapBuilder::new();

    cm_builder
        .metadata(cm_metadata)
        .add_data(CONFIG_PROPERTIES, config_properties)
        .add_data(
            JVM_SECURITY_PROPERTIES,
            to_java_properties_string(jvm_sec_props.iter()).with_context(|_| {
                JvmSecurityPropertiesSnafu {
                    rolegroup: rolegroup.role_group.clone(),
                }
            })?,
        );

    extend_role_group_config_map(
        rolegroup,
        vector_aggregator_address,
        &merged_config.logging,
        &mut cm_builder,
    )
    .context(InvalidLoggingConfigSnafu {
        cm_name: rolegroup.object_name(),
    })?;

    cm_builder
        .build()
        .with_context(|_| BuildRoleGroupConfigSnafu {
            rolegroup: rolegroup.clone(),
        })
}

/// The rolegroup [`Service`] is a headless service that allows direct access to the instances of a certain rolegroup
///
/// This is mostly useful for internal communication between peers, or for clients that perform client-side load balancing.
fn build_rolegroup_service(
    edc: &EDCCluster,
    resolved_product_image: &ResolvedProductImage,
    rolegroup: &RoleGroupRef<EDCCluster>,
) -> Result<Service> {
    let metadata = ObjectMetaBuilder::new()
        .name_and_namespace(edc)
        .name(&rolegroup.object_name())
        .ownerreference_from_resource(edc, None, Some(true))
        .context(ObjectMissingMetadataForOwnerRefSnafu)?
        .with_recommended_labels(build_recommended_labels(
            edc,
            &resolved_product_image.app_version_label,
            &rolegroup.role,
            &rolegroup.role_group,
        ))
        .context(ObjectMetaSnafu)?
        .build();

    let service_selector_labels =
        Labels::role_group_selector(edc, APP_NAME, &rolegroup.role, &rolegroup.role_group)
            .context(BuildLabelSnafu)?;

    let service_spec = ServiceSpec {
        // Internal communication does not need to be exposed
        type_: Some("ClusterIP".to_string()),
        cluster_ip: Some("None".to_string()),
        ports: Some(service_ports()),
        selector: Some(service_selector_labels.into()),
        publish_not_ready_addresses: Some(true),
        ..ServiceSpec::default()
    };

    Ok(Service {
        metadata,
        spec: Some(service_spec),
        status: None,
    })
}

/// The rolegroup [`StatefulSet`] runs the rolegroup, as configured by the administrator.
///
/// The [`Pod`](`stackable_operator::k8s_openapi::api::core::v1::Pod`)s are accessible through the
/// corresponding [`Service`] (from [`build_rolegroup_service`]).
fn build_server_rolegroup_statefulset(
    edc: &EDCCluster,
    resolved_product_image: &ResolvedProductImage,
    rolegroup_ref: &RoleGroupRef<EDCCluster>,
    metastore_config: &HashMap<PropertyNameKind, BTreeMap<String, String>>,
    merged_config: &ConnectorConfig,
    s3_conn: Option<&S3ConnectionSpec>,
    sa_name: &str,
) -> Result<StatefulSet> {
    let rolegroup = edc
        .spec
        .connectors
        .as_ref()
        .context(NoServerRoleSnafu)?
        .role_groups
        .get(&rolegroup_ref.role_group);
    let mut container_builder =
        ContainerBuilder::new(APP_NAME).context(FailedToCreateEdcContainerSnafu {
            name: APP_NAME.to_string(),
        })?;

    for (property_name_kind, config) in metastore_config {
        if property_name_kind == &PropertyNameKind::Env {
            // overrides
            for (property_name, property_value) in config {
                if property_name.is_empty() {
                    warn!("Received empty property_name for ENV... skipping");
                    continue;
                }
                container_builder.add_env_var(property_name, property_value);
            }
        }
    }

    let mut pod_builder = PodBuilder::new();
    let mut args = Vec::new();

    if let Some(ContainerLogConfig {
        choice: Some(ContainerLogConfigChoice::Automatic(log_config)),
    }) = merged_config.logging.containers.get(&Container::Connector)
    {
        args.push(product_logging::framework::capture_shell_output(
            STACKABLE_LOG_DIR,
            "edc",
            log_config,
        ));
    }

    // S3
    add_s3_volume_and_volume_mounts(s3_conn, &mut container_builder, &mut pod_builder)?;

    let mut java_cmd = vec![];
    java_cmd.extend(args);
    java_cmd.push("java".to_string());
    java_cmd.push(format!(
        "-D{}={}/{}",
        EDC_FS_CONFIG, STACKABLE_CONFIG_DIR, CONFIG_PROPERTIES
    ));
    java_cmd.push(format!(
        "-Djava.util.logging.config.file={}/{}",
        STACKABLE_CONFIG_DIR, LOGGING_PROPERTIES
    ));

    // Add S3 secret and access keys from the files mounted by the secret Operator
    if let Some(c) = s3_conn {
        if c.credentials.is_some() {
            let path = format!("{}/{}", STACKABLE_SECRETS_DIR, SECRET_KEY_S3_ACCESS_KEY);
            java_cmd.push(format!("-D{}=$(cat {})", EDC_IONOS_ACCESS_KEY, path));
            let path = format!("{}/{}", STACKABLE_SECRETS_DIR, SECRET_KEY_S3_SECRET_KEY);
            java_cmd.push(format!("-D{}=$(cat {})", EDC_IONOS_SECRET_KEY, path));
        }
    }

    // JVM security properties configured via configOverrides
    java_cmd.push(format!(
        "-Djava.security.properties={STACKABLE_CONFIG_DIR}/{JVM_SECURITY_PROPERTIES}"
    ));

    // We add this at the and, as the .jar file should be the last argument to the call to the java binary
    java_cmd.extend(vec!["-jar".to_string(), "connector.jar".to_string()]);

    // TODO if a custom container command is needed, add it here (.command)
    let container_edc = container_builder
        .command(vec!["/bin/bash".to_string(), "-c".to_string()])
        .args(vec![format!("{}", java_cmd.join(" "))])
        .image_from_product_image(resolved_product_image)
        .add_volume_mount(STACKABLE_CONFIG_DIR_NAME, STACKABLE_CONFIG_DIR)
        .add_volume_mount(STACKABLE_CERT_MOUNT_DIR_NAME, STACKABLE_CERT_MOUNT_DIR)
        .add_volume_mount(STACKABLE_LOG_DIR_NAME, STACKABLE_LOG_DIR)
        .add_volume_mount(
            STACKABLE_LOG_CONFIG_MOUNT_DIR_NAME,
            STACKABLE_LOG_CONFIG_MOUNT_DIR,
        )
        .add_container_port(HTTP_PORT_NAME, HTTP_PORT.into())
        .add_container_port(CONTROL_PORT_NAME, CONTROL_PORT.into())
        .add_container_port(MANAGEMENT_PORT_NAME, MANAGEMENT_PORT.into())
        .add_container_port(PROTOCOL_PORT_NAME, PROTOCOL_PORT.into())
        .add_container_port(PUBLIC_PORT_NAME, PUBLIC_PORT.into())
        .resources(merged_config.resources.clone().into())
        .readiness_probe(Probe {
            initial_delay_seconds: Some(10),
            period_seconds: Some(10),
            failure_threshold: Some(50),
            tcp_socket: Some(TCPSocketAction {
                port: IntOrString::String(HTTP_PORT_NAME.to_string()),
                ..TCPSocketAction::default()
            }),
            ..Probe::default()
        })
        .liveness_probe(Probe {
            initial_delay_seconds: Some(60),
            period_seconds: Some(20),
            tcp_socket: Some(TCPSocketAction {
                port: IntOrString::String(HTTP_PORT_NAME.to_string()),
                ..TCPSocketAction::default()
            }),
            ..Probe::default()
        })
        .add_env_var_from_secret(
            "EDC_IONOS_TOKEN",
            edc.spec.cluster_config.ionos.token_secret.to_owned(),
            "EDC_IONOS_TOKEN",
        )
        .build();

    let pb_metadata = ObjectMetaBuilder::new()
        .with_recommended_labels(build_recommended_labels(
            edc,
            &resolved_product_image.app_version_label,
            &rolegroup_ref.role,
            &rolegroup_ref.role_group,
        ))
        .context(ObjectMetaSnafu)?
        .build();

    pod_builder
        .metadata(pb_metadata)
        .image_pull_secrets_from_product_image(resolved_product_image)
        .add_container(container_edc)
        .add_volume(stackable_operator::k8s_openapi::api::core::v1::Volume {
            name: STACKABLE_CONFIG_DIR_NAME.to_string(),
            config_map: Some(ConfigMapVolumeSource {
                name: Some(rolegroup_ref.object_name()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .add_volume(Volume {
            name: STACKABLE_LOG_DIR_NAME.to_string(),
            empty_dir: Some(EmptyDirVolumeSource {
                medium: None,
                size_limit: Some(product_logging::framework::calculate_log_volume_size_limit(
                    &[MAX_LOG_FILES_SIZE],
                )),
            }),
            ..Volume::default()
        })
        .add_volume(Volume {
            name: STACKABLE_CERT_MOUNT_DIR_NAME.to_string(),
            secret: Some(SecretVolumeSource {
                secret_name: Some(edc.spec.cluster_config.cert_secret.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .affinity(&merged_config.affinity)
        .service_account_name(sa_name)
        .security_context(
            PodSecurityContextBuilder::new()
                .run_as_user(1000)
                .run_as_group(0)
                .fs_group(1000)
                .build(),
        );

    if let Some(ContainerLogConfig {
        choice:
            Some(ContainerLogConfigChoice::Custom(CustomContainerLogConfig {
                custom: ConfigMapLogConfig { config_map },
            })),
    }) = merged_config.logging.containers.get(&Container::Connector)
    {
        pod_builder.add_volume(Volume {
            name: STACKABLE_LOG_CONFIG_MOUNT_DIR_NAME.to_string(),
            config_map: Some(ConfigMapVolumeSource {
                name: Some(config_map.into()),
                ..ConfigMapVolumeSource::default()
            }),
            ..Volume::default()
        });
    } else {
        pod_builder.add_volume(Volume {
            name: STACKABLE_LOG_CONFIG_MOUNT_DIR_NAME.to_string(),
            config_map: Some(ConfigMapVolumeSource {
                name: Some(rolegroup_ref.object_name()),
                ..ConfigMapVolumeSource::default()
            }),
            ..Volume::default()
        });
    }

    if merged_config.logging.enable_vector_agent {
        pod_builder.add_container(product_logging::framework::vector_container(
            resolved_product_image,
            STACKABLE_CONFIG_DIR_NAME,
            STACKABLE_LOG_DIR_NAME,
            merged_config.logging.containers.get(&Container::Vector),
            ResourceRequirementsBuilder::new()
                .with_cpu_request("250m")
                .with_cpu_limit("500m")
                .with_memory_request("128Mi")
                .with_memory_limit("128Mi")
                .build(),
        ));
    }

    let metadata = ObjectMetaBuilder::new()
        .name_and_namespace(edc)
        .name(&rolegroup_ref.object_name())
        .ownerreference_from_resource(edc, None, Some(true))
        .context(ObjectMissingMetadataForOwnerRefSnafu)?
        .with_recommended_labels(build_recommended_labels(
            edc,
            &resolved_product_image.app_version_label,
            &rolegroup_ref.role,
            &rolegroup_ref.role_group,
        ))
        .context(ObjectMetaSnafu)?
        .build();

    let service_match_labels = Labels::role_group_selector(
        edc,
        APP_NAME,
        &rolegroup_ref.role,
        &rolegroup_ref.role_group,
    )
    .context(BuildLabelSnafu)?;

    let service_spec = StatefulSetSpec {
        pod_management_policy: Some("Parallel".to_string()),
        replicas: rolegroup.and_then(|rg| rg.replicas).map(i32::from),
        selector: LabelSelector {
            match_labels: Some(service_match_labels.into()),
            ..LabelSelector::default()
        },
        service_name: rolegroup_ref.object_name(),
        template: pod_builder.build_template(),
        volume_claim_templates: Some(vec![merged_config
            .resources
            .storage
            .data
            .build_pvc("data", Some(vec!["ReadWriteOnce"]))]),
        ..StatefulSetSpec::default()
    };

    Ok(StatefulSet {
        metadata,
        spec: Some(service_spec),
        status: None,
    })
}

fn add_s3_volume_and_volume_mounts(
    s3_conn: Option<&S3ConnectionSpec>,
    cb_druid: &mut ContainerBuilder,
    pb: &mut PodBuilder,
) -> Result<()> {
    if let Some(s3_conn) = s3_conn {
        if let Some(credentials) = &s3_conn.credentials {
            const VOLUME_NAME: &str = "s3-credentials";
            pb.add_volume(credentials.to_volume(VOLUME_NAME).context(
                CredentialsToVolumeSnafu {
                    volume_name: VOLUME_NAME,
                },
            )?);
            cb_druid.add_volume_mount(VOLUME_NAME, STACKABLE_SECRETS_DIR);
        }

        if let Some(tls) = &s3_conn.tls {
            match &tls.verification {
                TlsVerification::None {} => return S3TlsNoVerificationNotSupportedSnafu.fail(),
                TlsVerification::Server(server_verification) => {
                    match &server_verification.ca_cert {
                        CaCert::WebPki {} => {}
                        CaCert::SecretClass(secret_class) => {
                            let volume_name = format!("{secret_class}-tls-certificate");

                            let volume = VolumeBuilder::new(&volume_name)
                                .ephemeral(
                                    SecretOperatorVolumeSourceBuilder::new(secret_class)
                                        .build()
                                        .context(BuildTlsVolumeSnafu {
                                            volume_name: &volume_name,
                                        })?,
                                )
                                .build();
                            pb.add_volume(volume);
                            cb_druid.add_volume_mount(
                                &volume_name,
                                format!("{STACKABLE_CERTS_DIR}/{volume_name}"),
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn error_policy(_obj: Arc<EDCCluster>, _error: &Error, _ctx: Arc<Ctx>) -> Action {
    Action::requeue(Duration::from_secs(5))
}

fn service_ports() -> Vec<ServicePort> {
    vec![
        ServicePort {
            name: Some(HTTP_PORT_NAME.to_string()),
            port: HTTP_PORT.into(),
            protocol: Some("TCP".to_string()),
            ..ServicePort::default()
        },
        ServicePort {
            name: Some(CONTROL_PORT_NAME.to_string()),
            port: CONTROL_PORT.into(),
            protocol: Some("TCP".to_string()),
            ..ServicePort::default()
        },
        ServicePort {
            name: Some(MANAGEMENT_PORT_NAME.to_string()),
            port: MANAGEMENT_PORT.into(),
            protocol: Some("TCP".to_string()),
            ..ServicePort::default()
        },
        ServicePort {
            name: Some(PROTOCOL_PORT_NAME.to_string()),
            port: PROTOCOL_PORT.into(),
            protocol: Some("TCP".to_string()),
            ..ServicePort::default()
        },
        ServicePort {
            name: Some(PUBLIC_PORT_NAME.to_string()),
            port: PUBLIC_PORT.into(),
            protocol: Some("TCP".to_string()),
            ..ServicePort::default()
        },
    ]
}

/// Creates recommended `ObjectLabels` to be used in deployed resources
pub fn build_recommended_labels<'a, T>(
    owner: &'a T,
    app_version: &'a str,
    role: &'a str,
    role_group: &'a str,
) -> ObjectLabels<'a, T> {
    ObjectLabels {
        owner,
        app_name: APP_NAME,
        app_version,
        operator_name: OPERATOR_NAME,
        controller_name: EDC_CONTROLLER_NAME,
        role,
        role_group,
    }
}
