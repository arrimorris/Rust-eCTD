use bollard::Docker;

pub struct AppState {
    pub docker: Docker,
}

impl AppState {
    pub fn new() -> Self {
        let docker = Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker Daemon");
        Self { docker }
    }
}
