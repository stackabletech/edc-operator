//! This file contains the definition of all the custom resources that this Operator manages.
//! In this case, it is only the `EDCCluster`.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt, Snafu};
use stackable_operator::{
    commons::{
        affinity::StackableAffinity,
        cluster_operation::ClusterOperation,
        product_image_selection::ProductImage,
        resources::{
            CpuLimitsFragment, MemoryLimitsFragment, NoRuntimeLimits, NoRuntimeLimitsFragment,
            PvcConfig, PvcConfigFragment, Resources, ResourcesFragment,
        },
        s3,
    },
    config::{
        fragment,
        fragment::{Fragment, ValidationError},
        merge::Merge,
    },
    k8s_openapi::apimachinery::pkg::api::resource::Quantity,
    kube::{runtime::reflector::ObjectRef, CustomResource, ResourceExt},
    product_config_utils::{self, Configuration},
    product_logging::{self, spec::Logging},
    role_utils::{Role, RoleGroupRef},
    schemars::{self, JsonSchema},
    status::condition::{ClusterCondition, HasStatusCondition},
};
use strum::{Display, EnumIter};

use crate::affinity::get_affinity;

pub const APP_NAME: &str = "edc";
// directories
pub const STACKABLE_SECRETS_DIR: &str = "/stackable/secrets";
pub const STACKABLE_CONFIG_DIR: &str = "/stackable/config";
pub const STACKABLE_CONFIG_DIR_NAME: &str = "config";
pub const STACKABLE_CERT_MOUNT_DIR: &str = "/stackable/mount/cert";
pub const STACKABLE_CERT_MOUNT_DIR_NAME: &str = "cert-mount";
pub const STACKABLE_LOG_DIR: &str = "/stackable/log";
pub const STACKABLE_LOG_DIR_NAME: &str = "log";
pub const STACKABLE_LOG_CONFIG_MOUNT_DIR: &str = "/stackable/mount/log-config";
pub const STACKABLE_LOG_CONFIG_MOUNT_DIR_NAME: &str = "log-config-mount";
pub const STACKABLE_CERTS_DIR: &str = "/stackable/certificates";
// config file names
pub const CONFIG_PROPERTIES: &str = "config.properties";
pub const LOGGING_PROPERTIES: &str = "logging.properties";
pub const JVM_SECURITY_PROPERTIES: &str = "security.properties";
// secret keys
pub const STACKABLE_CERT_MOUNT_KEYSTORE: &str = "cert.pfx";
pub const STACKABLE_CERT_MOUNT_VAULT: &str = "vault.properties";
// config properties (sorted alphabetically)
pub const EDC_API_AUTH_KEY: &str = "edc.api.auth.key";
pub const EDC_DSP_CALLBACK_ADDRESS: &str = "edc.dsp.callback.address";
pub const EDC_DATAPLANE_TOKEN_VALIDATION_ENDPOINT: &str = "edc.dataplane.token.validation.endpoint";
pub const EDC_FS_CONFIG: &str = "edc.fs.config";
pub const EDC_HOSTNAME: &str = "edc.hostname";
pub const EDC_IDS_ID: &str = "edc.ids.id";
pub const EDC_IONOS_ACCESS_KEY: &str = "edc.ionos.access.key";
pub const EDC_IONOS_SECRET_KEY: &str = "edc.ionos.secret.key";
pub const EDC_IONOS_ENDPOINT: &str = "edc.ionos.endpoint";
pub const EDC_KEYSTORE: &str = "edc.keystore";
pub const EDC_PARTICIPANT_ID: &str = "edc.participant.id";
pub const EDC_VAULT: &str = "edc.vault";
pub const EDC_VAULT_CERTIFICATE: &str = "edc.vault.certificate";
pub const EDC_VAULT_CLIENTID: &str = "edc.vault.clientid";
pub const EDC_VAULT_NAME: &str = "edc.vault.name";
pub const EDC_VAULT_TENANTID: &str = "edc.vault.tenantid";
pub const EDC_VAULT_HASHICORP_URL: &str = "edc.vault.hashicorp.url";
pub const EDC_VAULT_HASHICORP_TOKEN: &str = "edc.vault.hashicorp.token";
pub const WEB_HTTP_PORT: &str = "web.http.port";
pub const WEB_HTTP_PATH: &str = "web.http.path";
pub const WEB_HTTP_MANAGEMENT_PORT: &str = "web.http.management.port";
pub const WEB_HTTP_MANAGEMENT_PATH: &str = "web.http.management.path";
pub const WEB_HTTP_PROTOCOL_PORT: &str = "web.http.protocol.port";
pub const WEB_HTTP_PROTOCOL_PATH: &str = "web.http.protocol.path";
pub const WEB_HTTP_PUBLIC_PORT: &str = "web.http.public.port";
pub const WEB_HTTP_PUBLIC_PATH: &str = "web.http.public.path";
pub const WEB_HTTP_CONTROL_PORT: &str = "web.http.control.port";
pub const WEB_HTTP_CONTROL_PATH: &str = "web.http.control.path";
// S3
pub const SECRET_KEY_S3_ACCESS_KEY: &str = "accessKey";
pub const SECRET_KEY_S3_SECRET_KEY: &str = "secretKey";
// default ports
pub const HTTP_PORT_NAME: &str = "http";
pub const HTTP_PORT: u16 = 8181;
pub const CONTROL_PORT_NAME: &str = "control";
pub const CONTROL_PORT: u16 = 8283;
pub const MANAGEMENT_PORT_NAME: &str = "management";
pub const MANAGEMENT_PORT: u16 = 8182;
pub const PROTOCOL_PORT_NAME: &str = "protocol";
pub const PROTOCOL_PORT: u16 = 8282;
pub const PUBLIC_PORT_NAME: &str = "public";
pub const PUBLIC_PORT: u16 = 8284;

