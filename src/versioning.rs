use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Dependencies {
    agreed_deps: HashMap<String, DepAgreement>,
}
impl Dependencies {
    pub fn new() -> Self {
        Self {
            agreed_deps: HashMap::new(),
        }
    }
    pub fn insert(
        &mut self,
        dep: String,
        version: String,
        service: String,
    ) -> Result<(), DependencyInsertionError> {
        let agreed = self
            .agreed_deps
            .entry(dep.clone())
            .or_insert_with(|| DepAgreement::new(version.clone()));
        if agreed.version() != version {
            return Err(DependencyInsertionError {
                dep,
                agreement: agreed.clone(),
                incoming_version: version,
                incoming_service: service,
            });
        }
        agreed.insert(service);
        Ok(())
    }
}
impl Default for Dependencies {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone)]
pub struct DependencyInsertionError {
    pub dep: String,
    pub agreement: DepAgreement,
    pub incoming_version: String,
    pub incoming_service: String,
}

#[derive(Debug, Clone)]
pub struct DepAgreement {
    version: String,
    services: Vec<String>,
}
impl DepAgreement {
    pub fn new(version: String) -> Self {
        Self {
            version,
            services: vec![],
        }
    }
    pub fn insert(&mut self, service: String) {
        self.services.push(service);
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn services(&self) -> &[String] {
        &self.services
    }
}
