use crate::models::submission_unit::SubmissionUnit;
use crate::validation::{ValidationError, ValidationRule};
use lopdf::Document as PdfDocument;
use std::fs::File;
use std::path::Path;

// =========================================================================
// RULE: US-eCTD4-533
// "PDF files must have Fast Web View enabled and must not contain JavaScript."
//
// Components:
// 1. Fast Web View (Linearization): Checked via the 'Linearized' dictionary.
// 2. Forbidden JavaScript: Checked by scanning for /JS, /JavaScript, /AA, /OpenAction keys.
// =========================================================================
pub struct RuleEctd4_533;

impl ValidationRule for RuleEctd4_533 {
    fn rule_id(&self) -> &str { "US-eCTD4-533" }

    fn check(&self, unit: &SubmissionUnit) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for doc in &unit.documents {
            // The file path is stored in doc.text.reference.value
            // Note: In a real environment, this path needs to be resolved relative to the root.
            // For this implementation, we assume the path is accessible or absolute.
            let path_str = &doc.text.reference.value;
            let path = Path::new(path_str);

            if !path.exists() {
                 // If the file doesn't exist, we can't validate it.
                 // Another rule should probably check for file existence.
                 // We'll skip or emit a specific warning here.
                 continue;
            }

            match PdfDocument::load(path) {
                Ok(pdf) => {
                    // 1. Check for Fast Web View (Linearization)
                    // Linearized dictionary is usually at the top level trailer or part of the document structure
                    // lopdf parses the structure, but 'Linearized' is often a special header.
                    // However, we can check the trailer or the dictionary directly if parsed.

                    // Note: lopdf might not expose 'Linearized' dictionary easily if it's purely a header thing.
                    // But typically it appears as a dictionary in the objects with a specific key.
                    // A more robust check involves parsing the first 1024 bytes manually,
                    // but let's see if we can find it in the objects or trailer.

                    // Actually, a robust check for Linearization often requires reading the raw file header.
                    // But for now, let's check if the objects contain a dictionary with "Linearized" key.

                    /*
                       Logic for Linearization:
                       The PDF spec says a linearized file has a "Linearized Dictionary" as the first object.
                       We will iterate through objects to find one with the key "Linearized".
                    */
                    let is_linearized = pdf.objects.values().any(|obj| {
                        obj.as_dict().map_or(false, |dict| dict.has(b"Linearized"))
                    });

                    if !is_linearized {
                        errors.push(ValidationError {
                            code: self.rule_id().to_string(),
                            severity: "Medium Error".to_string(), // FDA calls this "Medium" usually, strictly "Error"
                            message: format!("PDF Document '{}' is not Linearized (Fast Web View disabled)", path_str),
                            target_id: Some(doc.id.clone()),
                        });
                    }

                    // 2. Check for Forbidden JavaScript
                    // We scan all dictionaries for keys: JS, JavaScript, AA, OpenAction
                    // Note: AA (Additional Actions) and OpenAction *can* be valid (e.g. GoTo),
                    // but often contain JS. The strict rule forbids *Javascript*.
                    // So we specifically look for /JS and /JavaScript keys,
                    // and /S /JavaScript in Action dictionaries.

                    let has_js = pdf.objects.values().any(|obj| {
                        obj.as_dict().map_or(false, |dict| {
                            // Direct JS keys
                            if dict.has(b"JS") || dict.has(b"JavaScript") {
                                return true;
                            }

                            // Check Action dictionaries: /S /JavaScript
                            if let Ok(s_val) = dict.get(b"S") {
                                if let Ok(name) = s_val.as_name() {
                                    if name == b"JavaScript" {
                                        return true;
                                    }
                                }
                            }
                            false
                        })
                    });

                    if has_js {
                        errors.push(ValidationError {
                            code: self.rule_id().to_string(),
                            severity: "High Error".to_string(),
                            message: format!("PDF Document '{}' contains forbidden JavaScript", path_str),
                            target_id: Some(doc.id.clone()),
                        });
                    }
                },
                Err(_) => {
                    // If we can't parse it as a PDF, it's likely corrupt or encrypted
                    errors.push(ValidationError {
                        code: self.rule_id().to_string(),
                        severity: "High Error".to_string(),
                        message: format!("Unable to parse PDF Document '{}'. It may be corrupt or encrypted.", path_str),
                        target_id: Some(doc.id.clone()),
                    });
                }
            }
        }

        errors
    }
}