// logging
pub const _JAVA_LOGGING: &str = "java-logging.properties";
pub const EDC_CONNECTOR_JAVA_LOG_FILE: &str = "logging.properties";

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("no metastore role configuration provided"))]
    MissingMetaStoreRole,
    #[snafu(display("fragment validation failure"))]
    FragmentValidationFailure { source: ValidationError },
}

#[derive(Clone, CustomResource, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[kube(
    group = "edc.stackable.tech",
    version = "v1alpha1",
    kind = "EDCCluster",
    plural = "edcclusters",
    shortname = "edc",
    status = "EDCClusterStatus",
    namespaced,
    crates(
        kube_core = "stackable_operator::kube::core",
        k8s_openapi = "stackable_operator::k8s_openapi",
        schemars = "stackable_operator::schemars"
    )
)]
pub struct EDCClusterSpec {
    /// General Hive metastore cluster settings
    pub cluster_config: EDCClusterConfig,
    /// Cluster operations like pause reconciliation or cluster stop.
    #[serde(default)]
    pub cluster_operation: ClusterOperation,
    /// The image to use. In this example this will be an nginx image
    pub image: ProductImage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connectors: Option<Role<ConnectorConfigFragment>>,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EDCClusterConfig {
    /// Name of the Vector aggregator discovery ConfigMap.
    /// It must contain the key `ADDRESS` with the address of the Vector aggregator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_aggregator_config_map_name: Option<String>,
    /// In the future this setting will control, which ListenerClass <https://docs.stackable.tech/home/stable/listener-operator/listenerclass.html>
    /// will be used to expose the service.
    /// Currently only a subset of the ListenerClasses are supported by choosing the type of the created Services
    /// by looking at the ListenerClass name specified,
    /// In a future release support for custom ListenerClasses will be introduced without a breaking change:
    ///
    /// * cluster-internal: Use a ClusterIP service
    ///
    /// * external-unstable: Use a NodePort service
    ///
    /// * external-stable: Use a LoadBalancer service
    #[serde(default)]
    pub listener_class: CurrentlySupportedListenerClasses,

    pub cert_secret: String,

    pub ionos: Ionos,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ionos {
    pub token_secret: String,
    pub s3: s3::S3BucketDef,
}
// TODO: the secret should be mounted as an env var, and then in the secret should be a EDC_IONOS_TOKEN var with the value.
// The jar should be able to pick it up

// TODO: Temporary solution until listener-operator is finished
#[derive(Clone, Debug, Default, Display, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum CurrentlySupportedListenerClasses {
    #[default]
    #[serde(rename = "cluster-internal")]
    ClusterInternal,
    #[serde(rename = "external-unstable")]
    ExternalUnstable,
    #[serde(rename = "external-stable")]
    ExternalStable,
}

impl CurrentlySupportedListenerClasses {
    pub fn k8s_service_type(&self) -> String {
        match self {
            CurrentlySupportedListenerClasses::ClusterInternal => "ClusterIP".to_string(),
            CurrentlySupportedListenerClasses::ExternalUnstable => "NodePort".to_string(),
            CurrentlySupportedListenerClasses::ExternalStable => "LoadBalancer".to_string(),
        }
    }
}

#[derive(Display)]
#[strum(serialize_all = "camelCase")]
pub enum EDCRole {
    #[strum(serialize = "server")]
    Connector,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Display,
    Eq,
    EnumIter,
    JsonSchema,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Container {
    Connector,
    Vector,
}

#[derive(Clone, Debug, Default, JsonSchema, PartialEq, Fragment)]
#[fragment_attrs(
    derive(
        Clone,
        Debug,
        Default,
        Deserialize,
        Merge,
        JsonSchema,
        PartialEq,
        Serialize
    ),
    serde(rename_all = "camelCase")
)]
pub struct ConnectorStorageConfig {
    #[fragment_attrs(serde(default))]
    pub data: PvcConfig,
}

