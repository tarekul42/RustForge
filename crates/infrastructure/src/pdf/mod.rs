use printpdf::*;
use sw_domain::value_objects::Money;
use sw_domain::value_objects::ids::PaymentId;

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
    let mut doc = PdfDocument::new(&format!("Invoice {}", data.transaction_id));

    let helv = PdfFontHandle::Builtin(BuiltinFont::Helvetica);
    let helv_bold = PdfFontHandle::Builtin(BuiltinFont::HelveticaBold);
    let mut warnings = Vec::new();

    let page = PdfPage::new(
        Mm(210.0),
        Mm(297.0),
        vec![
            Op::StartTextSection,
            Op::SetFont {
                font: helv_bold.clone(),
                size: Pt(24.0),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(270.0)),
            },
            Op::ShowText {
                items: vec!["INVOICE".into()],
            },
            Op::SetFont {
                font: helv.clone(),
                size: Pt(12.0),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(255.0)),
            },
            Op::ShowText {
                items: vec![format!("Invoice #: {}", data.transaction_id).into()],
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(250.0)),
            },
            Op::ShowText {
                items: vec![TextItem::Text("_".repeat(70))],
            },
            Op::SetFont {
                font: helv_bold.clone(),
                size: Pt(14.0),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(240.0)),
            },
            Op::ShowText {
                items: vec!["Bill To:".into()],
            },
            Op::SetFont {
                font: helv.clone(),
                size: Pt(12.0),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(230.0)),
            },
            Op::ShowText {
                items: vec![data.user_name.as_str().into()],
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(220.0)),
            },
            Op::ShowText {
                items: vec![data.user_email.as_str().into()],
            },
            Op::SetFont {
                font: helv_bold.clone(),
                size: Pt(14.0),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(205.0)),
            },
            Op::ShowText {
                items: vec!["Workshop:".into()],
            },
            Op::SetFont {
                font: helv.clone(),
                size: Pt(12.0),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(195.0)),
            },
            Op::ShowText {
                items: vec![data.workshop_title.as_str().into()],
            },
            Op::SetFont {
                font: helv_bold,
                size: Pt(14.0),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(175.0)),
            },
            Op::ShowText {
                items: vec!["Amount Paid:".into()],
            },
            Op::SetFont {
                font: helv.clone(),
                size: Pt(12.0),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(165.0)),
            },
            Op::ShowText {
                items: vec![format!("BDT {:.2}", data.amount.cents() as f64 / 100.0).into()],
            },
            Op::SetFont {
                font: helv.clone(),
                size: Pt(10.0),
            },
            Op::SetTextCursor {
                pos: Point::new(Mm(20.0), Mm(30.0)),
            },
            Op::ShowText {
                items: vec!["Thank you for your purchase!".into()],
            },
            Op::EndTextSection,
        ],
    );

    doc.with_pages(vec![page]);
    Ok(doc.save(&PdfSaveOptions::default(), &mut warnings))
}

/// Errors from PDF generation.
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    /// PDF generation failed.
    #[error("PDF generation failed: {0}")]
    GenerationFailed(String),
}
