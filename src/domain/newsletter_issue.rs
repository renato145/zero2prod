pub struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

impl NewsletterIssue {
    pub fn try_new(
        title: String,
        text_content: String,
        html_content: String,
    ) -> Result<Self, String> {
        if title.is_empty() {
            return Err("Title can't be empty.".to_string());
        }
        if text_content.is_empty() {
            return Err("Text content can't be empty.".to_string());
        }
        if html_content.is_empty() {
            return Err("Html content can't be empty.".to_string());
        }
        Ok(Self {
            title,
            text_content,
            html_content,
        })
    }

    /// Get a reference to the newsletter issue's title.
    pub fn title(&self) -> &str {
        self.title.as_ref()
    }

    /// Get a reference to the newsletter issue's text content.
    pub fn text_content(&self) -> &str {
        self.text_content.as_ref()
    }

    /// Get a reference to the newsletter issue's html content.
    pub fn html_content(&self) -> &str {
        self.html_content.as_ref()
    }
}