#[derive(Clone, Debug, Default, Fragment, JsonSchema, PartialEq)]
#[fragment_attrs(
    derive(
        Clone,
        Debug,
        Default,
        Deserialize,
        Merge,
        JsonSchema,
        PartialEq,
        Serialize
    ),
    serde(rename_all = "camelCase")
)]
pub struct ConnectorConfig {
    #[fragment_attrs(serde(default))]
    pub resources: Resources<ConnectorStorageConfig, NoRuntimeLimits>,
    #[fragment_attrs(serde(default))]
    pub logging: Logging<Container>,
    #[fragment_attrs(serde(default))]
    pub affinity: StackableAffinity,
}

impl ConnectorConfig {
    fn default_config(cluster_name: &str, role: &EDCRole) -> ConnectorConfigFragment {
        ConnectorConfigFragment {
            resources: ResourcesFragment {
                cpu: CpuLimitsFragment {
                    min: Some(Quantity("200m".to_owned())),
                    max: Some(Quantity("4".to_owned())),
                },
                memory: MemoryLimitsFragment {
                    limit: Some(Quantity("2Gi".to_owned())),
                    runtime_limits: NoRuntimeLimitsFragment {},
                },
                storage: ConnectorStorageConfigFragment {
                    data: PvcConfigFragment {
                        capacity: Some(Quantity("2Gi".to_owned())),
                        storage_class: None,
                        selectors: None,
                    },
                },
            },
            logging: product_logging::spec::default_logging(),
            affinity: get_affinity(cluster_name, role),
        }
    }
}

// TODO: Temporary solution until listener-operator is finished
#[derive(Clone, Debug, Display, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ServiceType {
    NodePort,
    ClusterIP,
}

impl Default for ServiceType {
    fn default() -> Self {
        Self::NodePort
    }
}

impl Configuration for ConnectorConfigFragment {
    type Configurable = EDCCluster;

    fn compute_env(
        &self,
        _connector: &Self::Configurable,
        _role_name: &str,
    ) -> Result<BTreeMap<String, Option<String>>, product_config_utils::Error> {
        let result = BTreeMap::new();
        // no ENV args necessary
        Ok(result)
    }

    fn compute_cli(
        &self,
        _connector: &Self::Configurable,
        _role_name: &str,
    ) -> Result<BTreeMap<String, Option<String>>, product_config_utils::Error> {
        let result = BTreeMap::new();
        // No CLI args necessary
        Ok(result)
    }

