use printpdf::*;
use sw_domain::value_objects::ids::PaymentId;
use sw_domain::value_objects::Money;

/// Invoice data required for PDF generation.
pub struct InvoiceData {
    /// Payment transaction ID.
    pub transaction_id: String,
    /// Payment ID.
    pub payment_id: PaymentId,
    /// User's full name.
    pub user_name: String,
    /// User's email.
    pub user_email: String,
    /// Workshop title.
    pub workshop_title: String,
    /// Amount paid.
    pub amount: Money,
}

/// Generate a PDF invoice from the given data.
///
/// Returns the raw PDF bytes.
pub fn generate_invoice(data: &InvoiceData) -> Result<Vec<u8>, PdfError> {
    let (doc, page1, layer1) = PdfDocument::new(
        format!("Invoice {}", data.transaction_id),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| PdfError::GenerationFailed(e.to_string()))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| PdfError::GenerationFailed(e.to_string()))?;

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Title
    current_layer.use_text("INVOICE", 24.0, Mm(20.0), Mm(270.0), &font_bold);

    // Transaction ID
    current_layer.use_text(
        format!("Invoice #: {}", data.transaction_id),
        12.0,
        Mm(20.0),
        Mm(255.0),
        &font,
    );

    // Separator
    current_layer.use_text("_".repeat(70), 10.0, Mm(20.0), Mm(250.0), &font);

    // Customer info
    current_layer.use_text("Bill To:", 14.0, Mm(20.0), Mm(240.0), &font_bold);
    current_layer.use_text(&data.user_name, 12.0, Mm(20.0), Mm(230.0), &font);
    current_layer.use_text(&data.user_email, 12.0, Mm(20.0), Mm(220.0), &font);

    // Workshop info
    current_layer.use_text("Workshop:", 14.0, Mm(20.0), Mm(205.0), &font_bold);
    current_layer.use_text(&data.workshop_title, 12.0, Mm(20.0), Mm(195.0), &font);

    // Amount
    current_layer.use_text("Amount Paid:", 14.0, Mm(20.0), Mm(175.0), &font_bold);
    current_layer.use_text(
        format!("BDT {:.2}", data.amount.cents() as f64 / 100.0),
        12.0,
        Mm(20.0),
        Mm(165.0),
        &font,
    );

    // Footer
    current_layer.use_text(
        "Thank you for your purchase!",
        10.0,
        Mm(20.0),
        Mm(30.0),
        &font,
    );

    doc.save_to_bytes()
        .map_err(|e| PdfError::GenerationFailed(e.to_string()))
}

/// Errors from PDF generation.
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    /// PDF generation failed.
    #[error("PDF generation failed: {0}")]
    GenerationFailed(String),
}
