//! PDF Form Filling
//!
//! Interactive forms, signature capture, form field creation, data export.

use std::collections::HashMap;
use crate::pdf::{FormField, FormFieldType, PDFError};

/// Form manager
pub struct FormManager {
    fields: HashMap<String, FormField>,
    values: HashMap<String, String>,
}

impl FormManager {
    /// Create a new form manager
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            values: HashMap::new(),
        }
    }

    /// Load form fields from document
    pub async fn load_fields(&mut self, fields: Vec<FormField>) -> Result<(), PDFError> {
        self.fields.clear();
        self.values.clear();

        for field in fields {
            self.fields.insert(field.id.clone(), field.clone());
            self.values.insert(field.id.clone(), field.value.clone());
        }

        log::info!("Loaded {} form fields", self.fields.len());
        Ok(())
    }

    /// Get all fields
    pub async fn get_fields(&self) -> Result<Vec<FormField>, PDFError> {
        Ok(self.fields.values().cloned().collect())
    }

    /// Get field by ID
    pub fn get_field(&self, id: &str) -> Option<&FormField> {
        self.fields.get(id)
    }

    /// Fill form field
    pub async fn fill_field(&mut self, id: &str, value: &str) -> Result<(), PDFError> {
        if !self.fields.contains_key(id) {
            return Err(PDFError::FormError(format!("Field not found: {}", id)));
        }

        let field = self.fields.get(id).unwrap();
        if field.is_readonly {
            return Err(PDFError::FormError(format!("Field is readonly: {}", id)));
        }

        // Validate field type
        let field_type = field.field_type;
        if !self.validate_field_type(field_type, value) {
            return Err(PDFError::FormError(format!("Invalid value for field type: {:?}", field_type)));
        }

        self.values.insert(id.to_string(), value.to_string());
        log::info!("Filled field {} with value: {}", id, value);
        Ok(())
    }

    /// Get field value
    pub fn get_value(&self, id: &str) -> Option<&String> {
        self.values.get(id)
    }

    /// Get all values
    pub fn get_all_values(&self) -> HashMap<String, String> {
        self.values.clone()
    }

    /// Validate field type
    fn validate_field_type(&self, field_type: FormFieldType, value: &str) -> bool {
        match field_type {
            FormFieldType::Text | FormFieldType::FreeText => true, // Any text is valid
            FormFieldType::Checkbox => value == "true" || value == "false" || value == "" {
                value.parse::<bool>().is_ok()
            }
            FormFieldType::RadioButton => !value.is_empty(),
            FormFieldType::Dropdown => true, // Assume dropdown options are pre-defined
            FormFieldType::ListBox => true,
            FormFieldType::Signature => !value.is_empty(),
            FormFieldType::Date => {
                // Simple date validation
                value.contains("-") || value.contains("/")
            }
            FormFieldType::Email => {
                value.contains("@") && value.contains(".")
            }
            FormFieldType::Number => {
                value.parse::<f64>().is_ok()
            }
        }
    }

    /// Check if form is complete
    pub fn is_complete(&self) -> bool {
        for field in self.fields.values() {
            if field.is_required && self.values.get(&field.id).map_or(true, |v| v.is_empty()) {
                return false;
            }
        }
        true
    }

    /// Reset form to default values
    pub fn reset(&mut self) {
        for (id, field) in &self.fields {
            self.values.insert(id.clone(), field.value.clone());
        }
        log::info!("Form reset to default values");
    }

    /// Clear all field values
    pub fn clear(&mut self) {
        self.values.clear();
        for id in self.fields.keys() {
            self.values.insert(id.clone(), String::new());
        }
        log::info!("Form cleared");
    }

    /// Export form data to FDF format
    pub fn export_fdf(&self) -> Result<String, PDFError> {
        let mut fdf = String::from("%FDF-1.2\n");
        fdf.push_str("1 0 obj\n<<\n/FDF <<\n/Fields [\n");

        for (id, value) in &self.values {
            if let Some(field) = self.fields.get(id) {
                fdf.push_str(&format!(
                    "<< /V ({}) /T ({}) /FT /{:?} >>\n",
                    value, field.name, field.field_type
                ));
            }
        }

        fdf.push_str("]\n>>\n>>\nendobj\n");
        fdf.push_str("trailer\n<<\n/Root 1 0 R\n>>\n");
        fdf.push_str("%%EOF\n");

        Ok(fdf)
    }

    /// Export form data to XFDF format (XML-based)
    pub fn export_xfdf(&self) -> Result<String, PDFError> {
        let mut xfdf = String::from("<?xml version=&quot;1.0&quot; encoding=&quot;UTF-8&quot;?>\n");
        xfdf.push_str("<xfdf xmlns=&quot;http://ns.adobe.com/xfdf/&quot; xml:space=&quot;preserve&quot;>\n");
        xfdf.push_str("  <fields>\n");

        for (id, value) in &self.values {
            if let Some(field) = self.fields.get(id) {
                xfdf.push_str(&format!(
                    "    <field name=&quot;{}&quot;><value>{}</value></field>\n",
                    field.name, value
                ));
            }
        }

        xfdf.push_str("  </fields>\n");
        xfdf.push_str("</xfdf>");

        Ok(xfdf)
    }

    /// Export form data to JSON format
    pub fn export_json(&self) -> Result<String, PDFError> {
        let mut json = String::from("{\n");
        let mut first = true;

        for (id, value) in &self.values {
            if let Some(field) = self.fields.get(id) {
                if !first {
                    json.push_str(",\n");
                }
                first = false;
                json.push_str(&format!(
                    "  &quot;{}&quot;: &quot;{}&quot;",
                    field.name, value
                ));
            }
        }

        json.push_str("\n}");
        Ok(json)
    }

    /// Import form data
    pub async fn import_data(&mut self, data: &FormDataImport) -> Result<(), PDFError> {
        match data {
            FormDataImport::FDF(content) => self.import_fdf(content).await,
            FormDataImport::XFDF(content) => self.import_xfdf(content).await,
            FormDataImport::JSON(content) => self.import_json(content).await,
        }
    }

    /// Import FDF data
    async fn import_fdf(&mut self, content: &str) -> Result<(), PDFError> {
        // In a real implementation, this would parse FDF format
        log::info!("Importing FDF data");
        Ok(())
    }

    /// Import XFDF data
    async fn import_xfdf(&mut self, content: &str) -> Result<(), PDFError> {
        // In a real implementation, this would parse XFDF (XML) format
        log::info!("Importing XFDF data");
        Ok(())
    }

    /// Import JSON data
    async fn import_json(&mut self, content: &str) -> Result<(), PDFError> {
        // In a real implementation, this would parse JSON format
        log::info!("Importing JSON data");
        Ok(())
    }

    /// Create new form field
    pub fn create_field(&mut self, field: FormField) -> Result<(), PDFError> {
        self.fields.insert(field.id.clone(), field.clone());
        self.values.insert(field.id.clone(), field.value.clone());
        Ok(())
    }

    /// Delete form field
    pub fn delete_field(&mut self, id: &str) -> Result<(), PDFError> {
        self.fields.remove(id);
        self.values.remove(id);
        Ok(())
    }

    /// Get required fields
    pub fn get_required_fields(&self) -> Vec<&FormField> {
        self.fields.values()
            .filter(|f| f.is_required)
            .collect()
    }

    /// Get empty required fields
    pub fn get_empty_required_fields(&self) -> Vec<&FormField> {
        self.fields.values()
            .filter(|f| f.is_required && self.values.get(&f.id).map_or(true, |v| v.is_empty()))
            .collect()
    }
}

