use std::collections::HashMap;
use std::sync::Arc;
use crate::adapter::RadioAdapter;
use crate::schema::DeviceModel;

/// Routes device operations to the correct adapter based on model or transport type.
pub struct AdapterRouter {
    adapters: HashMap<String, Arc<dyn RadioAdapter>>,
}

impl AdapterRouter {
    pub fn new() -> Self {
        Self { adapters: HashMap::new() }
    }

    pub fn register(&mut self, adapter: Arc<dyn RadioAdapter>) {
        for model in adapter.supported_models() {
            self.adapters.insert(model, adapter.clone());
        }
    }

    pub fn resolve(&self, model: &DeviceModel) -> Option<Arc<dyn RadioAdapter>> {
        self.adapters.get(model).cloned()
    }
}

impl Default for AdapterRouter {
    fn default() -> Self {
        Self::new()
    }
}
