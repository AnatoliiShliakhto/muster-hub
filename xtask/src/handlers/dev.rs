use crate::models::args::DevAction;
use crate::services::docker::DockerCompose;
use anyhow::Result;

pub fn handle_dev_command(action: DevAction) -> Result<()> {
    let docker = DockerCompose::new();

    match action {
        DevAction::Up {} => {
            docker.up()?;
            println!("\n✨ Infrastructure is ready.");
            println!("🔗 SurrealDB: ws://localhost:8101");
        },
        DevAction::Down { volumes } => {
            docker.down(volumes)?;
        },
        DevAction::Logs { service } => {
            docker.logs(service.as_deref())?;
        },
    }

    Ok(())
}