impl Default for FormManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Form data import format
#[derive(Debug, Clone)]
pub enum FormDataImport {
    FDF(String),
    XFDF(String),
    JSON(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_form_manager_creation() {
        let manager = FormManager::new();
        assert_eq!(manager.fields.len(), 0);
    }

    #[tokio::test]
    async fn test_load_fields() {
        let mut manager = FormManager::new();
        let fields = vec![
            FormField {
                id: "field1".to_string(),
                field_type: FormFieldType::Text,
                name: "Name".to_string(),
                value: String::new(),
                is_required: true,
                is_readonly: false,
                bounds: (100.0, 100.0, 200.0, 30.0),
                page: 1,
            }
        ];
        
        manager.load_fields(fields).await.unwrap();
        assert_eq!(manager.fields.len(), 1);
    }

    #[tokio::test]
    async fn test_fill_field() {
        let mut manager = FormManager::new();
        let fields = vec![
            FormField {
                id: "field1".to_string(),
                field_type: FormFieldType::Text,
                name: "Name".to_string(),
                value: String::new(),
                is_required: true,
                is_readonly: false,
                bounds: (100.0, 100.0, 200.0, 30.0),
                page: 1,
            }
        ];
        
        manager.load_fields(fields).await.unwrap();
        manager.fill_field("field1", "John Doe").await.unwrap();
        
        assert_eq!(manager.get_value("field1").unwrap(), "John Doe");
    }

    #[tokio::test]
    async fn test_fill_readonly_field() {
        let mut manager = FormManager::new();
        let fields = vec![
            FormField {
                id: "field1".to_string(),
                field_type: FormFieldType::Text,
                name: "Name".to_string(),
                value: String::new(),
                is_required: false,
                is_readonly: true,
                bounds: (100.0, 100.0, 200.0, 30.0),
                page: 1,
            }
        ];
        
        manager.load_fields(fields).await.unwrap();
        let result = manager.fill_field("field1", "Test").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_is_complete() {
        let mut manager = FormManager::new();
        let fields = vec![
            FormField {
                id: "field1".to_string(),
                field_type: FormFieldType::Text,
                name: "Name".to_string(),
                value: String::new(),
                is_required: true,
                is_readonly: false,
                bounds: (100.0, 100.0, 200.0, 30.0),
                page: 1,
            }
        ];
        
        manager.fields.insert("field1".to_string(), fields[0].clone());
        manager.values.insert("field1".to_string(), String::new());
        
        assert!(!manager.is_complete());
        
        manager.values.insert("field1".to_string(), "John".to_string());
        assert!(manager.is_complete());
    }

    #[test]
    fn test_export_json() {
        let mut manager = FormManager::new();
        let field = FormField {
            id: "field1".to_string(),
            field_type: FormFieldType::Text,
            name: "Name".to_string(),
            value: "John".to_string(),
            is_required: false,
            is_readonly: false,
            bounds: (100.0, 100.0, 200.0, 30.0),
            page: 1,
        };
        
        manager.fields.insert("field1".to_string(), field);
        manager.values.insert("field1".to_string(), "John".to_string());
        
        let json = manager.export_json().unwrap();
        assert!(json.contains("Name"));
        assert!(json.contains("John"));
    }
}