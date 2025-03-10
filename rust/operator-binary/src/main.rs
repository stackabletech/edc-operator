mod affinity;
mod controller;
mod crd;
mod product_logging;

use std::sync::Arc;

use clap::{crate_description, crate_version, Parser};
use crd::{EDCCluster, APP_NAME};
use futures::stream::StreamExt;
use stackable_operator::{
    cli::{Command, ProductOperatorRun},
    k8s_openapi::api::{
        apps::v1::StatefulSet,
        core::v1::{ConfigMap, Service},
    },
    kube::runtime::{watcher, Controller},
    logging::controller::report_controller_reconciled,
    CustomResourceExt,
};

use crate::controller::EDC_CONTROLLER_NAME;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

const OPERATOR_NAME: &str = "edc.stackable.tech";

#[derive(Parser)]
#[clap(about, author)]
struct Opts {
    #[clap(subcommand)]
    cmd: Command,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    match opts.cmd {
        Command::Crd => EDCCluster::print_yaml_schema(built_info::PKG_VERSION)?,
        Command::Run(ProductOperatorRun {
            product_config,
            watch_namespace,
            tracing_target,
        }) => {
            stackable_operator::logging::initialize_logging(
                "EDC_OPERATOR_LOG",
                APP_NAME,
                tracing_target,
            );
            stackable_operator::utils::print_startup_string(
                crate_description!(),
                crate_version!(),
                built_info::GIT_VERSION,
                built_info::TARGET,
                built_info::BUILT_TIME_UTC,
                built_info::RUSTC_VERSION,
            );

            let product_config = product_config.load(&[
                "deploy/config-spec/properties.yaml",
                "/etc/stackable/edc-operator/config-spec/properties.yaml",
            ])?;

            let client =
                stackable_operator::client::create_client(Some(OPERATOR_NAME.to_string())).await?;

            Controller::new(
                watch_namespace.get_api::<EDCCluster>(&client),
                watcher::Config::default(),
            )
            .owns(
                watch_namespace.get_api::<Service>(&client),
                watcher::Config::default(),
            )
            .owns(
                watch_namespace.get_api::<StatefulSet>(&client),
                watcher::Config::default(),
            )
            .owns(
                watch_namespace.get_api::<ConfigMap>(&client),
                watcher::Config::default(),
            )
            .shutdown_on_signal()
            .run(
                controller::reconcile_edc,
                controller::error_policy,
                Arc::new(controller::Ctx {
                    client: client.clone(),
                    product_config,
                }),
            )
            .map(|res| {
                report_controller_reconciled(
                    &client,
                    &format!("{EDC_CONTROLLER_NAME}.{OPERATOR_NAME}"),
                    &res,
                );
            })
            .collect::<()>()
            .await;
        }
    }

    Ok(())
}
