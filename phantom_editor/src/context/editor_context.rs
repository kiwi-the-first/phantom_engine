use phantom_core::ecs::World;
use phantom_project::phantom_project::PhantomProject;

pub struct EditorContext {
    pub project: PhantomProject,
    pub active_world: World,
}

impl EditorContext {
    pub fn new(project: PhantomProject, world: World) -> Self {
        Self {
            project: project,
            active_world: world,
        }
    }
}
