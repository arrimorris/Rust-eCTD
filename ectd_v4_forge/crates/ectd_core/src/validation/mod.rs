use crate::models::submission_unit::SubmissionUnit;
use serde::Serialize;

pub mod rules;

// The structure of a failure
#[derive(Debug, Serialize, Clone)]
pub struct ValidationError {
    pub code: String,      // e.g., "eCTD4-013"
    pub severity: String,  // "High Error", "Warning"
    pub message: String,   // "Sequence Number must be between 1 and 999999"
    pub target_id: Option<String>, // Which element failed?
}

// The contract every rule must fulfill
pub trait ValidationRule {
    fn check(&self, unit: &SubmissionUnit) -> Vec<ValidationError>;
    fn rule_id(&self) -> &str;
}

// The Engine that holds the registry of all rules
pub struct ValidationEngine {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ValidationEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule<R: ValidationRule + 'static>(mut self, rule: R) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn run(&self, unit: &SubmissionUnit) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        for rule in &self.rules {
            let mut rule_errors = rule.check(unit);
            errors.append(&mut rule_errors);
        }
        errors
    }
}
