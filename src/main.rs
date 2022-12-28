use std::process::exit;
use log::{debug, error, info};
use log4rs;
use crate::invoice::invoice::{CompletedInvoices};

mod settings;
mod invoice;

fn main() {
    log4rs::init_file("logging_config.yaml", Default::default()).unwrap();
    info!("Attempting to Load settings file");

    let config = match settings::load_settings() {
        Ok(config) => config,
        Err(e) => {
            error!("Error loading settings file: {}", e);
            exit(1);
        }
    };

    if config.validate() {
        info!("Settings file loaded successfully");
    } else {
        error!("Settings file failed validation, attempting to get new config from user");

        settings::update_settings();
    }

    info!("Parsing invoices.csv file");
    let invoices = invoice::invoice::read_invoice_csv("invoices.csv".into());
    info!("Found {} invoices", invoices.len());

    info!("Parsing done_invoices.csv file");

    let done_invoices = invoice::
    invoice::try_read_done_invoices_csv("done_invoices.csv".into());
    info!("Found {} done invoices", done_invoices.len());

    let to_upload = invoices.iter().filter(|invoice| {
        // checks if the invoice number is in done_invoices
        !done_invoices.iter().any(|done_invoice| invoice.invoice_number() == done_invoice.invoice_number())
    }).collect::<Vec<_>>();

    info!("Found {} invoices to upload", to_upload.len());
    let mut invoices_uploaded = done_invoices;
    for invoice in to_upload {
        debug!("Uploading invoice {}", invoice.invoice_number());
        let time_start = std::time::Instant::now();
        invoices_uploaded.push(CompletedInvoices::new(invoice));
        info!("Uploaded invoice {} in {}ms", invoice.invoice_number(), time_start.elapsed().as_millis());
    }

    info!("Writing done_invoices.csv file");
    invoice::invoice::write_done_invoice_csv(&"done_invoices.csv".to_string(), &invoices_uploaded);

}
