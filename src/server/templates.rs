//! HTML templates with askama

use askama::Template;

/// List keywords template
#[derive(Debug, Clone, Template)]
#[template(path = "list_keywords.htm")]
pub struct ListKeywords {
	pub keywords: Vec<String>,
}