    fn compute_files(
        &self,
        edc: &Self::Configurable,
        _role_name: &str,
        file: &str,
    ) -> Result<BTreeMap<String, Option<String>>, product_config_utils::Error> {
        let name = edc.name_unchecked();

        let mut result = BTreeMap::new();

        if file == CONFIG_PROPERTIES {
            result.insert(EDC_HOSTNAME.to_owned(), Some(name.to_owned()));
            result.insert(EDC_IDS_ID.to_owned(), Some(format!("urn:connector:{name}")));
            result.insert(EDC_PARTICIPANT_ID.to_owned(), Some(name.to_owned()));
            result.insert(
                EDC_DSP_CALLBACK_ADDRESS.to_owned(),
                Some(format!("http://{name}:{PROTOCOL_PORT}/protocol")),
            );
            // This is the password you need to pass in to the API via a header
            // It should probably be set in a secret
            result.insert(EDC_API_AUTH_KEY.to_owned(), Some("password".to_owned()));
            // Ports
            result.insert(WEB_HTTP_PORT.to_owned(), Some(HTTP_PORT.to_string()));
            result.insert(WEB_HTTP_PATH.to_owned(), Some("/api".to_owned()));
            result.insert(
                WEB_HTTP_CONTROL_PORT.to_owned(),
                Some(CONTROL_PORT.to_string()),
            );
            result.insert(
                WEB_HTTP_CONTROL_PATH.to_owned(),
                Some("/control".to_owned()),
            );
            result.insert(
                WEB_HTTP_MANAGEMENT_PORT.to_owned(),
                Some(MANAGEMENT_PORT.to_string()),
            );
            result.insert(
                WEB_HTTP_MANAGEMENT_PATH.to_owned(),
                Some("/management".to_owned()),
            );
            result.insert(
                WEB_HTTP_PROTOCOL_PORT.to_owned(),
                Some(PROTOCOL_PORT.to_string()),
            );
            result.insert(
                WEB_HTTP_PROTOCOL_PATH.to_owned(),
                Some("/protocol".to_owned()),
            );
            result.insert(
                WEB_HTTP_PUBLIC_PORT.to_owned(),
                Some(PUBLIC_PORT.to_string()),
            );
            result.insert(WEB_HTTP_PUBLIC_PATH.to_owned(), Some("/public".to_owned()));

            result.insert(
                EDC_DATAPLANE_TOKEN_VALIDATION_ENDPOINT.to_owned(),
                Some(format!("http://{}:{}/control/token", name, CONTROL_PORT)),
            );

            // result.insert(
            //     EDC_RECEIVER_HTTP_ENDPOINT.to_owned(),
            //     Some("http://backend:4000/receiver/urn:connector:provider/callback".to_owned()),
            // ); // TODO backend URL shouldn't be hardcoded here. Possibly part of the CRD?
            // result.insert(
            //     EDC_PUBLIC_KEY_ALIAS.to_owned(),
            //     Some("public-key".to_owned()),
            // );
            // result.insert(
            //     EDC_TRANSFER_DATAPLANE_TOKEN_SIGNER_PRIVATEKEY_ALIAS.to_owned(),
            //     Some("1".to_owned()),
            // );
            // result.insert(
            //     EDC_TRANSFER_PROXY_TOKEN_SIGNER_PRIVATEKEY_ALIAS.to_owned(),
            //     Some("1".to_owned()),
            // );
            // result.insert(
            //     EDC_TRANSFER_PROXY_TOKEN_VERIFIER_PUBLICKEY_ALIAS.to_owned(),
            //     Some("public-key".to_owned()),
            // );

            result.insert(EDC_VAULT_CLIENTID.to_owned(), Some("company1".to_owned()));
            result.insert(EDC_VAULT_TENANTID.to_owned(), Some("1".to_owned()));
            result.insert(
                EDC_VAULT_CERTIFICATE.to_owned(),
                Some("./resources".to_owned()), // TODO
            );
            result.insert(EDC_VAULT_NAME.to_owned(), Some("ionos".to_owned()));
            result.insert(
                EDC_VAULT_HASHICORP_URL.to_owned(),
                Some("http://consumer-vault:8200".to_owned()), // TODO probably also a CRD arg
            );
            result.insert(
                EDC_VAULT_HASHICORP_TOKEN.to_owned(),
                Some("dev-token".to_owned()), // TODO needs to be provided as a secret
            );
            result.insert(
                EDC_KEYSTORE.to_owned(),
                Some(format!(
                    "{}/{}",
                    STACKABLE_CERT_MOUNT_DIR, STACKABLE_CERT_MOUNT_KEYSTORE
                )),
            );
            result.insert(
                EDC_VAULT.to_owned(),
                Some(format!(
                    "{}/{}",
                    STACKABLE_CERT_MOUNT_DIR, STACKABLE_CERT_MOUNT_VAULT
                )),
            );
        }

        Ok(result)
    }
}

