/// Macro to define a "get" command struct and handler function.
///
/// Generates a command struct with an `id: i64` field and an async function
/// that fetches the entity by ID and prints it as JSON.
///
/// # Example
/// ```ignore
/// define_get_command!(GetRoasterCommand, get_roaster, RoasterId, roasters);
/// ```
macro_rules! define_get_command {
    ($cmd_name:ident, $fn_name:ident, $id_type:ty, $client_method:ident) => {
        #[derive(Debug, clap::Args)]
        pub struct $cmd_name {
            #[arg(long)]
            pub id: i64,
        }

        pub async fn $fn_name(
            client: &crate::infrastructure::client::BrewlogClient,
            command: $cmd_name,
        ) -> anyhow::Result<()> {
            let entity = client
                .$client_method()
                .get(<$id_type>::new(command.id))
                .await?;
            super::print_json(&entity)
        }
    };
}

/// Macro to define a "delete" command struct and handler function.
///
/// Generates a command struct with an `id: i64` field and an async function
/// that deletes the entity by ID and prints a confirmation JSON response.
///
/// # Example
/// ```ignore
/// define_delete_command!(DeleteRoasterCommand, delete_roaster, RoasterId, roasters, "roaster");
/// ```
macro_rules! define_delete_command {
    ($cmd_name:ident, $fn_name:ident, $id_type:ty, $client_method:ident, $resource_name:literal) => {
        #[derive(Debug, clap::Args)]
        pub struct $cmd_name {
            #[arg(long)]
            pub id: i64,
        }

        pub async fn $fn_name(
            client: &crate::infrastructure::client::BrewlogClient,
            command: $cmd_name,
        ) -> anyhow::Result<()> {
            let id = <$id_type>::new(command.id);
            client.$client_method().delete(id).await?;
            let response = serde_json::json!({
                "status": "deleted",
                "resource": $resource_name,
                "id": id.into_inner(),
            });
            super::print_json(&response)
        }
    };
}

pub(super) use define_delete_command;
pub(super) use define_get_command;