#[derive(Clone, Default, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EDCClusterStatus {
    pub conditions: Vec<ClusterCondition>,
}

impl HasStatusCondition for EDCCluster {
    fn conditions(&self) -> Vec<ClusterCondition> {
        match &self.status {
            Some(status) => status.conditions.clone(),
            None => vec![],
        }
    }
}

#[derive(Debug, Snafu)]
#[snafu(display("object has no namespace associated"))]
pub struct NoNamespaceError;

impl EDCCluster {
    /// The name of the role-level load-balanced Kubernetes `Service`
    pub fn server_role_service_name(&self) -> Option<&str> {
        self.metadata.name.as_deref()
    }

    /// Metadata about a server rolegroup
    pub fn server_rolegroup_ref(&self, group_name: impl Into<String>) -> RoleGroupRef<EDCCluster> {
        RoleGroupRef {
            cluster: ObjectRef::from_obj(self),
            role: EDCRole::Connector.to_string(),
            role_group: group_name.into(),
        }
    }

    /// List all pods expected to form the cluster
    ///
    /// We try to predict the pods here rather than looking at the current cluster state in order to
    /// avoid instance churn.
    pub fn pods(&self) -> Result<impl Iterator<Item = PodRef> + '_, NoNamespaceError> {
        let ns = self.metadata.namespace.clone().context(NoNamespaceSnafu)?;
        Ok(self
            .spec
            .connectors
            .iter()
            .flat_map(|role| &role.role_groups)
            // Order rolegroups consistently, to avoid spurious downstream rewrites
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .flat_map(move |(rolegroup_name, rolegroup)| {
                let rolegroup_ref = self.server_rolegroup_ref(rolegroup_name);
                let ns = ns.clone();
                (0..rolegroup.replicas.unwrap_or(0)).map(move |i| PodRef {
                    namespace: ns.clone(),
                    role_group_service_name: rolegroup_ref.object_name(),
                    pod_name: format!("{}-{}", rolegroup_ref.object_name(), i),
                })
            }))
    }

    pub fn get_role(&self, role: &EDCRole) -> Option<&Role<ConnectorConfigFragment>> {
        match role {
            EDCRole::Connector => self.spec.connectors.as_ref(),
        }
    }

    /// Retrieve and merge resource configs for role and role groups
    pub fn merged_config(
        &self,
        role: &EDCRole,
        role_group: &str,
    ) -> Result<ConnectorConfig, Error> {
        // Initialize the result with all default values as baseline
        let conf_defaults = ConnectorConfig::default_config(&self.name_any(), role);

        let role = self.get_role(role).context(MissingMetaStoreRoleSnafu)?;

        // Retrieve role resource config
        let mut conf_role = role.config.config.to_owned();

        // Retrieve rolegroup specific resource config
        let mut conf_rolegroup = role
            .role_groups
            .get(role_group)
            .map(|rg| rg.config.config.clone())
            .unwrap_or_default();

        // Merge more specific configs into default config
        // Hierarchy is:
        // 1. RoleGroup
        // 2. Role
        // 3. Default
        conf_role.merge(&conf_defaults);
        conf_rolegroup.merge(&conf_role);

        tracing::debug!("Merged config: {:?}", conf_rolegroup);
        fragment::validate(conf_rolegroup).context(FragmentValidationFailureSnafu)
    }
}

/// Reference to a single `Pod` that is a component of a [`EDCCluster`]
/// Used for service discovery.
pub struct PodRef {
    pub namespace: String,
    pub role_group_service_name: String,
    pub pod_name: String,
}

impl PodRef {
    pub fn fqdn(&self) -> String {
        format!(
            "{}.{}.{}.svc.cluster.local",
            self.pod_name, self.role_group_service_name, self.namespace
        )
    }
}
